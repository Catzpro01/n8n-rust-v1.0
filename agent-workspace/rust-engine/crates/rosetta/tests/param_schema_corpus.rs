//! Gate R-6 Slice-3: schema terinduksi korpus (Jalur A + multi) thd korpus
//! NYATA + Jalur B (upstream-verified) thd korpus.
//!
//! RULING 38 §7: gate = diff-test validate thd node nyata — skema yang
//! terlalu sempit/salah akan MENOLAK parameter korpus → kegagalan
//! terlihat. Skema tanpa provenance tidak mungkin ada (assert di modul
//! + di sini).
//!
//! RULING 45b/54e: field-absen ditolak hanya bila required_ui=true tanpa
//! default statis. Skema korpus (A/multi) tanpa bukti UI ⇒ required_ui=None
//! ⇒ absen TIDAK ditolak oleh validate(); klaim `required_corpus` diawasi
//! lewat validate_notes() — pada korpus klaim tsb harus SELALU terpenuhi
//! (field required_corpus=true hadir), jadi notes harus kosong (ini
//! mempertahankan daya deteksi gate lama pasca-relaksasi). Kunci asing &
//! tipe-menyimpang tetap ditolak validate().
//!
//! Tipe dengan skema Jalur B (upstream-verified, n=1): 50 entri = 44 node
//! biasa + 6 varian Tool sintetis (n8n-nodes-base.<x>Tool — disintesis
//! dari node base usableAsTool oleh packages/cli/src/node-types.ts +
//! ai-tools.ts convertNodeToAiTool; verified field konverter =
//! ai-tools.ts, field lain = deklarasi base). Tipe base yang TIDAK punya
//! skema sama sekali tinggal httpRequest (deferred — dikerjakan paling
//! akhir).

use std::collections::BTreeSet;
use std::path::PathBuf;

use rosetta::param_schema::{all_schemas, schema_for, validate, validate_notes};
use rosetta::param_schema_b::{jalur_b_entries, jalur_b_for, validate_jalur_b};
use rosetta::parse::parse_workflow_bytes;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn corpus_files() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(fixtures_dir())
        .expect("tests/fixtures ada")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .collect();
    v.sort();
    v
}

