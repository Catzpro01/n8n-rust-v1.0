//! R-6 Slice-3 — ParameterSchema terinduksi korpus.
//!
//! RULING 38 §7 (matt #1291): dua jalur + provenance; RULING 38b: setiap
//! skema WAJIB ber-provenance.
//!
//! JALUR A: 22 tipe 1-bentuk keyset, n>=2. provenance:
//! `corpus-induced (n=…, majors=[…])`.
//! JALUR A2 MULTI: 74 tipe multi-bentuk union (dari 75; httpRequest
//! DITUNDA — matt: kerjakan paling akhir). provenance:
//! `corpus-induced-union (n=…, majors=[…], shapes=…)`.
//! httpRequest (274 node) → PALING AKHIR.
//!
//! RULING 54e (matt #1542) — bentuk skema TERKUNCI (mengunci 45a/45b):
//!   - `required_corpus` = OBSERVASI korpus (true: hadir di setiap
//!     instance tipe; false: nol instance membawanya) — klaim, bukan
//!     penolakan: validasi TIDAK menolak field absen atas dasar ini
//!     (skema murni korpus tak bisa menebak keharusan displayOptions);
//!     discrepancy dicatat lewat `validate_notes`, bukan error.
//!   - `required_ui` + `static_default` = kontrak UI yang diketahui.
//!     Field absen ditolak HANYA bila `required_ui==true` dan
//!     `static_default!=true` (54e). Skema A/multi murni korpus ⇒
//!     required_ui=None ⇒ absen tidak pernah ditolak.
//!   - Kunci asing & tipe-menyimpang TETAP ditolak (fail-loud).
//!
//! Data: data/param_schemas_jalur_a.json + data/param_schemas_multi.json
//! (GENERATED dari korpus — regenerasi = scan korpus, jangan edit manual;
//! migrasi 54e = rename `required`→`required_corpus`, sha256 A
//! e2e789947123662b / multi 20a140b52b49ca37).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Satu field parameter (hasil induksi korpus + kontrak UI bila diketahui).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaField {
    /// Nama kunci parameter persis seperti di JSON workflow n8n.
    pub name: String,
    /// OBSERVASI korpus (54e): true = hadir di SEMUA node tipe ini;
    /// false = nol node membawanya. Klaim — TIDAK menolak absen.
    #[serde(default)]
    pub required_corpus: bool,
    /// Kontrak UI (INodeProperties `required: true`) bila diketahui —
    /// skema murni korpus A/multi: None (tak ada bukti UI).
    #[serde(default)]
    pub required_ui: Option<bool>,
    /// Bukti field memuat nilai `default:` statis di UI: bila true, absen
    /// sah meski required_ui=true (n8n tak menserialisasi nilai=default).
    #[serde(default)]
    pub static_default: Option<bool>,
    /// Union tipe JSON yang teramati: string|number|boolean|object|array|null.
    pub kinds: Vec<String>,
    /// Provenance per-field (opsional; skema A/multi: None).
    #[serde(default)]
    pub provenance: Option<String>,
    /// Catatan bentuk khusus (opsional).
    #[serde(default)]
    pub notes: Option<String>,
}

/// ParameterSchema per tipe — WAJIB ber-provenance (RULING 38b).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParamSchema {
    /// `type_full` persis seperti di JSON workflow (kunci kindmap).
    pub type_full: String,
    /// Major typeVersion yang teramati (minor: lihat kindmap/TypeVersion).
    pub majors: Vec<u32>,
    /// Jumlah node korpus yg menjadi dasar induksi.
    pub n: usize,
    /// Field hasil induksi (urutan stabil dari generator).
    pub fields: Vec<SchemaField>,
    /// `corpus-induced (n=…)` / `corpus-induced-union (n=…)` (RULING 38b).
    pub provenance: String,
}

