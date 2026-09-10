//! INTEG-01 (golden diff): regenerasi dari spec HARUS byte-identical
//! dengan artefak ter-commit (CI rebuild + diff = 0).

use openapi_codegen::emit::emit;
use openapi_codegen::ingest::ingest;
use openapi_codegen::translate::translate_collect;

fn spec(name: &str) -> String {
    format!("{}/../specs/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn integ01_frankfurter_golden_diff() {
    let doc = ingest(&spec("frankfurter.openapi.json")).unwrap();
    let rep = translate_collect(&doc).unwrap();
    let art = emit(&doc, &rep, "0.1.0").unwrap();
    assert_eq!(
        art.nodes_rs,
        include_str!("../src/generated/frankfurter.rs"),
        "golden drift: regenerate dgn `openapi-codegen --spec specs/frankfurter.openapi.json --emit nodes-openapi/src/generated`"
    );
}

#[test]
fn integ01_github_golden_diff() {
    // Spec 12,9MB: parse butuh stack > default test-thread (2MiB) — temuan M3,
    // relevan juga utk runtime engine (catat di laporan). Jalankan di thread
    // ber-stack eksplisit agar tidak bergantung RUST_MIN_STACK.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            let doc = ingest(&spec("github-rest.openapi.json")).unwrap();
            let rep = translate_collect(&doc).unwrap();
            let art = emit(&doc, &rep, "0.1.0").unwrap();
            assert_eq!(
                art.nodes_rs,
                include_str!("../src/generated/github_rest.rs"),
                "golden drift: regenerate dgn `openapi-codegen --spec specs/github-rest.openapi.json --emit nodes-openapi/src/generated`"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn registry_contains_all_generated() {
    let r = nodes_openapi::registry();
    assert_eq!(r.len(), 1211); // 5 frankfurter + 1206 github
    assert!(r.iter().any(|d| d.kind.as_str() == "generated.frankfurter.getcurrencies"));
    assert!(r.iter().any(|d| d.kind.as_str() == "generated.github-rest.actions.add-custom-labels-to-self-hosted-runner-for-org"));
    // side effect ter-set benar lewat ResourceHint
    let post = r.iter().find(|d| d.kind.as_str().ends_with("add-custom-labels-to-self-hosted-runner-for-org")).unwrap();
    assert_eq!(post.hints.side_effect, kernel::node::SideEffect::NonIdempotent);
    let get = r.iter().find(|d| d.kind.as_str() == "generated.frankfurter.getcurrencies").unwrap();
    assert_eq!(get.hints.side_effect, kernel::node::SideEffect::Idempotent);
}
