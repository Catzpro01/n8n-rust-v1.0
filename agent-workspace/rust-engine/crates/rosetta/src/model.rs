//! Model IR Rosetta M1 — grafik kanonik yang mempertahankan SEMUA informasi
//! sumber (opaque-safe) sambil mengekspos bentuk terstruktur utk migrasi.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Versi tipe node n8n — dapat berupa `1`, `1.2`, `1.3` (bukti korpus:
/// scheduleTrigger 1.1–1.3). Major utk scope aturan; minor dipertahankan
/// supaya tidak ada informasi hilang.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypeVersion {
    /// Versi major.
    pub major: u32,
    /// Versi minor (0 bila sumber menulis integer).
    pub minor: u32,
}

impl TypeVersion {
    /// Versi baru.
    ///
    /// Konstrain minor 0..=9 TIDAK diasumsikan empiris di sini: klaim
    /// "minor n8n selalu 0..=9" (komentar lama) TIDAK diverifikasi saat
    /// ditulis. Pengukuran korpus 2026-09-09 (171 fixture, 2.260 node;
    /// scan nilai + scan teks mentah): minor maksimum = 9
    /// (chainLlm typeVersion 1.9, tpl-11807), nilai desimal 2 digit = 0.
    /// Guard fail-closed di bawah menegakkan konstrain itu per nilai;
    /// kalau upstream suatu hari mengirim minor dua digit, hasilnya
    /// kegagalan TERLIHAT (None + typeVersion_raw), bukan peleburan
    /// senyap (temuan matt P1, #1188).
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Parse dari JSON value n8n: number (`1`, `1.0`, `1.2`) atau string.
    pub fn from_value(v: &Value) -> Option<Self> {
        match v {
            Value::Number(n) => n.as_f64().and_then(Self::from_f64),
            Value::String(s) => Self::parse_str(s),
            _ => None,
        }
    }

    /// Parse dari f64 (`1.0` → major 1 minor 0; `1.2` → 1/2).
    ///
    /// FAIL-CLOSED (temuan matt #1188): aritmetika f64 lama melebur minor
    /// dua digit secara senyap (1.12 → major 1 minor 1, guard lolos).
    /// Sekarang skala ×10 harus mendekati bilangan bulat dalam 1e-9;
    /// 1.12 → 11.200000000000001 → selisih 0.2 > epsilon → None
    /// (terlihat), sementara 1.1/1.2/4.2 → selisih ~1e-15 → sah.
    /// Keterbatasan jujur: JSON number `4.10` tak bisa dibedakan dari
    /// `4.1` setelah parse f64 — jalur string (parse_str) menutupnya
    /// fail-closed untuk sumber tekstual.
    pub fn from_f64(f: f64) -> Option<Self> {
        if !f.is_finite() || f < 0.0 {
            return None;
        }
        let major = f.trunc();
        let scaled = f * 10.0;
        let rounded = scaled.round();
        if (scaled - rounded).abs() > 1e-9 {
            return None; // pecahan ≥2 digit desimal: tolak keras
        }
        let minor = rounded as i64 - (major as i64) * 10;
        if !(0..=9).contains(&minor) {
            return None;
        }
        Some(Self {
            major: major as u32,
            minor: minor as u32,
        })
    }

    /// Parse dari representasi string (`"1"`, `"1.2"`).
    ///
    /// Digit-murni (bukan via f64): pecahan harus TEPAT SATU digit
    /// (0..=9); `"1.12"`, `"1.10"`, `"2.39"` → None fail-closed,
    /// `"1"` → minor 0, `"1.0"` → minor 0, `"1.2"` → minor 2.
    pub fn parse_str(s: &str) -> Option<Self> {
        let t = s.trim();
        if t.is_empty() {
            return None;
        }
        let (major_part, minor_part) = match t.split_once('.') {
            Some((a, b)) => (a, Some(b)),
            None => (t, None),
        };
        if major_part.is_empty() || !major_part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let major: u32 = major_part.parse().ok()?;
        let minor: u32 = match minor_part {
            None => 0,
            Some(mp) => {
                if mp.len() != 1 || !mp.bytes().all(|b| b.is_ascii_digit()) {
                    return None; // pecahan 2+ digit / kosong / non-digit → tolak
                }
                mp.parse().ok()?
            }
        };
        Some(Self { major, minor })
    }

