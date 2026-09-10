# envelope — W3-EXEC-ENVELOPE (Execution Envelope & rolling hash-chain)

**Status:** v0.1 — implementasi + gate hijau; integrasi storage & penempatan kanonik belum.
**Task queue:** `W3-EXEC-ENVELOPE` (Wave 3, P1, ROLE_COMPLIANCE) — IN_PROGRESS, agent10.
**Spec yang diimplementasikan:** `docs/AGENT10-EXEC-ENVELOPE-SPEC.md` v1.2 (sha `6286c1ef…`) §1–§5, §11
+ `docs/AGENT10-AUDIT-ADDENDUM-A1-CORRECTION.md` (urutan lapisan) + `docs/AGENT10-W0-ANCHOR-SPEC.md` v1.1.

## Atribusi & lane (integritas — spec §13, #628 butir 5)

| Bagian | Penulis |
|---|---|
| spec / konstruksi crypto (`EXEC-ENVELOPE-SPEC.md` v1.2) | agent10 **"sesi kedua"** |
| implementasi crate ini + gate runner + E2E | agent10 **"sesi A"** |

Lane Envelope belum diratifikasi fern. Implementasi ini **mengikuti spec apa adanya** dan tidak
menyunting berkas sesi lain. Bila lane diputus berbeda, crate ini yang dipindahkan/diserahkan.

## Hasil verifikasi (semua dijalankan, bukan diklaim)

```
cargo clippy --all-targets   -> 0 warning
cargo test --release         -> 21 passed; 0 failed
envelope-demo                -> 30 gate PASS, 0 FAIL, exit 0
tests/e2e_anchor.sh          -> 10 gate PASS, 0 FAIL   (TSA NYATA: freetsa.org)
```

### Yang paling penting: C-01 tertutup end-to-end (E2E-3/4/5)
```
head rantai (1000 entri)  = bed7f5021a44acc84ce0e96f7236b23147a5d351d02278cb7e236a669879346a
di-anchor ke FreeTSA      = status=ANCHORED quorum=1-of-1 chain=ENVELOPE-E2E [0..999]
fingerprint CA            = a6379e7cecc05faa3cbf076013d745e327bbbaa38c0b9af22469d4701d18aabc

bersih + anchored         -> VERIFIED_ANCHORED            (anchored_through=1000)
ubah entri + REKOMPUTASI  -> BROKEN, first_deviation=999, "ANCHOR MISMATCH (post-hoc rewrite detected)"
  head setelah pemalsuan  = e817e72c931e3c5cbe0c822aa9097f863919433aba5d9f1a796726dc427993b8
serangan SAMA tanpa anchor-> VERIFIED_UNANCHORED  (LOLOS — dan itu memang yang diharapkan)
offline re-verify         -> ANCHORED (re-verified) dari artefak saja
```
E2E-4 dan E2E-5 **harus lulus bersamaan**. E2E-4 membuktikan anchor menutup C-01; E2E-5 membuktikan
bahwa tanpa anchor pemalsuan lolos — yaitu isi addendum A1: *keyed chain perlu, anchor yang menjamin.*
Menyembunyikan E2E-5 akan membuat klaim "tamper-proof" terlihat benar.

### Gate lain (envelope-demo)
| Gate | Hasil |
|---|---|
| G-C6 KAT BLAKE3("")/SHA256("") + vektor "abc" | PASS (4 vektor) |
| C-02 domain separation: keyed vs unkeyed, envelope vs lineage context | PASS |
| KOREKSI-1 `derive_key(CTX, material)` == jalur `new_derive_key` | PASS |
| §11 ambiguitas `"abc"‖"X"` == `"ab"‖"cX"` tanpa framing; berbeda dengan framing u32-BE | PASS |
| SEC-TRANSIENT 1 hasher = 1.920 B (46%), 2 = 3.840 B (93%), 3 = 5.760 B (140% → melanggar) | PASS |
| G-C4 replay deterministik → head identik; field observasional tidak masuk entri (C-04) | PASS |
| G-C1a hapus entri tengah (2 varian) | PASS → BROKEN, `first_deviation=2` |
| G-C2 fuzz 10.000 pasangan → 0 tumbukan; pergeseran batas antar field berbeda | PASS |
| A1.5b `chain_key_id` tak dikenal → ditolak | PASS |
| A.3 jendela jujur: anchor sebagian → `VERIFIED_UNANCHORED`, tidak pernah "VERIFIED" polos | PASS |
| Checkpoint tiap K=1.000, membawa `chain_key_id` | PASS |

## Build & reproduksi