// RULING 50c — situs parse tepercaya (alasan tertulis):
// JALUR_A_JSON/MULTI_JSON adalah konstanta COMPILE-TIME (include_str! dari pohon
// crate, di-generate + dijaga gate param_schema_corpus), bukan input runtime:
// kedalaman terbatas oleh konstruksi generator dan diverifikasi tiap build.
// Guard kedalaman tidak diperlukan di sini.
const JALUR_A_JSON: &str = include_str!("../data/param_schemas_jalur_a.json");
const MULTI_JSON: &str = include_str!("../data/param_schemas_multi.json");

/// Skema Jalur A (1-bentuk, n>=2) — parsing per pemanggilan; data kecil.
pub fn jalur_a_schemas() -> Vec<ParamSchema> {
    serde_json::from_str(JALUR_A_JSON).expect("param_schemas_jalur_a.json valid (generated)")
}

/// Skema Jalur A2 (multi-bentuk union) — parsing per pemanggilan.
pub fn multi_schemas() -> Vec<ParamSchema> {
    serde_json::from_str(MULTI_JSON).expect("param_schemas_multi.json valid (generated)")
}

/// Semua skema korpus: Jalur A + multi (httpRequest belum — deferred).
pub fn all_schemas() -> Vec<ParamSchema> {
    let mut v = jalur_a_schemas();
    v.extend(multi_schemas());
    v
}

/// Cari skema utk tipe; None = tipe tanpa skema korpus (Jalur B / httpRequest / unknown).
pub fn schema_for<'a>(schemas: &'a [ParamSchema], type_full: &str) -> Option<&'a ParamSchema> {
    schemas.iter().find(|s| s.type_full == type_full)
}

/// Kategori tipe JSON (nama sama dgn generator python).
pub fn json_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Validasi parameter nyata thd skema — error dgn pesan yang menunjukkan
/// field & jenis pelanggaran (fail-loud; schema terlalu sempit = test
/// korpus gagal, tidak pernah senyap).
///
/// RULING 54e (mengunci 45a/45b):
///   - absen: ditolak HANYA bila `required_ui==Some(true)` dan
///     `static_default!=Some(true)`. Skema murni korpus (required_ui=None)
///     TIDAK pernah menolak absen — observasi `required_corpus` adalah
///     klaim (lihat `validate_notes` utk discrepancy-logging).
///   - kunci asing: ditolak (skema = himpunan tertutup thd korpus).
///   - tipe menyimpang: ditolak.
pub fn validate(params: &Map<String, Value>, schema: &ParamSchema) -> Vec<String> {
    let mut errs = Vec::new();
    for f in &schema.fields {
        match params.get(&f.name) {
            None => {
                if f.required_ui == Some(true) && f.static_default != Some(true) {
                    errs.push(format!(
                        "{}: field wajib-UI `{}` absen tanpa default statis (54e)",
                        schema.type_full, f.name
                    ));
                }
            }
            Some(v) => {
                let k = json_kind(v);
                if !f.kinds.iter().any(|x| x == k) {
                    errs.push(format!(
                        "{}: field `{}` bertipe {k}, skema hanya menerima [{}]",
                        schema.type_full,
                        f.name,
                        f.kinds.join("|")
                    ));
                }
            }
        }
    }
    // kunci asing: skema 1-bentuk = himpunan tertutup thd korpus;
    // kunci baru = kegagalan TERLIHAT (bukan diterima diam-diam)
    for k in params.keys() {
        if !schema.fields.iter().any(|f| &f.name == k) {
            errs.push(format!(
                "{}: parameter tak dikenal `{k}` (skema korpus = himpunan tertutup)",
                schema.type_full
            ));
        }
    }
    errs
}

