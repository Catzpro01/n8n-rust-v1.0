//! Alat bantu end-to-end: menghubungkan rantai Envelope (crate ini) dengan anchor TSA
//! eksternal (W0-ANCHOR-IMPL agent1) supaya penutupan C-01 bisa dibuktikan dengan TSA nyata,
//! bukan dengan anchor tiruan di dalam proses.
//!
//! ```text
//! envelope_e2e emit  <n>                          -> cetak head hex (rantai deterministik)
//! envelope_e2e check <n> <anchor_head_hex|none> <clean|tamper>
//!                                                 -> cetak verdict verifier
//! ```
//! Fixture sengaja deterministik (LCG-free, tanpa dependensi waktu) supaya `emit` dan `check`
//! menghasilkan rantai yang identik di proses berbeda — syarat agar head bisa di-anchor oleh
//! satu proses lalu diverifikasi oleh proses lain.

use envelope::chain::recompute;
use envelope::entry::{EnvelopeEntry, Status, SCHEMA_VERSION_V1};
use envelope::verify::{verify, AnchorRecord, Verdict};

const KEY: [u8; 32] = [0x42u8; 32];
const KID: &str = "kid-2026-09";

fn entry(seq: u64) -> EnvelopeEntry {
    EnvelopeEntry {
        schema_version: SCHEMA_VERSION_V1,
        exec_id: b"exec-e2e".to_vec(),
        seq,
        node_id: format!("node{}", seq % 17).into_bytes(),
        input_digest: [(seq & 0xff) as u8; 32],
        output_digest: [0x22; 32],
        status: Status::Succeeded,
        engine_version: b"0.1.0".to_vec(),
        config_digest: [0x33; 32],
        class_flags: 0,
    }
}

fn fixture(n: u64) -> Vec<EnvelopeEntry> {
    (0..n).map(entry).collect()
}

fn parse_hex32(s: &str) -> Option<[u8; 32]> {
    let s = s.trim().to_lowercase();
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: envelope_e2e emit <n> | check <n> <anchor_head_hex|none> <clean|tamper>");
        std::process::exit(2);
    }
    match args[1].as_str() {
        "emit" => {
            let n: u64 = args[2].parse().unwrap_or_else(|_| {
                eprintln!("n bukan angka");
                std::process::exit(2);
            });
            let entries = fixture(n);
            let (heads, _) = recompute(&entries, &KEY, KID);
            let head = heads.last().copied().unwrap_or([0u8; 32]);
            println!("{}", hex(&head));
        }
        "check" => {
            if args.len() < 5 {
                eprintln!("usage: envelope_e2e check <n> <anchor_head_hex|none> <clean|tamper>");
                std::process::exit(2);
            }
            let n: u64 = args[2].parse().unwrap_or_else(|_| {
                eprintln!("n bukan angka");
                std::process::exit(2);
            });
            let mut entries = fixture(n);
            let (clean_heads, _) = recompute(&entries, &KEY, KID);

            let mode = args[4].as_str();
            if mode == "tamper" {
                // serangan G-C1b: ubah satu entri lalu REKOMPUTASI seluruh rantai
                let idx = (n / 2) as usize;
                entries[idx].status = Status::FailedError;
                entries[idx].output_digest = [0x99; 32];
            } else if mode != "clean" {
                eprintln!("mode harus 'clean' atau 'tamper'");
                std::process::exit(2);
            }
            let (heads, _) = recompute(&entries, &KEY, KID);

            let anchors: Vec<AnchorRecord> = if args[3].eq_ignore_ascii_case("none") {
                Vec::new()
            } else {
                match parse_hex32(&args[3]) {
                    Some(h) => vec![AnchorRecord { anchor_id: 1, covers_n: n, head: h }],
                    None => {
                        eprintln!("anchor_head_hex harus 64 hex atau 'none'");
                        std::process::exit(2);
                    }
                }
            };

            let r = verify(&entries, &heads, &KEY, KID, &[KID.to_string()], &anchors);
            println!("verdict={}", r.verdict.as_str());
            println!("n={} anchored_through={} first_deviation={:?}", r.n, r.anchored_through, r.first_deviation);
            println!("head_now={}", hex(&r.head));
            println!("head_clean={}", hex(clean_heads.last().unwrap_or(&[0u8; 32])));
            if let Some(reason) = &r.reason {
                println!("reason={reason}");
            }
            // exit code: 0 = sesuai harapan pemanggil (diperiksa skrip), 3 = BROKEN
            if r.verdict == Verdict::Broken {
                std::process::exit(3);
            }
        }
        other => {
            eprintln!("subcommand tak dikenal: {other}");
            std::process::exit(2);
        }
    }
}

fn hex(b: &[u8; 32]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
