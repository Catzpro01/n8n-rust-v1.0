//! Gate M6: fixture manifest RFC v0.3 valid + type-check terpakai thd
//! parameter node (MV-2 arah Rosetta).

use std::path::PathBuf;

use rosetta::manifest_wcb::{parse_manifest, type_check_params, validate_manifest};

#[test]
fn m6_fixture_manifest_valid() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/manifest_fixtures/manifest-wcb-example.json");
    let s = std::fs::read_to_string(&path).expect("fixture ada");
    let m = parse_manifest(&s).unwrap();
    let errs = validate_manifest(&m);
    assert!(errs.is_empty(), "fixture manifest harus valid: {errs:?}");
    assert_eq!(
        m.n8n_type.as_deref(),
        Some("@blotato/n8n-nodes-blotato.blotato")
    );
}

#[test]
fn m6_type_check_contoh_node() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/manifest_fixtures/manifest-wcb-example.json");
    let s = std::fs::read_to_string(&path).unwrap();
    let m = parse_manifest(&s).unwrap();

    let ok = type_check_params(
        &m,
        &serde_json::json!({"url": "https://example.com", "mode": "css", "selector": ".a", "concurrency": 2}),
    );
    assert!(ok.is_empty(), "{ok:?}");

    let bad = type_check_params(&m, &serde_json::json!({"mode": "css", "concurrency": 9}));
    assert!(bad.iter().any(|i| i.param == "url"), "url wajib: {bad:?}");
    assert!(
        bad.iter()
            .any(|i| i.param == "concurrency" && i.message.contains("max")),
        "concurrency >5: {bad:?}"
    );
}
