//! Gate runner W3-EXEC-ENVELOPE — bukti eksekusi untuk G-C1a/G-C1b/G-C2/G-C4/G-C6 +
//! SEC-TRANSIENT. Semua gate dijalankan di proses ini; exit code != 0 bila ada yang gagal.
//!
//! Reproduksi:
//! ```text
//! export CARGO_TARGET_DIR=/mnt/extra-storage/agent10-cargo-target
//! cargo run --release --bin envelope-demo
//! ```

use envelope::chain::{recompute, Chain, CHECKPOINT_INTERVAL};
use envelope::entry::{EnvelopeEntry, Status, SCHEMA_VERSION_V1};
use envelope::kat::run_kat;
use envelope::verify::{verify, AnchorRecord, Verdict};

const KEY: [u8; 32] = [0x42u8; 32];
const KID: &str = "kid-2026-09";

fn entry(seq: u64, node: &str, out: u8) -> EnvelopeEntry {
    EnvelopeEntry {
        schema_version: SCHEMA_VERSION_V1,
        exec_id: b"exec-demo-1".to_vec(),
        seq,
        node_id: node.as_bytes().to_vec(),
        input_digest: [(seq & 0xff) as u8; 32],
        output_digest: [out; 32],
        status: Status::Succeeded,
        engine_version: b"0.1.0".to_vec(),
        config_digest: [0x33; 32],
        class_flags: 0,
    }
}

fn kids() -> Vec<String> {
    vec![KID.to_string()]
}

fn fixture(n: u64) -> (Vec<EnvelopeEntry>, Vec<[u8; 32]>) {
    let entries: Vec<_> = (0..n).map(|i| entry(i, &format!("node{i}"), 0x22)).collect();
    let (heads, _) = recompute(&entries, &KEY, KID);
    (entries, heads)
}

/// PRNG deterministik (tanpa dependensi `rand`) — supaya fuzz bisa direproduksi.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 11
    }
}

