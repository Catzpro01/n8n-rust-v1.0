//! Gate M5-inti atas korpus 171: kanon deterministik (digest stabil),
//! N-01 (root allowlist), N-02 (tanpa id node), dan digest asli ≠ digest
//! migrasi tepat pada file yang mengandung function/cron.

use std::path::{Path, PathBuf};

use rosetta::kanon::canonical_digest;
use rosetta::migrate::migrate_workflow;
use rosetta::parse::parse_workflow_path;

fn corpus_files() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> =
        std::fs::read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"))
            .expect("tests/fixtures ada")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
            .collect();
    v.sort();
    v
}

fn make_migrated_digest(f: &Path) -> Option<([u8; 32], usize)> {
    let wf = parse_workflow_path(f).unwrap();
    let (out, report) = migrate_workflow(&wf);
    if report.migrated_count() == 0 {
        return None;
    }
    let wf2 = rosetta::model::Workflow {
        name: wf.name.clone(),
        nodes: out,
        connections: wf.connections.clone(),
        extra: wf.extra.clone(),
        opaque_nodes: wf.opaque_nodes.clone(),
    };
    Some((canonical_digest(&wf2).unwrap(), report.migrated_count()))
}

#[test]
fn m5_kanon_deterministik_171() {
    let mut digests = Vec::new();
    for f in corpus_files() {
        let wf = parse_workflow_path(&f).unwrap();
        let a = canonical_digest(&wf).unwrap();
        let b = canonical_digest(&wf).unwrap();
        assert_eq!(a, b, "digest tidak stabil: {}", f.display());
        digests.push((f, a));
    }
    // semua digest unik (file beda → konten beda; probabilitas tabrakan ~0)
    let mut seen = std::collections::HashSet::new();
    for (_f, d) in &digests {
        assert!(seen.insert(*d), "duplikat digest?");
    }
    eprintln!("M5: {} file kanon stabil & unik", digests.len());
}

#[test]
fn m5_kanon_tanpa_id_dan_root_bersih() {
    for f in corpus_files() {
        let wf = parse_workflow_path(&f).unwrap();
        let bytes = rosetta::kanon::canonical_json_bytes(&wf).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let obj = v.as_object().unwrap();
        // N-01: root hanya allowlist + nodes/connections
        for k in obj.keys() {
            assert!(
                [
                    "name",
                    "nodes",
                    "connections",
                    "settings",
                    "pinData",
                    "meta"
                ]
                .contains(&k.as_str()),
                "root tak diizinkan {} di {}",
                k,
                f.display()
            );
        }
        // N-02: node nodes[] tak boleh punya kunci id
        let nodes = obj.get("nodes").unwrap().as_array().unwrap();
        for n in nodes {
            assert!(
                !n.as_object().unwrap().contains_key("id"),
                "node id tersisa: {}",
                f.display()
            );
        }
    }
}

#[test]
fn m5_digest_asli_vs_migrasi() {
    let mut changed = 0usize;
    let mut unchanged_but_migrated = 0usize;
    for f in corpus_files() {
        let wf = parse_workflow_path(&f).unwrap();
        let d_orig = canonical_digest(&wf).unwrap();
        if let Some((d_mig, _n)) = make_migrated_digest(&f) {
            if d_orig != d_mig {
                changed += 1;
            } else {
                unchanged_but_migrated += 1;
            }
        }
    }
    // 13 file cron + 12 file function = 20 file unik termigrasi (overlap
    // tpl-175/199/225/378/693 memuat keduanya) — digest harus berubah di
    // SEMUA file yang termigrasi
    assert_eq!(
        changed, 20,
        "ke-20 file termigrasi harus berubah digest (aktual {changed})"
    );
    assert_eq!(
        unchanged_but_migrated, 0,
        "tidak boleh ada migrasi tanpa jejak digest"
    );
    eprintln!("M5 diffgate-L1: {changed} file digest berubah setelah migrasi");
}