/// Catatan (BUKAN error) — discrepancy-logging 54e:
///   - field `required_corpus==true` absen di node (klaim korpus
///     dilanggar node; dicatat jujur, tidak menolak);
///   - field absen dgn `required_ui==true` + `static_default==true`
///     (nilai default dipakai; n8n tak menserialisasi nilai=default).
pub fn validate_notes(params: &Map<String, Value>, schema: &ParamSchema) -> Vec<String> {
    let mut notes = Vec::new();
    for f in &schema.fields {
        if params.contains_key(&f.name) {
            continue;
        }
        if f.required_corpus {
            notes.push(format!(
                "{}: klaim korpus required_corpus=true, field `{}` absen di node ini (discrepancy dicatat, bukan error — 54e)",
                schema.type_full, f.name
            ));
        } else if f.required_ui == Some(true) && f.static_default == Some(true) {
            notes.push(format!(
                "{}: `{}` absen = default statis (sah)",
                schema.type_full, f.name
            ));
        }
    }
    notes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skema_termuat_dan_provenance_lengkap() {
        let ss = jalur_a_schemas();
        assert_eq!(ss.len(), 22, "Jalur A = 22 tipe (scan 2026-09-09)");
        for s in &ss {
            assert!(
                s.provenance.starts_with("corpus-induced (n="),
                "{}: provenance wajib (RULING 38b), dapat: {}",
                s.type_full,
                s.provenance
            );
            assert!(s.n >= 2, "{}: Jalur A wajib n>=2 (n=1 -> Jalur B)", s.type_full);
            assert!(!s.type_full.is_empty());
            assert!(!s.majors.is_empty(), "{}: majors wajib teramati", s.type_full);
            // nama field unik + kinds non-kosong
            let mut names: Vec<&str> = s.fields.iter().map(|f| f.name.as_str()).collect();
            names.sort_unstable();
            let uniq: std::collections::BTreeSet<&str> = names.iter().copied().collect();
            assert_eq!(names.len(), uniq.len(), "{}: nama field duplikat", s.type_full);
            for f in &s.fields {
                assert!(!f.kinds.is_empty(), "{}: field {} tanpa jenis", s.type_full, f.name);
            }
        }
        // Rename 54e (required -> required_corpus): A 1-bentuk keyset => 33/33 true
        let a_total: usize = ss.iter().map(|s| s.fields.len()).sum();
        let a_true: usize = ss
            .iter()
            .flat_map(|s| s.fields.iter())
            .filter(|f| f.required_corpus)
            .count();
        assert_eq!(a_total, 33, "field A total (terkunci)");
        assert_eq!(a_true, 33, "A: keyset 1-bentuk => semua field required_corpus=true");
    }

    #[test]
    fn multi_skema_lengkap_dan_provenance_union() {
        let m = multi_schemas();
        assert_eq!(m.len(), 74, "multi = 74 (75 minus httpRequest deferred; scan 2026-09-09)");
        let m_total: usize = m.iter().map(|s| s.fields.len()).sum();
        let m_true: usize = m
            .iter()
            .flat_map(|s| s.fields.iter())
            .filter(|f| f.required_corpus)
            .count();
        assert_eq!(m_total, 447, "field multi total (terkunci)");
        assert_eq!(m_true, 62, "multi required_corpus=true = irisan (terkunci)");
        for s in &m {
            assert!(
                s.provenance.starts_with("corpus-induced-union (n="),
                "{}: provenance union wajib (RULING 38b), dapat: {}",
                s.type_full,
                s.provenance
            );
            assert!(s.n >= 2, "{}: multi wajib n>=2", s.type_full);
            assert!(!s.fields.is_empty(), "{}: tipe multi pasti punya >=1 kunci", s.type_full);
        }
        // gabungan A+multi tidak tumpang-tindih tipe
        let a = jalur_a_schemas();
        let a_keys: std::collections::BTreeSet<&str> =
            a.iter().map(|s| s.type_full.as_str()).collect();
        for s in &m {
            assert!(!a_keys.contains(s.type_full.as_str()), "{} ada di A dan multi", s.type_full);
        }
        assert_eq!(all_schemas().len(), a.len() + m.len(), "all = A + multi");
        // Skema A/multi MURNI korpus: tidak ada field dgn klaim UI
        for s in a.iter().chain(m.iter()) {
            for f in &s.fields {
                assert_eq!(f.required_ui, None, "{}.{}: data korpus tak punya klaim UI", s.type_full, f.name);
                assert_eq!(f.static_default, None, "{}.{}: data korpus tak punya static_default", s.type_full, f.name);
            }
        }
    }

    #[test]
    fn validasi_54e_absen_ditolak_hanya_required_ui_tanpa_default() {
        let mk = |name: &str, rc: bool, ui: Option<bool>, sd: Option<bool>| SchemaField {
            name: name.to_string(),
            required_corpus: rc,
            required_ui: ui,
            static_default: sd,
            kinds: vec!["string".to_string()],
            provenance: None,
            notes: None,
        };
        let spec = ParamSchema {
            type_full: "n8n-nodes-base.unit54e".into(),
            majors: vec![1],
            n: 22,
            provenance: "corpus-induced (n=22, majors=[1])".into(),
            fields: vec![
                mk("fUiNoDefault", true, Some(true), None),
                mk("fUiDefault", false, Some(true), Some(true)),
                mk("fCorpusOnly", true, None, None),
                mk("fAbsentTotal", false, None, None),
            ],
        };
        let p = Map::new();
        // 54e: hanya fUiNoDefault (required_ui=true tanpa default) menolak
        let errs = validate(&p, &spec);
        assert_eq!(errs.len(), 1, "hanya fUiNoDefault: {errs:?}");
        assert!(errs[0].contains("fUiNoDefault") && errs[0].contains("54e"));
        // relaksasi 45b: klaim korpus (fCorpusOnly, fUiDefault) absen = BUKAN error
        let notes = validate_notes(&p, &spec);
        let n_claim = notes.iter().filter(|n| n.contains("fCorpusOnly")).count();
        let n_def = notes.iter().filter(|n| n.contains("fUiDefault")).count();
        assert_eq!(n_claim, 1, "klaim korpus absen dicatat (discrepancy): {notes:?}");
        assert_eq!(n_def, 1, "absen-dgn-default dicatat: {notes:?}");
        assert!(!notes.iter().any(|n| n.contains("fAbsentTotal")), "fAbsentTotal bukan catatan");
        // fUiNoDefault absen = error 54e SEKALIGUS discrepancy klaim korpus
        // (required_corpus=true) → tercatat di notes; itu perilaku benar.
        // required_corpus=true tidak menolak absen lewat validate()
        assert!(validate(&p, &spec).iter().all(|e| e.contains("fUiNoDefault")));
    }

    #[test]
    fn validasi_tolak_tipe_menyimpang_dan_kunci_asing() {
        let ss = jalur_a_schemas();
        // tipe A dgn field kinds=[string,...] (semua A punya kinds string)
        let s = &ss[0];
        let f0 = &s.fields[0];
        let mut p = Map::new();
        // isi SEMUA field dgn nilai sesuai jenis pertama (biar lolos jenis)
        for f in &s.fields {
            let first = f.kinds.first().map(String::as_str).unwrap_or("string");
            let v = match first {
                "null" => Value::Null,
                "boolean" => Value::Bool(true),
                "number" => Value::from(1),
                "object" => Value::Object(Map::new()),
                "array" => Value::Array(vec![]),
                _ => Value::String("x".into()),
            };
            p.insert(f.name.clone(), v);
        }
        assert!(validate(&p, s).is_empty(), "params sesuai skema harus lolos (54e: absen longgar, hadir wajib cocok)");
        // tipe menyimpang thd kinds
        let mut p2 = p.clone();
        p2.insert(f0.name.clone(), Value::Bool(true));
        let k = json_kind(&p2[f0.name.as_str()]);
        let ok = f0.kinds.iter().any(|x| x == k);
        if !ok {
            assert!(
                !validate(&p2, s).is_empty(),
                "tipe menyimpang harus ditolak ({}: {k} ∉ {:?})",
                f0.name,
                f0.kinds
            );
        }
        // kunci asing ditolak
        let mut p3 = p.clone();
        p3.insert("kunciAsing54e".into(), Value::Bool(true));
        assert!(!validate(&p3, s).is_empty(), "kunci asing harus ditolak");
    }
}