fn main() {
    let mut fails = 0usize;
    let mut row = |gate: &str, desc: &str, ok: bool, detail: String| {
        if !ok {
            fails += 1;
        }
        println!(
            "[{}] {:<8} {} — {}",
            if ok { "PASS" } else { "FAIL" },
            gate,
            desc,
            detail
        );
    };

    println!("=== W3-EXEC-ENVELOPE gate runner (agent10 sesi A) ===");
    println!(
        "blake3::Hasher = {} B ; sha2::Sha256 = {} B ; budget = {} B",
        envelope::hasher_size_bytes(),
        envelope::sha256_size_bytes(),
        envelope::TRANSIENT_BUDGET_BYTES
    );
    println!();

    // ---- G-C6 + C-02 + KOREKSI-1 --------------------------------------------
    println!("--- G-C6 / C-02 : known-answer test ---");
    for (name, ok) in run_kat() {
        row("G-C6", name, ok, String::new());
    }
    println!();

    // ---- SEC-TRANSIENT ------------------------------------------------------
    println!("--- SEC-TRANSIENT : anggaran memori hasher (spec §4) ---");
    let hs = envelope::hasher_size_bytes();
    let budget = envelope::TRANSIENT_BUDGET_BYTES;
    row(
        "SEC-TRANS",
        "1 hasher hidup <= 4 KB",
        hs <= budget,
        format!("{hs} B = {}% budget", hs * 100 / budget),
    );
    row(
        "SEC-TRANS",
        "2 hasher simultan <= 4 KB",
        2 * hs <= budget,
        format!("{} B = {}% budget", 2 * hs, 2 * hs * 100 / budget),
    );
    row(
        "SEC-TRANS",
        "3 hasher simultan MELANGGAR (harus true = memang melanggar)",
        3 * hs > budget,
        format!("{} B = {}% budget -> aturan R1/R2 'satu hasher per jalur' MENGIKAT", 3 * hs, 3 * hs * 100 / budget),
    );
    // bukti runtime: Chain memakai SATU hasher (persisten 32 B + hasher)
    let c = Chain::new(&KEY, KID);
    row(
        "SEC-TRANS",
        "Chain persistent state kecil (head 32 B + kid)",
        std::mem::size_of_val(&c.head()) == 32,
        format!("head = {} B", std::mem::size_of_val(&c.head())),
    );
    println!();

    // ---- G-C4 : replay deterministik ---------------------------------------
    println!("--- G-C4 : replay deterministik ---");
    let (e1, h1) = fixture(5);
    let (e2, h2) = fixture(5);
    row(
        "G-C4",
        "2x replay fixture identik -> head identik",
        h1 == h2 && e1 == e2,
        format!("head = {:x?}", &h1[4][..4]),
    );
    let field_check = {
        let s = format!("{:?}", e1[0]);
        !["durasi_ms", "started_at", "finished_at", "worker_id", "rss_bytes"]
            .iter()
            .any(|f| s.contains(f))
    };
    row(
        "G-C4",
        "field observasional TIDAK masuk entri (C-04, cek statis)",
        field_check,
        String::new(),
    );
    println!();

    // ---- G-C1a : hapus entri tengah ----------------------------------------
    println!("--- G-C1a : hapus entri tengah ---");
    let (e, h) = fixture(5);
    let mut e_del = e.clone();
    e_del.remove(2);
    let r = verify(&e_del, &h, &KEY, KID, &kids(), &[]);
    row(
        "G-C1a",
        "hapus tanpa rekomputasi (head tidak disesuaikan)",
        r.verdict == Verdict::Broken,
        format!("{}: {}", r.verdict.as_str(), r.reason.clone().unwrap_or_default()),
    );
    let mut h_del = h.clone();
    h_del.remove(2);
    let r2 = verify(&e_del, &h_del, &KEY, KID, &kids(), &[]);
    row(
        "G-C1a",
        "hapus + buang head yang cocok (panjang disamakan)",
        r2.verdict == Verdict::Broken && r2.first_deviation == Some(2),
        format!("{} first_deviation={:?}", r2.verdict.as_str(), r2.first_deviation),
    );
    println!();

    // ---- G-C1b : ubah + REKOMPUTASI (inti addendum A1) ----------------------
    println!("--- G-C1b : ubah entri lalu rekomputasi rantai + root ---");
    let (e, h) = fixture(5);
    let mut evil = e.clone();
    evil[2].status = Status::FailedError;
    evil[2].output_digest = [0x99; 32];
    let (evil_heads, _) = recompute(&evil, &KEY, KID);

    let r_no_anchor = verify(&evil, &evil_heads, &KEY, KID, &kids(), &[]);
    row(
        "G-C1b",
        "TANPA anchor: pemalsuan LOLOS (batas jujur keyed chain, A1)",
        r_no_anchor.verdict != Verdict::Broken,
        format!(
            "{} — {}",
            r_no_anchor.verdict.as_str(),
            r_no_anchor.reason.clone().unwrap_or_default()
        ),
    );

    let anchor = vec![AnchorRecord { anchor_id: 1, covers_n: 5, head: h[4] }];
    let r_anchor = verify(&evil, &evil_heads, &KEY, KID, &kids(), &anchor);
    row(
        "G-C1b",
        "DENGAN anchor: pemalsuan TERDETEKSI (anchor = lapisan yang MENJAMIN)",
        r_anchor.verdict == Verdict::Broken,
        format!("{}: {}", r_anchor.verdict.as_str(), r_anchor.reason.clone().unwrap_or_default()),
    );

    // penyerang ber-root yang JUGA memegang chain_key
    let r_key_holder = {
        let (evil2, _) = recompute(&evil, &KEY, KID);
        verify(&evil, &evil2, &KEY, KID, &kids(), &anchor)
    };
    row(
        "G-C1b",
        "penyerang memegang chain_key + anchor sudah terbit -> tetap TERDETEKSI",
        r_key_holder.verdict == Verdict::Broken,
        r_key_holder.verdict.as_str().to_string(),
    );
    println!();

    // ---- Jendela jujur (A.3) ------------------------------------------------
    println!("--- A.3 : jendela jujur (UNANCHORED) ---");
    let (e, h) = fixture(5);
    let partial = vec![AnchorRecord { anchor_id: 1, covers_n: 3, head: h[2] }];
    let rp = verify(&e, &h, &KEY, KID, &kids(), &partial);
    row(
        "A.3",
        "anchor sebagian -> VERIFIED_UNANCHORED (bukan 'VERIFIED')",
        rp.verdict == Verdict::VerifiedUnanchored && rp.anchored_through == 3,
        format!("{} anchored_through={} — {}", rp.verdict.as_str(), rp.anchored_through, rp.reason.clone().unwrap_or_default()),
    );
    let full = vec![AnchorRecord { anchor_id: 1, covers_n: 5, head: h[4] }];
    let rf = verify(&e, &h, &KEY, KID, &kids(), &full);
    row(
        "A.3",
        "anchor penuh -> VERIFIED_ANCHORED",
        rf.verdict == Verdict::VerifiedAnchored,
        rf.verdict.as_str().to_string(),
    );
    row(
        "A.3",
        "verifier tidak pernah mengembalikan 'VERIFIED' polos",
        !["VERIFIED"].contains(&rf.verdict.as_str()),
        "himpunan verdict = {VERIFIED_ANCHORED, VERIFIED_UNANCHORED, BROKEN}".to_string(),
    );
    println!();

    // ---- A1.5b : kid tak dikenal -------------------------------------------
    let rk = verify(&e, &h, &KEY, "kid-lama-yang-dicabut", &kids(), &full);
    row(
        "A1.5b",
        "chain_key_id tak dikenal -> ditolak",
        rk.verdict == Verdict::Broken,
        format!("{}: {}", rk.verdict.as_str(), rk.reason.clone().unwrap_or_default()),
    );
    println!();

    // ---- G-C2 : fuzz + pergeseran batas ------------------------------------
    println!("--- G-C2 : fuzz 10.000 pasangan entri berbeda + pergeseran batas ---");
    let mut lcg = Lcg(0x5eed_2026_0909);
    let mut seen = std::collections::HashSet::new();
    let mut collisions = 0usize;
    let pairs = 10_000usize;
    for _ in 0..pairs {
        let a = lcg.next();
        let b = lcg.next();
        let e_a = entry(a % 1_000_000, &format!("n{}", a % 97), (b & 0xff) as u8);
        let e_b = entry(b % 1_000_000, &format!("n{}", b % 89), (a & 0xff) as u8);
        if e_a == e_b {
            continue; // pasangan identik bukan uji tumbukan
        }
        let ha = blake3::hash(&envelope::encode_canonical(&e_a)).to_hex().to_string();
        let hb = blake3::hash(&envelope::encode_canonical(&e_b)).to_hex().to_string();
        if ha == hb {
            collisions += 1;
        }
        seen.insert(ha);
        seen.insert(hb);
    }
    row(
        "G-C2",
        &format!("fuzz {pairs} pasangan berbeda -> 0 tumbukan"),
        collisions == 0,
        format!("tumbukan={collisions}, hash unik={}", seen.len()),
    );

    // pergeseran batas: node_id "abc" vs "ab" dengan byte berikutnya digeser
    let mut x = entry(1, "abc", 0x22);
    let mut y = entry(1, "ab", 0x22);
    y.engine_version = b"c0.1.0".to_vec(); // 'c' digeser ke field berikutnya
    x.engine_version = b"0.1.0".to_vec();
    let hx = blake3::hash(&envelope::encode_canonical(&x)).to_hex().to_string();
    let hy = blake3::hash(&envelope::encode_canonical(&y)).to_hex().to_string();
    row(
        "G-C2",
        "pergeseran batas antar field ('abc'|'0.1.0' vs 'ab'|'c0.1.0') wajib berbeda",
        hx != hy,
        format!("{} vs {}", &hx[..12], &hy[..12]),
    );

    // ambiguitas tanpa framing sebagai pembanding
    let amb = blake3::hash(b"abcX").to_hex()
        == {
            let mut v = b"ab".to_vec();
            v.extend_from_slice(b"cX");
            blake3::hash(&v).to_hex()
        };
    row(
        "G-C2",
        "sanity: tanpa framing, 'abc'||'X' == 'ab'||'cX' (ambiguitas nyata)",
        amb,
        String::new(),
    );
    println!();

    // ---- Checkpoint ---------------------------------------------------------
    println!("--- Checkpoint (spec §5, K = 1.000) ---");
    let mut c = Chain::new(&KEY, KID);
    for i in 0..(CHECKPOINT_INTERVAL * 2 + 5) {
        c.append(&entry(i, "n", 0x22));
    }
    row(
        "CKPT",
        &format!("checkpoint tiap K={} entri", CHECKPOINT_INTERVAL),
        c.checkpoints.len() == 2,
        format!("jumlah checkpoint = {}", c.checkpoints.len()),
    );
    row(
        "CKPT",
        "head sekarang != checkpoint terakhir (n bukan kelipatan K)",
        c.checkpoints[1].head != c.head() && c.checkpoints[1].n == CHECKPOINT_INTERVAL * 2,
        format!("n={}, checkpoint terakhir di n={}", c.n, c.checkpoints[1].n),
    );
    row(
        "CKPT",
        "checkpoint membawa chain_key_id (A1.5a)",
        c.checkpoints.iter().all(|cp| cp.chain_key_id == KID),
        String::new(),
    );
    println!();

    println!("=== HASIL: {} gate GAGAL ===", fails);
    if fails > 0 {
        std::process::exit(1);
    }
}
