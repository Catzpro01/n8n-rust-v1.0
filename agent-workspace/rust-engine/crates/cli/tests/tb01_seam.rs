// TB-01 seam test (Seam 1: CLI process). TDD slice 1.
// Verifies: run tb01 binary against 2-node fixture -> exit 0, stdout is a valid
// JSON array with exactly one item whose json.greeting equals hello.
// RED while the tb01 binary target does not exist.
use std::process::Command;

#[test]
fn tb01_two_node_workflow_emits_valid_json() {
    let bin = env!("CARGO_BIN_EXE_tb01");
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tb01-two-node.json"
    );
    let out = Command::new(bin)
        .arg(fixture)
        .output()
        .expect("spawn tb01 binary");
    assert!(
        out.status.success(),
        "tb01 exit non-zero: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("stdout is valid JSON");
    let arr = v.as_array().expect("stdout is JSON array");
    assert_eq!(arr.len(), 1, "expected exactly one output item");
    assert_eq!(arr[0]["json"]["greeting"], serde_json::json!("hello"));
}
