//! JALUR B — skema per-tipe n=1 yang diverifikasi-UPSTREAM (n8n
//! INodeProperties), semantik RULING 45a/45b + RULING 54e. 50 entri:
//! 44 node biasa + 6 varian Tool SINTETIS (`n8n-nodes-base.<x>Tool` —
//! tak punya deskripsi mandiri; disintesis dari node base usableAsTool
//! oleh packages/cli/src/node-types.ts + convertNodeToAiTool
//! (ai-tools.ts); verified field konverter merujuk ai-tools.ts, field
//! base merujuk deklarasi INodeProperties base).
//!
//! RULING 46a — TIGA kategori provenance (jangan digabung):
//!   44 non-Tool: `upstream-verified (INodeProperties @ <anchor>)`;
//!   6 Tool:      `upstream-tool-generated (base @ <anchor> + aturan
//!                injeksi tool …)` — bila n8n mengubah parameter injeksi,
//!                hanya kategori ini yang wajib diverifikasi ulang.
//!
//! Provenance per-entri: `upstream-verified (INodeProperties @ <anchor>)`
//! (RULING 46a). Anchor default `n8n 2.38.5 fcf21f5e`; pengecualian per
//! entri dicatat (mis. sendInBlue @ n8n@1.0.0).
//!
//! Provenance per-field:
//!   - `verified(file:line)` — bukti deklarasi INodeProperties (54e
//!     butir 2; `verified` = non-kosong utk semua field non-legacy);
//!   - `corpus-legacy` — pseudo-field 0-hit di UI anchor, hanya teramati
//!     di korpus (type_ui `?`, TIDAK boleh required_ui, tak ada verified).
//!     Tercatat 3: airtableTrigger.pollTimes, perplexity.requestOptions,
//!     googleAds.requestOptions.
//!
//! 54e (matt #1542) — bentuk skema TERKUNCI:
//!   - `required_ui` (kontrak UI) → validasi field-absen, DENGAN kondisi
//!     default: absen ditolak HANYA bila required_ui=true DAN field tidak
//!     memuat `default:` statis.
//!   - Tripwire butir 3: test menegaskan INVARIANT data — setiap field
//!     `required_ui=true` WAJIB punya default statis non-null (bila
//!     upstream berubah, re-ekstraksi melanggar dan test menyala).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Konstanta COMPILE-TIME (pola RULING 50c): bukan input runtime.
const JALUR_B_JSON: &str = include_str!("../data/param_schemas_jalur_b.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JalurBField {
    /// Nama kunci parameter persis seperti di JSON workflow n8n.
    pub name: String,
    /// Kontrak UI INodeProperties (n8n `required: true` di deklarasi).
    #[serde(default)]
    pub required_ui: bool,
    /// Tipe UI n8n (string/number/boolean/options/multiOptions/collection/
    /// fixedCollection/resourceLocator/json/...). `?` = tak terpetakan
    /// (corpus-legacy).
    #[serde(default)]
    pub type_ui: Option<String>,
    /// Nilai `default:` statis dari deklarasi upstream (54e: KEBERADAAN
    /// default statis yang membolehkan absen; nilai dicatat utk dokumentasi).
    #[serde(default)]
    pub default: Option<Value>,
    /// `file:line` bukti deklarasi INodeProperties (54e butir 2).
    #[serde(default)]
    pub verified: Option<String>,
    /// `corpus-legacy` utk pseudo-field korpus; selain itu None.
    #[serde(default)]
    pub provenance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JalurBEntry {
    /// `type_full` persis seperti di JSON workflow.
    pub type_full: String,
    /// typeVersion entri korpus (string di data: "1", "4.7", ...).
    #[serde(default, rename = "typeVersion")]
    pub type_version: String,
    /// n = 1 (definisi Jalur B).
    #[serde(default)]
    pub n: usize,
    /// Anchor versi n8n (2.38.5 fcf21f5e default; khusus per entri bila beda).
    #[serde(default)]
    pub anchor: Option<String>,
    /// `upstream-verified (INodeProperties @ …)`.
    #[serde(default)]
    pub provenance: Option<String>,
    /// `provisional` sampai bentuk disetujui penuh.
    #[serde(default)]
    pub provisional: Option<bool>,
    /// Konteks operasi (resource/operation default; catatan anomali).
    #[serde(default)]
    pub operation_context: Option<String>,
    /// Field permukaan (termasuk pseudo-field corpus-legacy).
    pub fields: Vec<JalurBField>,
}

pub fn jalur_b_entries() -> Vec<JalurBEntry> {
    serde_json::from_str(JALUR_B_JSON).expect("param_schemas_jalur_b.json valid")
}

pub fn jalur_b_for<'a>(entries: &'a [JalurBEntry], type_full: &str) -> Option<&'a JalurBEntry> {
    entries.iter().find(|e| e.type_full == type_full)
}