#[test]
fn r6_schema_korpus_validate_lolos_semua_node_base() {
    let schemas = all_schemas(); // Jalur A (22) + multi (74)
    assert_eq!(schemas.len(), 96, "A 22 + multi 74 = 96 (httpRequest deferred; scan 2026-09-09)");

    let files = corpus_files();
    assert_eq!(files.len(), 171);

    let mut validated = 0usize;
    let mut covered_types: BTreeSet<String> = BTreeSet::new();
    let mut unschematized_types: BTreeSet<String> = BTreeSet::new();
    let mut failures: Vec<String> = Vec::new();
    let mut claim_notes: Vec<String> = Vec::new();

    for f in &files {
        let label = f.file_name().unwrap().to_string_lossy().to_string();
        let raw = std::fs::read(f).unwrap_or_else(|e| panic!("read {label}: {e}"));
        let w = parse_workflow_bytes(&raw, &label).unwrap_or_else(|e| panic!("parse {label}: {e}"));
        for n in &w.nodes {
            if !n.type_full.starts_with("n8n-nodes-base.") {
                continue; // non-base: opaque (R-2), di luar slice-3
            }
            match schema_for(&schemas, &n.type_full) {
                Some(s) => {
                    let params = n.parameters.as_object().expect("parameters = objek (parse)");
                    let errs = validate(params, s);
                    if !errs.is_empty() {
                        failures.push(format!("{label} {}: {}", n.name, errs.join("; ")));
                    }
                    // 54e: klaim required_corpus pada korpus HARUS terpenuhi
                    // (data diinduksi dari korpus ini) → notes wajib kosong.
                    let notes = validate_notes(params, s);
                    if !notes.is_empty() {
                        claim_notes.push(format!("{label} {}: {}", n.name, notes.join("; ")));
                    }
                    validated += 1;
                    covered_types.insert(s.type_full.clone());
                }
                None => {
                    unschematized_types.insert(n.type_full.clone());
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} node ditolak skema korpus sendiri:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
    assert!(
        claim_notes.is_empty(),
        "{} node dgn klaim required_corpus absen (data tak sinkron korpus):\n  {}",
        claim_notes.len(),
        claim_notes.join("\n  ")
    );
    assert_eq!(
        covered_types.len(),
        schemas.len(),
        "semua skema (A+multi) harus terpakai oleh >=1 node korpus"
    );
    assert!(validated > 0, "harus ada node tervalidasi");
    // Tipe base tanpa skema A/multi = Jalur B (50 n=1) + httpRequest (deferred) = 51
    assert_eq!(
        unschematized_types.len(),
        51,
        "tanpa skema A/multi = 50 Jalur B + httpRequest deferred: {:?}",
        unschematized_types
    );
}

#[test]
fn r6_jalur_b_validate_lolos_dan_lengkap() {
    // RULING 46a/54e: entri Jalur B (upstream-verified) + gate thd korpus.
    // Node n=1 utk tiap tipe; validate_jalur_b menolak hanya bila
    // required_ui=true tanpa default statis (54e) + kunci tak dikenal.
    let entries = jalur_b_entries();
    assert_eq!(entries.len(), 50, "data Jalur B 50/50 (44 node + 6 varian Tool sintetis)");

    let files = corpus_files();
    let mut covered_b: BTreeSet<String> = BTreeSet::new();
    let mut failures: Vec<String> = Vec::new();
    let mut validated = 0usize;

    for f in &files {
        let label = f.file_name().unwrap().to_string_lossy().to_string();
        let raw = std::fs::read(f).unwrap_or_else(|e| panic!("read {label}: {e}"));
        let w = parse_workflow_bytes(&raw, &label).unwrap_or_else(|e| panic!("parse {label}: {e}"));
        for n in &w.nodes {
            if !n.type_full.starts_with("n8n-nodes-base.") {
                continue;
            }
            if let Some(e) = jalur_b_for(&entries, &n.type_full) {
                let params = n.parameters.as_object().expect("parameters = objek (parse)");
                let errs = validate_jalur_b(params, e);
                if !errs.is_empty() {
                    failures.push(format!("{label} {}: {}", n.name, errs.join("; ")));
                }
                validated += 1;
                covered_b.insert(e.type_full.clone());
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} node Jalur B ditolak entri sendiri:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
    assert_eq!(
        covered_b.len(),
        entries.len(),
        "semua entri Jalur B harus terpakai oleh >=1 node korpus (n=1)"
    );
    assert_eq!(validated, 50, "50 node Jalur B tervalidasi (n=1 per tipe)");
}

#[test]
fn r6_semua_skema_ber_provenance_dan_field_terverifikasi() {
    // RULING 38b ditegakkan dua lapis (modul + gate ini) — A/multi
    for s in all_schemas() {
        assert!(
            s.provenance.starts_with("corpus-induced"),
            "{} provenance salah: {}",
            s.type_full,
            s.provenance
        );
    }
    // 46a + 54e butir 2: entri B dua kategori provenance (jangan digabung):
    // 44 non-Tool upstream-verified + 6 Tool upstream-tool-generated;
    // anchor sah; tiap field non-legacy punya verified(file:line).
    let entries = jalur_b_entries();
    let mut n_tool = 0usize;
    for e in &entries {
        let anc = e.anchor.as_deref().unwrap_or("");
        assert!(
            anc.starts_with("n8n 2.38.5") || anc.starts_with("n8n@1.0.0"),
            "{}: anchor tidak dikenal: {anc}",
            e.type_full
        );
        let prov = e.provenance.as_deref().unwrap_or("");
        if e.type_full.ends_with("Tool") {
            n_tool += 1;
            assert!(
                prov.starts_with("upstream-tool-generated (base @"),
                "{}: kategori tool (46a): {prov}",
                e.type_full
            );
        } else {
            assert!(
                prov.starts_with("upstream-verified (INodeProperties @"),
                "{}: provenance upstream-verified (46a): {prov}",
                e.type_full
            );
        }
        for f in &e.fields {
            if f.provenance.as_deref() == Some("corpus-legacy") {
                assert!(!f.required_ui, "{}:{} corpus-legacy tak boleh required_ui", e.type_full, f.name);
                assert!(f.verified.is_none(), "{}:{} corpus-legacy tak boleh verified", e.type_full, f.name);
                continue;
            }
            let v = f.verified.as_deref().unwrap_or("");
            assert!(
                !v.is_empty(),
                "{}:{} wajib verified(file:line) — 54e butir 2",
                e.type_full,
                f.name
            );
            // tripwire butir 3: required_ui ⇒ default non-null
            if f.required_ui {
                assert!(f.default.is_some(), "{}:{} required_ui tanpa default (54e)", e.type_full, f.name);
            }
        }
    }
    // 46a: komposisi kategori tetap 44 + 6 (jangan digabung, jangan bocor)
    assert_eq!(n_tool, 6, "6 tipe Tool wajib berkategori upstream-tool-generated");
    assert_eq!(entries.len() - n_tool, 44, "44 tipe non-Tool upstream-verified");
}

#[test]
fn r6_tipe_tanpa_skema_apapun_hanya_httprequest() {
    // Setelah A/multi (96) + B (50) = 146 tipe terskema; sisa base di
    // korpus = 1: httpRequest (deferred — dikerjakan paling akhir).
    let schemas = all_schemas();
    let entries = jalur_b_entries();

    let files = corpus_files();
    let mut no_schema: BTreeSet<String> = BTreeSet::new();
    for f in &files {
        let label = f.file_name().unwrap().to_string_lossy().to_string();
        let raw = std::fs::read(f).unwrap_or_else(|e| panic!("read {label}: {e}"));
        let w = parse_workflow_bytes(&raw, &label).unwrap_or_else(|e| panic!("parse {label}: {e}"));
        for n in &w.nodes {
            if !n.type_full.starts_with("n8n-nodes-base.") {
                continue;
            }
            let in_am = schema_for(&schemas, &n.type_full).is_some();
            let in_b = jalur_b_for(&entries, &n.type_full).is_some();
            if !in_am && !in_b {
                no_schema.insert(n.type_full.clone());
            }
        }
    }
    let diharapkan: BTreeSet<String> =
        ["n8n-nodes-base.httpRequest" /* deferred — dikerjakan paling akhir */]
            .iter()
            .map(|s| s.to_string())
            .collect();
    assert_eq!(no_schema, diharapkan, "sisa tak terskema harus persis httpRequest");
}