    /// Representasi JSON n8n: `1` (bila minor 0) atau `1.2`.
    pub fn to_json_value(self) -> Value {
        if self.minor == 0 {
            serde_json::json!(self.major)
        } else {
            serde_json::json!(self.major as f64 + self.minor as f64 / 10.0)
        }
    }

    /// Major.
    pub fn major(self) -> u32 {
        self.major
    }
}

impl std::fmt::Display for TypeVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.minor == 0 {
            write!(f, "{}", self.major)
        } else {
            write!(f, "{}.{}", self.major, self.minor)
        }
    }
}

/// Asal node menurut namespace type-nya (lihat [`crate::catalog::origin_of`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeOrigin {
    /// `n8n-nodes-base.*` — bundle inti n8n.
    N8nBase,
    /// First-party lain milik n8n (`@n8n/*`, mis. langchain).
    N8nFirstParty,
    /// Komunitas (`@scope/pkg.*`) — opaque pada M1 (R-2).
    Community,
    /// Bukan format namespace n8n yang dikenal — opaque (R-2).
    Unknown,
}

impl NodeOrigin {
    /// True bila node tidak termasuk katalog resmi yang diverifikasi (R-2):
    /// dipertahankan utuh, tidak pernah menghentikan impor.
    pub fn is_opaque(self) -> bool {
        matches!(self, NodeOrigin::Community | NodeOrigin::Unknown)
    }
}

/// Catatan node opaque (R-2): tipe di luar katalog resmi, dipertahankan utuh.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpaqueNote {
    /// Indeks node dalam `Workflow::nodes` (urutan file sumber dipertahankan).
    pub node_index: usize,
    /// Nama node (unik dalam workflow).
    pub node_name: String,
    /// Type penuh dari sumber.
    pub type_full: String,
    /// Alasan digolongkan opaque.
    pub reason: String,
}

/// Node dalam grafik kanonik. Semua kunci node sumber selain yang
/// terstruktur (name/type/typeVersion/position/parameters) dipertahankan
/// byte-identik dalam `extra` — tidak ada kehilangan informasi.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanonNode {
    /// Nama node (unik per workflow di n8n).
    pub name: String,
    /// Type penuh dari sumber (mis. `n8n-nodes-base.cron`).
    pub type_full: String,
    /// `typeVersion` dari sumber; `None` bila absen (workflow era lama).
    pub type_version: Option<TypeVersion>,
    /// Klasifikasi namespace.
    pub origin: NodeOrigin,
    /// Posisi kanvas `[x, y]`, bila ada.
    pub position: Option<[f64; 2]>,
    /// `parameters` — dipertahankan UTUH sebagai JSON (normalisasi = M5).
    pub parameters: Value,
    /// Kunci node lain (id, disabled, notes, webhookId, dst.) — preserved.
    pub extra: Map<String, Value>,
}

impl CanonNode {
    /// Versi major efektif: major sumber, atau 1 (default era n8n tanpa
    /// `typeVersion`).
    pub fn effective_major(&self) -> u32 {
        self.type_version.map(|v| v.major).unwrap_or(1)
    }
}

/// Workflow hasil parse (grafik kanonik M1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    /// Nama workflow bila ada di sumber (bukan kunci wajib di file template).
    pub name: Option<String>,
    /// Node dalam URUTAN SUMBER (determinisme R-8).
    pub nodes: Vec<CanonNode>,
    /// `connections` dipertahankan utuh (interpretasi grafik = M2+).
    pub connections: Value,
    /// Kunci workflow lain (settings, pinData, meta, dst.) — preserved.
    pub extra: Map<String, Value>,
    /// Catatan opaque (R-2) — bahan receipt (M3).
    pub opaque_nodes: Vec<OpaqueNote>,
}

