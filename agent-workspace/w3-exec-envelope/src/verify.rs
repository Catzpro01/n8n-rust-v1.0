//! Verifier (spec §5) + postur klaim jujur (spec §1 prinsip 1, addendum A1.3/A1.4).
//!
//! Tiga kemungkinan hasil, dan **tidak ada "Verified" polos**:
//! - `VerifiedAnchored` : konsisten internal DAN seluruh entri dijangkau anchor yang cocok.
//! - `VerifiedUnanchored` : konsisten internal, tetapi ada entri di luar jangkauan anchor
//!   terakhir — "jendela jujur" (A.3); verifier DILARANG menyebutnya terverifikasi penuh.
//! - `Broken` : penyimpangan terdeteksi, dengan indeks pertama yang menyimpang.
//!
//! Inti addendum A1: konsistensi internal SAJA tidak menutup C-01. Penyerang yang mengubah
//! entri lalu merekomputasi rantai+root lolos langkah 1–4. Yang menangkapnya adalah langkah 5
//! (anchor), karena root yang sudah dipublikasikan ke TSA tidak bisa ditulis ulang root lokal.

use crate::chain::recompute;
use crate::entry::EnvelopeEntry;

/// Satu rekaman anchor eksternal (baris `anchor_log` di W0-ANCHOR-SPEC §9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorRecord {
    pub anchor_id: u64,
    /// jumlah entri yang dijangkau anchor ini (1-based count)
    pub covers_n: u64,
    /// `chain_head` yang dipublikasikan ke TSA
    pub head: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    VerifiedAnchored,
    VerifiedUnanchored,
    Broken,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::VerifiedAnchored => "VERIFIED_ANCHORED",
            Verdict::VerifiedUnanchored => "VERIFIED_UNANCHORED",
            Verdict::Broken => "BROKEN",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyReport {
    pub verdict: Verdict,
    /// indeks (0-based) entri pertama yang menyimpang, bila ada
    pub first_deviation: Option<u64>,
    pub head: [u8; 32],
    pub n: u64,
    /// jumlah entri yang dijangkau anchor yang cocok
    pub anchored_through: u64,
    pub reason: Option<String>,
}

impl VerifyReport {
    fn broken(n: u64, head: [u8; 32], dev: Option<u64>, reason: &str) -> VerifyReport {
        VerifyReport {
            verdict: Verdict::Broken,
            first_deviation: dev,
            head,
            n,
            anchored_through: 0,
            reason: Some(reason.to_string()),
        }
    }
}

