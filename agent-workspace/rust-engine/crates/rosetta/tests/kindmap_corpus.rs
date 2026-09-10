//! Gate R-6 (slice-1): tabel kindmap PROPOSED thd korpus nyata 171.
//!
//! - SEMUA node base (n8n-nodes-base.*) di korpus harus termuat tabel
//!   (fail-loud: unmapped = kegagalan test, bukan tebakan).
//! - Jumlah tipe base unik teramati == jumlah baris tabel (147).
//! - Node non-base (langchain/komunitas) → Err(NonBase) — kategori opaque
//!   (R-2), TIDAK dipetakan.
//! - version_u16 utk semua typeVersion teramati → Some (guard minor<10;
//!   pengukuran korpus: minor maks 9 — #1199).

use std::collections::BTreeSet;
use std::path::PathBuf;

use rosetta::kindmap::{kind_for_type, version_u16, KindError, BASE_KIND};
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
fn r6_kindmap_corpus_171_base_tercover_lengkap() {
    let files = corpus_files();
    assert_eq!(files.len(), 171, "korpus fixtures harus 171 file JSON");

    let mut base_seen: BTreeSet<String> = BTreeSet::new();
    let mut failures: Vec<String> = Vec::new();
    let mut nonbase_ok = 0usize;

    for f in &files {
        let label = f.file_name().unwrap().to_string_lossy().to_string();
        let raw = std::fs::read(f).unwrap_or_else(|e| panic!("read {label}: {e}"));
        let w = parse_workflow_bytes(&raw, &label).unwrap_or_else(|e| panic!("parse {label}: {e}"));
        for n in &w.nodes {
            if n.type_full.starts_with("n8n-nodes-base.") {
                base_seen.insert(n.type_full.clone());
                if let Err(e) = kind_for_type(&n.type_full) {
                    failures.push(format!("{label}: {} -> {e:?}", n.type_full));
                }
                // semua node base punya typeVersion sah di korpus (scan #1199:
                // 0 missing) — version_u16 wajib Some; kalau ada minor>=10
                // (guard #1176) ini kegagalan TERLIHAT.
                if let Some(tv) = n.type_version {
                    if version_u16(tv).is_none() {
                        failures.push(format!(
                            "{label}: {} version_u16({tv}) -> None (guard minor<10)",
                            n.type_full
                        ));
                    }
                }
            } else {
                // non-base: opaque — kindmap harus menolak eksplisit
                match kind_for_type(&n.type_full) {
                    Err(KindError::NonBase) => nonbase_ok += 1,
                    other => failures.push(format!(
                        "{label}: {} non-base harus NonBase, dapat {other:?}",
                        n.type_full
                    )),
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} kegagalan mapping korpus:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
    assert_eq!(base_seen.len(), 147, "jumlah tipe base unik korpus");
    assert_eq!(base_seen.len(), BASE_KIND.len(), "tabel == korpus (tipe baru di korpus harus masuk tabel eksplisit — fail-loud, jangan tebak)");
    assert!(nonbase_ok > 0, "korpus harus memuat node non-base (opaque)");
}

#[test]
fn r6_tabel_tipe_unik() {
    // prasyarat: tabel tidak memuat tipe duplikat (guard tambahan)
    let mut keys: Vec<&str> = BASE_KIND.iter().map(|(t, _)| *t).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), BASE_KIND.len(), "tabel memuat duplikat tipe");
}