impl Workflow {
    /// Jumlah node.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Jumlah node opaque.
    pub fn opaque_count(&self) -> usize {
        self.opaque_nodes.len()
    }

    /// Cari node by name.
    pub fn node(&self, name: &str) -> Option<&CanonNode> {
        self.nodes.iter().find(|n| n.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typeversion_parse_sah_dan_utuh() {
        // number integer & satu desimal
        assert_eq!(TypeVersion::from_f64(1.0), Some(TypeVersion::new(1, 0)));
        assert_eq!(TypeVersion::from_f64(1.2), Some(TypeVersion::new(1, 2)));
        assert_eq!(TypeVersion::from_f64(1.3), Some(TypeVersion::new(1, 3)));
        assert_eq!(TypeVersion::from_f64(4.2), Some(TypeVersion::new(4, 2)));
        // via JSON value number
        assert_eq!(
            TypeVersion::from_value(&serde_json::json!(1.2)),
            Some(TypeVersion::new(1, 2))
        );
        assert_eq!(
            TypeVersion::from_value(&serde_json::json!(1)),
            Some(TypeVersion::new(1, 0))
        );
        // string: integer, satu desimal, nol-desimal
        assert_eq!(TypeVersion::parse_str("1"), Some(TypeVersion::new(1, 0)));
        assert_eq!(TypeVersion::parse_str("1.2"), Some(TypeVersion::new(1, 2)));
        assert_eq!(TypeVersion::parse_str("1.0"), Some(TypeVersion::new(1, 0)));
        assert_eq!(TypeVersion::parse_str(" 4.2 "), Some(TypeVersion::new(4, 2)));
        // via JSON value string
        assert_eq!(
            TypeVersion::from_value(&serde_json::json!("1.2")),
            Some(TypeVersion::new(1, 2))
        );
    }

    #[test]
    fn typeversion_minor_dua_digit_ditolak_fail_closed() {
        // Kasus temuan matt P1 (#1188): aritmetika lama melebur 1.12→{1,1},
        // 2.39→{2,4}, 3.11→{3,1} SENYAP. Sekarang harus None (terlihat).
        assert_eq!(TypeVersion::from_f64(1.12), None);
        assert_eq!(TypeVersion::from_f64(2.39), None);
        assert_eq!(TypeVersion::from_f64(3.11), None);
        assert_eq!(TypeVersion::parse_str("1.12"), None);
        assert_eq!(TypeVersion::parse_str("1.10"), None);
        assert_eq!(TypeVersion::parse_str("2.39"), None);
        // bentuk lain yang tak sah — tolak keras, bukan tebak
        assert_eq!(TypeVersion::parse_str(""), None);
        assert_eq!(TypeVersion::parse_str("1."), None);
        assert_eq!(TypeVersion::parse_str(".5"), None);
        assert_eq!(TypeVersion::parse_str("1.2.3"), None);
        assert_eq!(TypeVersion::parse_str("abc"), None);
        assert_eq!(TypeVersion::parse_str("-1.2"), None);
        assert_eq!(TypeVersion::parse_str("1e2"), None);
        assert_eq!(TypeVersion::from_value(&serde_json::json!("1.12")), None);
        assert_eq!(TypeVersion::from_value(&serde_json::Value::Null), None);
    }

    #[test]
    fn typeversion_roundtrip_dan_display() {
        let tv = TypeVersion::new(1, 2);
        assert_eq!(tv.to_json_value(), serde_json::json!(1.2));
        assert_eq!(tv.to_string(), "1.2");
        let tv0 = TypeVersion::new(4, 0);
        assert_eq!(tv0.to_json_value(), serde_json::json!(4));
        assert_eq!(tv0.to_string(), "4");
    }
}
