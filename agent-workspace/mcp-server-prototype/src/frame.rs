//! Pembingkaian stdio: newline-delimited JSON-RPC (MCP 2025 family & 2026 stdio).
//! stdout = protokol murni; stderr = log (dipisah di main.rs).
//! Batas ukuran frame: MAX_FRAME_BYTES (1 MiB) — FR-1 / tabel batas transport.

pub const MAX_FRAME_BYTES: usize = 1024 * 1024;
pub const MAX_TOKEN_PER_PACKAGE: usize = 500; // paket konten resource ≤500 token (TOOLS-PLAN §1)

pub fn validate_frame(line: &str) -> Result<(), String> {
    if line.len() > MAX_FRAME_BYTES {
        return Err(format!("frame {} B melebihi batas {}", line.len(), MAX_FRAME_BYTES));
    }
    if line.contains('\n') || line.contains('\r') {
        return Err("newline embedded dalam frame".into());
    }
    Ok(())
}

/// Estimasi token sederhana (heuristik 4 char/token) — H-3b: est_token = metadata TANPA fetch.
pub fn est_tokens(s: &str) -> usize {
    let visible: usize = s.chars().map(|c| if c.is_whitespace() { 0 } else { 1 }).sum();
    (visible / 4).max(1)
}

/// Potong paket ke ≤ max_token; bila terpotong, pasang flag truncated (kontrak L4 & TOOLS-PLAN §4).
pub fn clip_package(payload: &serde_json::Value, max_token: usize) -> (serde_json::Value, bool) {
    let full = payload.to_string();
    if est_tokens(&full) <= max_token {
        return (payload.clone(), false);
    }
    let mut s = full;
    while est_tokens(&s) > max_token && s.len() > 16 {
        s.truncate(s.len() - 16);
        // potong di perbatasan karakter utuh
        while !s.is_char_boundary(s.len()) { s.truncate(s.len() - 1); }
    }
    s.push_str("...\"}"); // tutup JSON tidak dijamin valid — dipakai hanya utk ringkasan
    (serde_json::Value::String(s), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-01: frame normal
    #[test]
    fn tc_frame_ok() {
        assert!(validate_frame(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#).is_ok());
    }
    // TC-02: frame melampaui batas → tolak
    #[test]
    fn tc_frame_too_big() {
        let big = "x".repeat(MAX_FRAME_BYTES + 2);
        assert!(validate_frame(&big).is_err());
    }
    // TC-03: newline embedded → tolak
    #[test]
    fn tc_frame_embedded_newline() {
        assert!(validate_frame("a\nb").is_err());
    }
    // TC-05: est_token tanpa fetch (H-3b) — murni metadata
    #[test]
    fn tc_est_tokens() {
        assert!(est_tokens("a b c d e f g h") >= 1);
        assert_eq!(est_tokens(""), 1);
    }
    // TC-04: paket besar → truncated flag
    #[test]
    fn tc_clip_package() {
        let big = serde_json::json!({"data": "x".repeat(10_000)});
        let (_p, t) = clip_package(&big, 50);
        assert!(t);
        let small = serde_json::json!({"a": 1});
        let (_p, t) = clip_package(&small, 500);
        assert!(!t);
    }
}
