//! Known-answer test (spec §11) + uji domain-separation (C-02) — gate G-C6.
//!
//! Vektor resmi:
//! ```text
//! BLAKE3("") = af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
//! SHA256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
//! ```

use crate::chain::CTX;
use sha2::{Digest, Sha256};

pub const BLAKE3_EMPTY: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
pub const SHA256_EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

fn framing(len: usize, b: &[u8]) -> Vec<u8> {
    let mut v = (len as u32).to_be_bytes().to_vec();
    v.extend_from_slice(b);
    v
}

/// Jalankan seluruh KAT. Tiap baris = (nama, lulus?).
pub fn run_kat() -> Vec<(&'static str, bool)> {
    let mut out = Vec::new();

    // G-C6: vektor resmi
    out.push((
        "G-C6 BLAKE3(\"\") == vektor resmi",
        blake3::hash(b"").to_hex().to_string() == BLAKE3_EMPTY,
    ));
    let mut h = Sha256::new();
    h.update(b"");
    out.push((
        "G-C6 SHA256(\"\") == vektor resmi",
        format!("{:x}", h.finalize()) == SHA256_EMPTY,
    ));

    // C-02: domain separation — keyed_hash(ckey, x) != hash(x)
    let chain_key = [0xABu8; 32];
    let ckey = blake3::derive_key(CTX, &chain_key);
    out.push((
        "C-02 keyed_hash(ckey,\"abc\") != hash(\"abc\")",
        blake3::keyed_hash(&ckey, b"abc").to_hex() != blake3::hash(b"abc").to_hex(),
    ));

    // C-02: konteks berbeda -> kunci berbeda (domain separation antar subsistem)
    let ckey_other = blake3::derive_key("n8nrust/lineage/v1", &chain_key);
    out.push((
        "C-02 derive_key(envelope) != derive_key(lineage)",
        ckey != ckey_other,
    ));

    // KOREKSI-1: derive_key 2-arg == new_derive_key().update().finalize_xof()
    let mut alt = [0u8; 32];
    let mut xof = blake3::Hasher::new_derive_key(CTX);
    xof.update(&chain_key);
    xof.finalize_xof().fill(&mut alt);
    out.push((
        "KOREKSI-1 derive_key(CTX,material) == new_derive_key path",
        alt == ckey,
    ));

    // Ambiguitas nyata tanpa framing (spec §11): "abc"||"X" == "ab"||"cX"
    let mut a = b"abc".to_vec();
    a.extend_from_slice(b"X");
    let mut b = b"ab".to_vec();
    b.extend_from_slice(b"cX");
    out.push((
        "§11 tanpa framing: BLAKE3(\"abc\"||\"X\") == BLAKE3(\"ab\"||\"cX\")  [ambigu, harus TRUE]",
        blake3::hash(&a).to_hex() == blake3::hash(&b).to_hex(),
    ));
    // Dengan prefix u32 BE: wajib BERBEDA
    let fa = {
        let mut v = framing(3, b"abc");
        v.extend_from_slice(&framing(1, b"X"));
        v
    };
    let fb = {
        let mut v = framing(2, b"ab");
        v.extend_from_slice(&framing(2, b"cX"));
        v
    };
    out.push((
        "§11 dengan framing u32-BE: keduanya BERBEDA  [harus TRUE]",
        blake3::hash(&fa).to_hex() != blake3::hash(&fb).to_hex(),
    ));

    // G-C5 (versi runtime): fungsi yang diklaim SHA-256 benar-benar SHA-256
    let mut h2 = Sha256::new();
    h2.update(b"abc");
    out.push((
        "G-C5 sha256(\"abc\") == vektor resmi",
        format!("{:x}", h2.finalize())
            == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    ));
    out.push((
        "G-C5 blake3(\"abc\") == vektor resmi",
        blake3::hash(b"abc").to_hex().to_string()
            == "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85",
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_kat_pass() {
        for (name, ok) in run_kat() {
            assert!(ok, "KAT GAGAL: {name}");
        }
    }
}
