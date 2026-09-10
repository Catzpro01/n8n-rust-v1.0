// TB-02 langkah-1: RED test jujur — fixture ekspor-nyata agent9 vs SetNode apa adanya.
// WF-A: Manual Trigger -> Set kanonik V2 (assignments, include:none) price=21.
// Merah sekarang: SetNode hanya baca fields.values[] (SET_BAD_PARAMS, stdout kosong).
// Jalan via seam bin tb01 yang sudah ada; CLI n8n-rust TB-02 menggantikan nanti.
use std::process::Command;

const WF_A: &str = "/opt/agent-workspace/docs/FIXTURE-TB/workflow-a-manual-to-set.json";

#[test]
fn tb02_wfa_canonical_set_emits_price() {
    let bin = env!("CARGO_BIN_EXE_tb01");
    let out = Command::new(bin)
        .arg(WF_A)
        .output()
        .expect("spawn tb01 on WF-A");
    assert!(
        out.status.success(),
        "tb01 exit non-0 on WF-A: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let val: serde_json::Value =
        serde_json::from_str(&stdout).expect("WF-A stdout must be JSON");
    let items = val.as_array().expect("WF-A stdout must be JSON array");
    assert_eq!(items.len(), 1, "WF-A must emit exactly 1 item");
    assert_eq!(
        items[0].get("json").and_then(|j| j.get("price")),
        Some(&serde_json::Value::from(21)),
        "WF-A item json.price must be 21"
    );
}