/// Verifikasi penuh.
///
/// * `stored_heads` — `h_i` yang tersimpan di append-only file (spec §7). Panjangnya WAJIB
///   sama dengan `entries`; selisih panjang = penghapusan/penambahan (G-C1a).
/// * `known_kids` — `chain_key_id` yang dikenal registri. Segmen dengan kid tak dikenal
///   ditolak (spec §6, A1.5b).
#[allow(clippy::too_many_arguments)]
pub fn verify(
    entries: &[EnvelopeEntry],
    stored_heads: &[[u8; 32]],
    chain_key: &[u8; 32],
    chain_key_id: &str,
    known_kids: &[String],
    anchors: &[AnchorRecord],
) -> VerifyReport {
    let n = entries.len() as u64;

    // 0. kid tak dikenal -> tolak (A1.5b)
    if !known_kids.iter().any(|k| k == chain_key_id) {
        return VerifyReport::broken(n, [0u8; 32], None, "unknown chain_key_id (A1.5b)");
    }

    // 1. jumlah record harus cocok -> menangkap penghapusan/penyisipan tanpa rekomputasi
    if stored_heads.len() as u64 != n {
        return VerifyReport::broken(
            n,
            *stored_heads.last().unwrap_or(&[0u8; 32]),
            Some(stored_heads.len().min(entries.len()) as u64),
            "record count != entry count (deletion/insertion)",
        );
    }

    // 2. seq harus ketat naik -> menangkap pengurutan ulang / penyisipan di tengah (C-03)
    for i in 1..entries.len() {
        if entries[i].seq <= entries[i - 1].seq {
            return VerifyReport::broken(
                n,
                stored_heads.last().copied().unwrap_or([0u8; 32]),
                Some(i as u64),
                "seq not strictly increasing (reorder/insertion)",
            );
        }
    }

    // 3. hitung ulang rantai, bandingkan head per entri
    let (heads, _checkpoints) = recompute(entries, chain_key, chain_key_id);
    for i in 0..heads.len() {
        if heads[i] != stored_heads[i] {
            return VerifyReport::broken(
                n,
                *heads.last().unwrap_or(&[0u8; 32]),
                Some(i as u64),
                "recomputed head != stored head",
            );
        }
    }

    let head = heads.last().copied().unwrap_or([0u8; 32]);

    // 4. rantai kosong: tidak ada yang bisa diklaim
    if n == 0 {
        return VerifyReport {
            verdict: Verdict::VerifiedUnanchored,
            first_deviation: None,
            head,
            n: 0,
            anchored_through: 0,
            reason: Some("empty chain: nothing to verify".to_string()),
        };
    }

    // 5. ANCHOR — lapisan yang MENJAMIN (A.2). Anchor yang tidak cocok = pemalsuan
    //    yang lolos langkah 1-4 (kasus G-C1b: ubah + rekomputasi).
    let mut anchored_through: u64 = 0;
    for a in anchors {
        if a.covers_n == 0 || a.covers_n > n {
            continue; // anchor di luar rentang: abaikan, jangan klaim
        }
        let expected = heads[(a.covers_n - 1) as usize];
        if expected != a.head {
            return VerifyReport::broken(
                n,
                head,
                Some(a.covers_n - 1),
                "ANCHOR MISMATCH: anchored head != recomputed head (post-hoc rewrite detected)",
            );
        }
        if a.covers_n > anchored_through {
            anchored_through = a.covers_n;
        }
    }

    if anchored_through >= n {
        VerifyReport {
            verdict: Verdict::VerifiedAnchored,
            first_deviation: None,
            head,
            n,
            anchored_through,
            reason: None,
        }
    } else {
        VerifyReport {
            verdict: Verdict::VerifiedUnanchored,
            first_deviation: None,
            head,
            n,
            anchored_through,
            reason: Some(format!(
                "{} of {} entries UNANCHORED (window since last anchor)",
                n - anchored_through,
                n
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{Status, SCHEMA_VERSION_V1};

    fn sample(seq: u64, node: &str) -> EnvelopeEntry {
        EnvelopeEntry {
            schema_version: SCHEMA_VERSION_V1,
            exec_id: b"exec-1".to_vec(),
            seq,
            node_id: node.as_bytes().to_vec(),
            input_digest: [seq as u8; 32],
            output_digest: [0x22; 32],
            status: Status::Succeeded,
            engine_version: b"0.1.0".to_vec(),
            config_digest: [0x33; 32],
            class_flags: 0,
        }
    }

    const KEY: [u8; 32] = [42u8; 32];
    const KID: &str = "kid-1";

    fn kids() -> Vec<String> {
        vec![KID.to_string()]
    }

    fn fixture(n: u64) -> (Vec<EnvelopeEntry>, Vec<[u8; 32]>) {
        let entries: Vec<_> = (0..n).map(|i| sample(i, &format!("node{i}"))).collect();
        let (heads, _) = recompute(&entries, &KEY, KID);
        (entries, heads)
    }

    #[test]
    fn honest_baseline_unanchored() {
        let (e, h) = fixture(5);
        let r = verify(&e, &h, &KEY, KID, &kids(), &[]);
        assert_eq!(r.verdict, Verdict::VerifiedUnanchored);
        assert_eq!(r.anchored_through, 0);
    }

    #[test]
    fn anchored_full_coverage() {
        let (e, h) = fixture(5);
        let a = vec![AnchorRecord { anchor_id: 1, covers_n: 5, head: h[4] }];
        let r = verify(&e, &h, &KEY, KID, &kids(), &a);
        assert_eq!(r.verdict, Verdict::VerifiedAnchored);
        assert_eq!(r.anchored_through, 5);
    }

    #[test]
    fn partial_anchor_is_unanchored_window() {
        let (e, h) = fixture(5);
        let a = vec![AnchorRecord { anchor_id: 1, covers_n: 3, head: h[2] }];
        let r = verify(&e, &h, &KEY, KID, &kids(), &a);
        assert_eq!(r.verdict, Verdict::VerifiedUnanchored);
        assert_eq!(r.anchored_through, 3);
        assert!(r.reason.unwrap().contains("2 of 5 entries UNANCHORED"));
    }

    /// G-C1a: hapus entri tengah tanpa merekomputasi.
    #[test]
    fn g_c1a_deletion_detected() {
        let (e, h) = fixture(5);
        let mut e2 = e.clone();
        e2.remove(2);
        // penyerang menyimpan head apa adanya (5 head untuk 4 entri)
        let r = verify(&e2, &h, &KEY, KID, &kids(), &[]);
        assert_eq!(r.verdict, Verdict::Broken);
        assert!(r.reason.unwrap().contains("record count"));

        // varian: penyerang juga membuang head ke-3 supaya panjangnya cocok
        let mut h2 = h.clone();
        h2.remove(2);
        let r2 = verify(&e2, &h2, &KEY, KID, &kids(), &[]);
        assert_eq!(r2.verdict, Verdict::Broken);
        assert_eq!(r2.first_deviation, Some(2));
    }

    /// G-C1b: ubah entri + REKOMPUTASI seluruh rantai.
    /// Tanpa anchor -> LOLOS (inilah batas jujur keyed chain, addendum A1).
    /// Dengan anchor -> BROKEN (anchor yang menangkapnya).
    #[test]
    fn g_c1b_recompute_passes_without_anchor_and_fails_with_anchor() {
        let (e, _h) = fixture(5);
        let mut evil = e.clone();
        evil[2].status = Status::FailedError; // ubah isi
        evil[2].output_digest = [0x99; 32];
        // penyerang merekomputasi rantai + head
        let (evil_heads, _) = recompute(&evil, &KEY, KID);

        // (a) tanpa anchor: konsistensi internal TERPUASKAN -> cacat lolos
        let r = verify(&evil, &evil_heads, &KEY, KID, &kids(), &[]);
        assert_ne!(r.verdict, Verdict::Broken, "A1: tanpa anchor, pemalsuan memang lolos");
        assert_eq!(r.verdict, Verdict::VerifiedUnanchored);

        // (b) dengan anchor yang dipublikasikan SEBELUM pemalsuan: TERDETEKSI
        let orig_anchor = vec![AnchorRecord { anchor_id: 1, covers_n: 5, head: _h[4] }];
        let r2 = verify(&evil, &evil_heads, &KEY, KID, &kids(), &orig_anchor);
        assert_eq!(r2.verdict, Verdict::Broken);
        assert!(r2.reason.unwrap().contains("ANCHOR MISMATCH"));
    }

    /// Urutan diubah (penyisipan/pengurutan ulang di tengah) -> BROKEN via seq.
    #[test]
    fn reorder_detected() {
        let (mut e, h) = fixture(5);
        e.swap(1, 3);
        let r = verify(&e, &h, &KEY, KID, &kids(), &[]);
        assert_eq!(r.verdict, Verdict::Broken);
    }

    #[test]
    fn unknown_kid_rejected() {
        let (e, h) = fixture(3);
        let r = verify(&e, &h, &KEY, "kid-unknown", &kids(), &[]);
        assert_eq!(r.verdict, Verdict::Broken);
        assert!(r.reason.unwrap().contains("unknown chain_key_id"));
    }

    #[test]
    fn wrong_key_rejected() {
        let (e, h) = fixture(3);
        let r = verify(&e, &h, &[43u8; 32], KID, &kids(), &[]);
        assert_eq!(r.verdict, Verdict::Broken);
        assert_eq!(r.first_deviation, Some(0));
    }

    #[test]
    fn empty_chain_claims_nothing() {
        let r = verify(&[], &[], &KEY, KID, &kids(), &[]);
        assert_eq!(r.verdict, Verdict::VerifiedUnanchored);
        assert_eq!(r.n, 0);
    }

    #[test]
    fn anchor_beyond_n_ignored() {
        let (e, h) = fixture(3);
        let a = vec![AnchorRecord { anchor_id: 1, covers_n: 99, head: h[2] }];
        let r = verify(&e, &h, &KEY, KID, &kids(), &a);
        assert_eq!(r.verdict, Verdict::VerifiedUnanchored, "anchor di luar rentang tidak boleh diklaim");
    }
}