/// 54e — invariant data (tripwire butir 3): setiap field required_ui=true
/// WAJIB memuat `default:` statis non-null. Bila upstream berubah (default
/// dicabut), re-ekstraksi melanggar → test menyala.
pub fn tripwire_violations(entries: &[JalurBEntry]) -> Vec<String> {
    let mut v = Vec::new();
    for e in entries {
        for f in &e.fields {
            if f.required_ui && f.default.is_none() {
                v.push(format!(
                    "{}: field `{}` required_ui=true TANPA default statis (54e)",
                    e.type_full, f.name
                ));
            }
        }
    }
    v
}

/// Validasi parameter nyata thd entri Jalur B (54e):
///   - absen: ditolak hanya bila `required_ui=true` && tanpa default statis;
///   - kunci tak dikenal: ditolak (permukaan tertutup = fields tercatat,
///     termasuk pseudo-field corpus-legacy yang dipakai korpus);
///   - tipe: tidak diperiksa di v1 (type_ui tak memetakan 1:1 ke JSON kind;
///     ekstraksi sudah memverifikasi bentuk per field).
pub fn validate_jalur_b(params: &Map<String, Value>, entry: &JalurBEntry) -> Vec<String> {
    let mut errs = Vec::new();
    for f in &entry.fields {
        if !params.contains_key(&f.name) && f.required_ui && f.default.is_none() {
            errs.push(format!(
                "{}: field `{}` absen (required_ui=true tanpa default statis — 54e)",
                entry.type_full, f.name
            ));
        }
    }
    for k in params.keys() {
        if !entry.fields.iter().any(|f| &f.name == k) {
            errs.push(format!(
                "{}: parameter tak dikenal `{k}` (permukaan upstream + corpus-legacy tertutup)",
                entry.type_full
            ));
        }
    }
    errs
}