```bash
export CARGO_HOME=/mnt/extra-storage/agent10-cargo-home
export CARGO_TARGET_DIR=/mnt/extra-storage/agent10-cargo-target
cd /opt/agent-workspace/w3-exec-envelope
cargo clippy --all-targets && cargo test --release && cargo build --release
$CARGO_TARGET_DIR/release/envelope-demo                       # 30 gate
bash tests/e2e_anchor.sh \
  $CARGO_TARGET_DIR/release/anchor \
  $CARGO_TARGET_DIR/release/envelope_e2e                      # 10 gate E2E (butuh jaringan ke TSA)
```
Dependensi: `blake3` 1.8, `sha2` 0.10. E2E memakai binary `anchor` (W0-ANCHOR-IMPL, agent1),
`openssl`, `curl`, `sqlite3`.

## Tata letak

| Berkas | Isi |
|---|---|
| `src/entry.rs` | `EnvelopeEntry_v1` + **satu** jalur encoding kanonik (framing u32-BE) + decoder |
| `src/chain.rs` | `CTX="n8nrust/envelope/v1"`, `derive_key` 2-arg, genesis ber-flag `0x00‖0x00`, checkpoint K=1.000 |
| `src/verify.rs` | verifier 3-verdict + `AnchorRecord`; menolak kid tak dikenal, seq tak naik, anchor tak cocok |
| `src/kat.rs` | KAT vektor resmi + uji domain-separation + ambiguitas framing |
| `src/main.rs` | gate runner (30 gate) |
| `src/bin/envelope_e2e.rs` | `emit`/`check` untuk dihubungkan ke anchor TSA nyata |
| `tests/e2e_anchor.sh` | E2E dengan TSA live (10 gate) |

## Keputusan desain yang dipegang

1. **Nol "VERIFIED" polos.** Hanya `VERIFIED_ANCHORED | VERIFIED_UNANCHORED | BROKEN` (+ indeks penyimpangan pertama).
2. **Satu hasher dipakai ulang** (`reset()`), bukan hasher baru per entri — 3 hasher simultan = 140% budget 4 KB.
3. **State persisten kecil:** `h_prev` 32 B + `ckey` 32 B + checkpoint 32 B per 1.000 entri. `heads` tidak ditahan di memori eksekusi (tugas storage §7); verifier offline boleh O(n) karena ia alat audit.
4. **Field observasional tidak ada di struct** (`durasi_ms`, `started_at`, `finished_at`, `worker_id`, `rss_bytes`) → C-04, dan ada test yang menegakkannya.
5. **`chain_key_id` di checkpoint** → rotasi kunci = pemutusan rantai yang tercatat (A1.5a); segmen dengan kid tak dikenal ditolak (A1.5b).
6. **Anchor di luar crate.** Crate ini tidak memanggil jaringan; ia menerima `AnchorRecord`. Pemisahan ini yang membuat E2E bisa memakai implementasi anchor pihak lain (agent1) tanpa kopling.

## Yang BELUM dikerjakan (jujur)

| Item | Penghalang |
|---|---|
| Integrasi storage (§7: append-only `envelope-{exec_id}.env`, tabel `envelope_checkpoint`, `execution_metrics`, `keyring_meta`) | DDL milik **agent2**; `W2-STORAGE-L0` baru DONE — perlu kontrak |
| Penegakan gate di CI + penomoran `SEC-*` + SEC-08 ditulis ulang | milik **agent5** (P3-03: gate = agent5) |
| Penempatan di pohon kanonik (`kernel-asli-d3bcff0/crates/envelope/`) + commit git | **C-5 belum diputus** matt/fern; crate ini sengaja standalone seperti `w0-anchor-impl` |
| Sumber `chain_key` dari secret-manager (D-4) | **K-4 belum dijawab pemilik**; saat ini kunci hanya parameter. Tanpa D-4, A.1 lemah — A.2 (anchor) yang menahan |
| `lineage_root` + `repr` masuk digest (33 B/entri) | menunggu **D-L1** dan skema `schema_version=2`; v1 = spec §2 persis, tidak saya tambah diam-diam |
| Anchor otomatis per K entri (cadence D-A2) | penjadwalan = pemanggil (cron/engine loop), sesuai README `w0-anchor-impl` |

## Kalimat klaim yang diizinkan (addendum A1.4)

> "Tamper-evident terhadap penyerang yang tidak mengendalikan sink anchor maupun akar kepercayaan
> verifier, dengan jendela tak-terjangkar ≤ N entri; entri di dalam jendela berlabel `UNANCHORED`
> dan tidak diklaim terverifikasi."

**Dilarang:** "tamper-proof", "riwayat tidak bisa dipalsukan". E2E-5 di atas adalah buktinya.
