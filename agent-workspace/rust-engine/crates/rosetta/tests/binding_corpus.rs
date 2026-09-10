//! Gate M4 (R-3) atas korpus 171: note-binding stickyNote deterministik
//! (2× hitung = identik); coverage tercatat per template.

use std::path::PathBuf;

use rosetta::binding::{bind_notes, note_rect, STICKY_TYPE};
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

#[test]
fn r3_determinisme_binding_korpus() {
    for f in corpus_files() {
        let wf = parse_workflow_path(&f).unwrap();
        let a = bind_notes(&wf.nodes);
        let b = bind_notes(&wf.nodes);
        assert_eq!(a, b, "binding beda pada 2× hitung: {}", f.display());
        // invariant: covered & ambiguous tidak tumpang tindih per node
        for bind in &a.bindings {
            for c in &bind.covered {
                assert!(
                    !bind.ambiguous.contains(c),
                    "node {c} ada di covered & ambiguous sekaligus ({})",
                    f.display()
                );
            }
        }
    }
}

#[test]
fn r3_statistik_binding_korpus() {
    let mut sticky = 0usize;
    let mut with_dim = 0usize;
    let mut unmeasurable = 0usize;
    let mut covered_total = 0usize;
    let mut ambiguous_total = 0usize;
    let mut files = 0usize;

    for f in corpus_files() {
        files += 1;
        let wf = parse_workflow_path(&f).unwrap();
        let mut f_sticky = 0;
        let mut f_with_dim = 0;
        for n in &wf.nodes {
            if n.type_full == STICKY_TYPE {
                f_sticky += 1;
                if note_rect(n).is_some() {
                    f_with_dim += 1;
                }
            }
        }
        let r = bind_notes(&wf.nodes);
        sticky += f_sticky;
        with_dim += f_with_dim;
        unmeasurable += r.unmeasurable.len();
        covered_total += r.covered_node_total;
        ambiguous_total += r.ambiguous_node_total;
    }

    assert_eq!(files, 171);
    // angka baseline korpus saat ini (scan 12:0x)
    assert_eq!(sticky, 450, "450 stickyNote di korpus");
    assert_eq!(with_dim, 388, "388 note dgn width+height");
    assert_eq!(unmeasurable, 62, "62 tanpa dimensi lengkap");
    assert!(
        covered_total > 0,
        "harus ada node kerja ter-cover (coverage nyata)"
    );

    eprintln!(
        "M4 korpus: {sticky} stickyNote ({with_dim} terukur, {unmeasurable} unmeasurable); \
         {covered_total} node ter-cover; {ambiguous_total} node ambigu"
    );
}
