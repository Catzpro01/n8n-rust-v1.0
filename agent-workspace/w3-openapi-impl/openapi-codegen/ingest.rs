//! INGEST: baca spec dari disk, hitung identitas, parse JSON.
//! v0.1 TANPA HTTP client (Ruling matt #1116): golden spec sudah di disk;
//! eksekusi request kelak via trait kernel HttpClient (context.rs:433).

use crate::error::{codes, CodegenError};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Batas kedalaman nesting JSON dokumen spec (Ruling 39b — selaras MAX_JSON_DEPTH agent6).
/// KEBIJAKAN KONSUMEN codegen (json.rs:13): pemeriksanya = kernel fn murni (Ruling 41),
/// angka batasnya tetap milik pemanggil.
pub const MAX_JSON_DEPTH: u32 = 64;

#[derive(Debug)]
pub struct SpecDoc {
    pub path: String,
    /// SHA-256 atas byte mentah — hash_domain = "raw-bytes" (addendum 1.6, NIT agent4 #1017).
    pub sha256_hex: String,
    pub size_bytes: u64,
    pub root: Value,
}

pub fn ingest(path: &str) -> Result<SpecDoc, CodegenError> {
    let bytes = std::fs::read(path)
        .map_err(|e| CodegenError::new(codes::NOT_OPENAPI, path, format!("baca file gagal: {e}")))?;
    // Ruling 39b/41: cek kedalaman SEBELUM serde parse (parse dokumen bersarang dalam
    // menumpuk frame rekursif serde — memeriksa setelah parse = terlambat).
    // KUTOVER R41: scanner transisional check_depth() DIGANTI fn murni kernel
    // (json_depth_exceeded — iteratif, string-aware, O(1)-stack; kanonik HEAD 5bc8ea8,
    // otorisasi fern #1469/#1474, kuorum 2/2 R49b). Batas TETAP milik codegen
    // sebagai kebijakan konsumen (json.rs:13).
    if let Some(depth) = kernel::json::json_depth_exceeded(&bytes, MAX_JSON_DEPTH) {
        return Err(CodegenError::new(
            codes::JSON_DEPTH_EXCEEDED,
            path,
            format!(
                "kedalaman nesting JSON {depth} melewati batas {MAX_JSON_DEPTH} \
                 (Ruling 39b; fail-closed; Ruling 41: pemeriksa = kernel fn murni)"
            ),
        ));
    }
    let mut h = Sha256::new();
    h.update(&bytes);
    let sha256_hex = hex(&h.finalize());
    let root: Value = serde_json::from_slice(&bytes)
        .map_err(|_| CodegenError::new(codes::NOT_OPENAPI, path, "dokumen bukan JSON valid"))?;
    if !root.is_object() {
        return Err(CodegenError::new(codes::NOT_OPENAPI, path, "root bukan objek JSON"));
    }
    let ver = root.get("openapi").and_then(Value::as_str).unwrap_or("");
    if !(ver.starts_with("3.0.") || ver.starts_with("3.1.")) {
        return Err(CodegenError::new(
            codes::NOT_OPENAPI,
            path,
            format!("versi openapi '{ver}' bukan 3.0.x/3.1.x"),
        ));
    }
    Ok(SpecDoc { path: path.to_string(), sha256_hex, size_bytes: bytes.len() as u64, root })
}

fn hex(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        format!("{}/../tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
    }

    #[test]
    fn reject_not_json() {
        let err = ingest(&fixture("not-json.txt")).unwrap_err();
        assert_eq!(err.code, "CG-E-101");
    }

    #[test]
    fn reject_wrong_version() {
        let err = ingest(&fixture("wrong-version.json")).unwrap_err();
        assert_eq!(err.code, "CG-E-101");
        assert!(err.reason.contains("2.0"));
    }

    #[test]
    fn depth_64_accepted_65_rejected() {
        // Ruling 39b: guard kedalaman terbukti pada batas eksak (64 ok, 65 tolak).
        // Dokumen valid (ada openapi/info) dengan nesting dalam di bawah
        // ekstensi x-schemas: kedalaman total = root(1) + n level.
        let mk = |n: usize| {
            let mut s = String::from(
                "{\"openapi\":\"3.0.0\",\"info\":{\"title\":\"t\",\"version\":\"1\"},\"x-schemas\":",
            );
            for _ in 0..n {
                s.push('{');
                s.push_str("\"k\":");
            }
            s.push('1');
            for _ in 0..n {
                s.push('}');
            }
            s.push('}');
            // FINDING-1 agent10 #1413: path UNIK PER-PROSES — /tmp sticky + berkas
            // 0644 milik proses lain membuat verifier non-owner gagal menimpa
            // (PermissionDenied = 58/59 di pihak lain, bukan bug logika).
            // Tanpa crate tempfile agar deps tetap minimal (serde_json + sha2).
            let path = std::env::temp_dir()
                .join(format!("depth-{n}-{}.json", std::process::id()))
                .to_string_lossy()
                .into_owned();
            std::fs::write(&path, &s).unwrap();
            path
        };
        let p63 = mk(63);
        assert!(ingest(&p63).is_ok(), "kedalaman total 64 harus diterima");
        let _ = std::fs::remove_file(&p63); // best-effort: bersihkan artefak proses ini
        let p64 = mk(64);
        let err = ingest(&p64).unwrap_err();
        let _ = std::fs::remove_file(&p64);
        assert_eq!(err.code, "CG-E-104");
        assert!(err.reason.contains("65"));
    }

    #[test]
    fn ingest_frankfurter_golden() {
        let path = format!("{}/../specs/frankfurter.openapi.json", env!("CARGO_MANIFEST_DIR"));
        let doc = ingest(&path).unwrap();
        assert_eq!(doc.size_bytes, 9777);
        assert_eq!(doc.sha256_hex, "e38804965823bcbd7db06471aa6a2b8b19b4dfc5f26f95d14aae8cfad1fb46a6");
    }
}