/// Catatan discrepancy (bukan error): absen-dgn-default = sah.
pub fn validate_jalur_b_notes(params: &Map<String, Value>, entry: &JalurBEntry) -> Vec<String> {
    let mut notes = Vec::new();
    for f in &entry.fields {
        if params.contains_key(&f.name) {
            continue;
        }
        if f.required_ui && f.default.is_some() {
            notes.push(format!(
                "{}: `{}` absen = nilai default (sah; n8n tak menserialisasi nilai = default)",
                entry.type_full, f.name
            ));
        }
    }
    notes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jalur_b_data_terkunci_50_entri_dan_tripwire_54e_bersih() {
        let entries = jalur_b_entries();
        assert_eq!(
            entries.len(),
            50,
            "Jalur B data = 50/50 (per 2026-09-10; 44 node + 6 varian Tool sintetis)"
        );
        for e in &entries {
            assert_eq!(e.n, 1, "{}: Jalur B wajib n=1", e.type_full);
            // RULING 46a: TIGA kategori provenance; dua utk Jalur B —
            // jangan digabung. 44 non-Tool = upstream-verified; 6 Tool =
            // upstream-tool-generated (dari aturan generasi).
            let prov = e.provenance.as_deref().unwrap_or("");
            let is_tool = e.type_full.ends_with("Tool");
            if is_tool {
                assert!(
                    prov.starts_with("upstream-tool-generated (base @"),
                    "{}: kategori tool (46a), dapat: {prov}",
                    e.type_full
                );
            } else {
                assert!(
                    prov.starts_with("upstream-verified"),
                    "{}: kategori non-tool (46a), dapat: {prov}",
                    e.type_full
                );
            }
            let anc = e.anchor.as_deref().unwrap_or("");
            assert!(
                anc.starts_with("n8n 2.38.5") || anc.starts_with("n8n@1.0.0"),
                "{}: anchor sah, dapat: {anc}",
                e.type_full
            );
            let mut names: Vec<&str> = e.fields.iter().map(|f| f.name.as_str()).collect();
            names.sort_unstable();
            let uniq: std::collections::BTreeSet<&str> = names.iter().copied().collect();
            assert_eq!(names.len(), uniq.len(), "{}: nama field duplikat", e.type_full);
            for f in &e.fields {
                // TRIPWIRE (54e butir 3): required_ui=true ⇒ default non-null
                if f.required_ui {
                    assert!(
                        f.default.is_some(),
                        "{}:{} required_ui tanpa default = pelanggaran 54e",
                        e.type_full,
                        f.name
                    );
                    assert_ne!(
                        f.provenance.as_deref(),
                        Some("corpus-legacy"),
                        "{}:{} legacy tak boleh required_ui",
                        e.type_full,
                        f.name
                    );
                }
                match f.provenance.as_deref() {
                    // legacy: pseudo-field korpus 0-hit UI — tak bisa verified
                    Some("corpus-legacy") => {
                        assert!(
                            f.verified.is_none(),
                            "{}:{} corpus-legacy tak boleh punya verified",
                            e.type_full,
                            f.name
                        );
                        assert_eq!(f.type_ui.as_deref(), Some("?"), "{}:{} legacy type_ui '?'", e.type_full, f.name);
                    }
                    // non-legacy: butir-2 54e — wajib verified(file:line)
                    _ => {
                        assert!(
                            f.verified.is_some(),
                            "{}:{} wajib verified (54e butir 2) — non-legacy",
                            e.type_full,
                            f.name
                        );
                    }
                }
            }
        }
        let viol = tripwire_violations(&entries);
        assert!(
            viol.is_empty(),
            "tripwire 54e menyala (required_ui tanpa default):\n  {}",
            viol.join("\n  ")
        );
    }

    #[test]
    fn jalur_b_enam_varian_tool_sintetis_lengkap() {
        // 50/50: enam varian Tool sintetis wajib ada, anchor 2.38.5,
        // toolDescription = deklarasi konverter (ai-tools.ts) & ber-default.
        let entries = jalur_b_entries();
        for t in [
            "n8n-nodes-base.cryptoTool",
            "n8n-nodes-base.dateTimeTool",
            "n8n-nodes-base.googleSheetsTool",
            "n8n-nodes-base.httpRequestTool",
            "n8n-nodes-base.rssFeedReadTool",
            "n8n-nodes-base.telegramTool",
        ] {
            let e = jalur_b_for(&entries, t).unwrap_or_else(|| panic!("{t} wajib ada (50/50)"));
            assert_eq!(e.anchor.as_deref(), Some("n8n 2.38.5 fcf21f5e"), "{t}: anchor");
            let td = e
                .fields
                .iter()
                .find(|f| f.name == "toolDescription")
                .unwrap_or_else(|| panic!("{t}: toolDescription wajib (konverter)"));
            assert!(td.required_ui, "{t}: toolDescription required di konverter");
            assert!(td.default.is_some(), "{t}: toolDescription default = deskripsi base");
            assert!(
                td.verified
                    .as_deref()
                    .unwrap_or("")
                    .starts_with("packages/cli/src/tool-generation/ai-tools.ts"),
                "{t}: toolDescription verified = deklarasi konverter"
            );
        }
    }

    #[test]
    fn validate_jalur_b_absen_ditolak_hanya_54e() {
        let entry = JalurBEntry {
            type_full: "n8n-nodes-base.testB".into(),
            type_version: "1".into(),
            n: 1,
            anchor: Some("n8n 2.38.5 test".into()),
            provenance: Some("upstream-verified (INodeProperties @ n8n 2.38.5 test)".into()),
            provisional: Some(true),
            operation_context: None,
            fields: vec![
                JalurBField {
                    name: "wajibTanpaDefault".into(),
                    required_ui: true,
                    type_ui: Some("string".into()),
                    default: None,
                    verified: Some("X.node.ts:1".into()),
                    provenance: None,
                },
                JalurBField {
                    name: "wajibDgnDefault".into(),
                    required_ui: true,
                    type_ui: Some("string".into()),
                    default: Some(Value::String("".into())),
                    verified: Some("X.node.ts:9".into()),
                    provenance: None,
                },
            ],
        };
        let p = Map::new();
        let errs = validate_jalur_b(&p, &entry);
        assert_eq!(errs.len(), 1, "hanya wajib-tanpa-default yang menolak: {errs:?}");
        assert!(errs[0].contains("wajibTanpaDefault"));
        assert!(errs[0].contains("54e"));
        let notes = validate_jalur_b_notes(&p, &entry);
        assert!(notes.iter().any(|n| n.contains("wajibDgnDefault")));
        // kunci tak dikenal
        let mut p2 = Map::new();
        p2.insert("wajibTanpaDefault".into(), Value::String("x".into()));
        p2.insert("aneh".into(), Value::Bool(true));
        let errs2 = validate_jalur_b(&p2, &entry);
        assert_eq!(errs2.len(), 1);
        assert!(errs2[0].contains("aneh"));
    }
}
