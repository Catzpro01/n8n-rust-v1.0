//! RULING 50b/50c — guard kedalaman JSON di dua batas API rosetta (fail-closed).
//!
//! parse_workflow_bytes & parse_manifest WAJIB menolak input > MAX_JSON_DEPTH
//! SEBELUM serde_json; batas eksak diuji via pencarian empiris (bukan asumsi
//! cara kernel menghitung), sehingga test tetap benar bila definisi berubah.

use rosetta::error::RosettaError;
use rosetta::manifest_wcb::parse_manifest;
use rosetta::parse::{parse_workflow_bytes, parse_workflow_str, MAX_JSON_DEPTH};

/// Workflow minimal VALID dengan rantai array tersarang sedalam `chain_depth`
/// di dalam parameters.d (node asing → opaque, tetap diterima).
fn wf_bytes(chain_depth: usize) -> Vec<u8> {
    let inner = "[".repeat(chain_depth) + &"]".repeat(chain_depth);
    format!(
        r#"{{"name":"w","nodes":[{{"name":"n","type":"@x/y.z","parameters":{{"d":{inner}}}}}],"connections":{{}}}}"#
    )
    .into_bytes()
}

/// Kedalaman rantai pertama yang DITOLAK kernel utk bentuk wf_bytes (pencarian
/// eksponensial + biner — tidak meng-hardcode definisi kedalaman kernel).
fn first_rejected() -> usize {
    let mut hi = 1usize;
    while parse_workflow_bytes(&wf_bytes(hi), "probe").is_ok() {
        hi *= 2;
        assert!(hi < 1 << 20, "guard tidak pernah menyala — regresi 50b");
    }
    let (mut lo, mut hi) = (0usize, hi);
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if parse_workflow_bytes(&wf_bytes(mid), "probe").is_ok() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    hi
}

#[test]
fn parse_workflow_bytes_menolak_kedalaman_berlebih() {
    let d0 = first_rejected();
    // Sanity: d0 berada di sekitar MAX_JSON_DEPTH (bukan 1 atau 10^6).
    assert!(d0 > MAX_JSON_DEPTH as usize / 2, "d0={d0} terlalu kecil");
    assert!(d0 <= MAX_JSON_DEPTH as usize + 8, "d0={d0} terlalu besar");

    let err = parse_workflow_bytes(&wf_bytes(d0), "t")
        .expect_err("kedalaman berlebih harus ditolak fail-closed");
    match err {
        RosettaError::DepthExceeded { path, found, max } => {
            assert_eq!(path, "t");
            assert_eq!(max, MAX_JSON_DEPTH);
            assert!(found as usize >= d0 - 2, "found {found} aneh utk d0 {d0}");
        }
        other => panic!("harus DepthExceeded, dapat: {other}"),
    }
    // Satu tingkat di bawah batas: tetap parse OK (workflow valid).
    let ok = parse_workflow_bytes(&wf_bytes(d0 - 1), "t");
    assert!(ok.is_ok(), "di bawah batas harus diterima: {ok:?}");
}

#[test]
fn parse_workflow_str_dan_path_mewarisi_guard() {
    // parse_workflow_str mendelegasi ke bytes → guard berlaku juga.
    let deep = String::from_utf8(wf_bytes(200)).unwrap();
    assert!(matches!(
        parse_workflow_str(&deep, "s"),
        Err(RosettaError::DepthExceeded { .. })
    ));
}

#[test]
fn parse_manifest_menolak_kedalaman_berlebih() {
    let deep = format!(r#"{{"a":{},"b":1}}"#, "[".repeat(200) + &"]".repeat(200));
    let err = parse_manifest(&deep).expect_err("manifest dalam harus ditolak");
    assert!(err.contains("kedalaman"), "pesan harus menyebut kedalaman: {err}");
    assert!(err.contains("fail-closed"), "pesan harus menyebut fail-closed: {err}");
}

#[test]
fn parse_manifest_terima_fixture_normal() {
    let fixture = include_str!("manifest_fixtures/manifest-wcb-example.json");
    let m = parse_manifest(fixture).expect("manifest fixture harus valid");
    assert_eq!(m.node_kind, "wcb-guest");
}
