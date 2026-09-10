# ADR v2 — TIME-TRAVEL EXECUTION REPLAY (Berbasis Lineage + Content-Digest)

| | |
|---|---|
| **Pilar** | 2 — Penambahan fitur bernilai tinggi (leveraging data yang SUDAH ada) |
| **Pengusul** | agent10 (ROLE_COMPLIANCE) — verifier yang selama ini menguji jalur lineage; proposal ini lahir dari bukti disk yang saya kumpulkan hari ini, bukan dari desain ideal |
| **Status** | SUBMITTED — Sayembara ADR v2 ZERO-SACRIFICE (fern #1490 §3) |
| **Dasar hukum data** | Ruling 18 (#1116) BLAKE3-256 salted + content_proof_plain; Ruling 15 SpillPath filename-basis; Ruling 32/33 retryability; W0-CONTEXT (656e265) LogFields redaksi; G-4 idx_lineage_edge_output_ref (#1345, terukur 100k) |
| **Komplementer, bukan tabrakan** | proposal agent2 (WAL+Batch INSERT) = jalur TULIS; proposal ini = jalur BACA read-only; CASD agent3 (spill_gc/spill_intent) dipakai apa adanya, tidak disentuh |

## 1. Masalah

Hari ini, sesudah eksekusi selesai, jawaban atas pertanyaan debugging paling umum tidak punya API:

1. "Node X menghasilkan output apa pada attempt ke-2?" — `node_output` (001:134) terbaca, tetapi tersebar; komponen tidak tahu urutan waktu.
2. "Item mana yang masih hidup setelah permintaan erasure W?" — `erasure_log` (004:33) append-only menyimpan jejaknya, tetapi TIDAK ADA pembaca yang menautkannya ke lineage.
3. "Apakah digest yang tercatat masih utuh (tidak korup, tidak diubah)?" — `content_hash` BLAKE3-256(salt_exec‖canon) (Ruling 18) tersimpan, tetapi tidak pernah DIVERIFIKASI ulang oleh jalur baca mana pun.

Konsekuensi: satu-satunya cara menjawab ketiganya adalah membuka SQLite manual (sqlite3 CLI) — bukan produk. Dan tidak ada "waktu" yang dilayani: semua pembacaan melihat keadaan TERAKHIR, tidak ada cara melihat eksekusi SEBAGAIMANA IA TERJADI.

## 2. Solusi — Replay read-only, 100% reuse infrastruktur

Modul baca `replay` (lokasi: keputusan @matt — lihat §7; saya tidak berasumsi lokasi) yang mengkonsumsi SATU eksekusi dan merekonstruksi urutan aslinya:

```
replay(execution_id):
  stream = SELECT l.execution_id, l.node_id, l.attempt, l.output_index, l.seq_start, l.seq_end, l.output_ref
           FROM lineage_lookup l
           WHERE l.execution_id = ?1
           ORDER BY l.seq_start                          -- urutan waktu ASLI (LIN-1)
           -- memakai idx_lookup_query (execution_id, node_id, attempt, output_index, seq_start)
           -- G-4: reverse (output_ref -> execution) memakai idx_lineage_edge_output_ref
  untuk setiap baris (window streaming, bukan materialisasi penuh):
    edge  = SELECT content_hash FROM lineage_edge WHERE execution_id=? AND output_ref=?   -- Ruling 18 digest
    ext   = SELECT data_class, content_proof_plain FROM lineage_ext WHERE (execution_id, output_ref)
    erasure = SELECT destroyed_at FROM erasure_log WHERE execution_id=? AND output_ref=?   -- append-only
    if erasure ada                        -> materi PCI: {redacted:true, destroyed_at} (pola put_credential W0-CONTEXT)
    else if data_class = 'PERSONAL'       -> materi redacted: content_proof_plain WAJIB NULL (Ruling 18)
    else                                  -> Item = FileSpillStore.get(spill_path)  (dari node_output.spill_path, 001:139)
                                              VERIFIKASI: recompute BLAKE3-256(salt_exec || canon(item))
                                              == edge.content_hash ?  OK  :  STOP + error terlihat (digest trail rusak)
  keluaran: stream Item (bisa -> --json deterministik, tanpa duration_ms, tanpa warna) | --dot (visual graf)

Sifat kunci (semua dari fakta disk, bukan niat):
- READ-ONLY: tidak ada INSERT/UPDATE di jalur replay. Tidak menyentuh hot path runtime.
- Deterministik: tanpa wall-clock; urutan dari seq_start; digest di luar representasi (R-2, INTEG-07 pola).
- GDPR-aware: erasure_log + data_class dibaca SEBELUM materi; PERSONAL/output ter-erasure TIDAK PERNAH muncul polos (redaksi by-construction).
- Fail-closed: digest mismatch / spill_path hilang / row lineage_ext absen -> ERROR TERLIHAT, bukan skip diam-diam.
- SpillPath filename-basis (Ruling 15): resolver = node_output.spill_path -> FileSpillStore (spill.rs:177, deserialize_item :253, checksum SHA-256 :276 terbukti ada) — BUKAN content-id addressing.

## 3. Bukti Zero-Regression

Definisi sayembara: TANPA memangkas fitur, TANPA mengurangi akurasi/kompatibilitas, hard-cap <500MB RSS.

1. **Nol DDL, nol migrasi**: tidak menyentuh 001/002/004 — replay hanya MEMBACA tabel. (Berbeda dari agent2 yang memakai 005 opsional — tidak tumpang tindih.)
2. **Nol perubahan hot path**: modul baru; runtime kernel/storage TIDAK diubah satu baris pun. Argumen struktural, sama dengan INTEG-05 agent9 (zero-leak: fitur baca tidak menambah jalur tulis).
3. **Nol pengujian yang berubah**: 19/19 storage test + 55/55 kernel test TIDAK disentuh; replay punya suite sendiri.
4. **Verifikasi-nya sendiri adalah mutan** (bukan klaim): seluruh tabel fakta di §1 sudah teruji oleh saya HARI INI — G-4 reverse index (100k, SEARCH vs SCAN, #1401), M1a/M1c/M3b/G-2 mutan MATI (#1288/#1356/#1401/#1448), digest trail Ruling 18 (keputusan #1116, diuji LIN-2 sebagai demonstrasi + label jujur #1448).
5. **Boundary jujur**: replay mengikuti retensi (retention_expired_at) — item yang sudah dibersihkan GC direportasikan sebagai "DESTROYED (retensi)", tidak diciptakan ulang. Ini BUKAN pengurangan fitur: ia jujur terhadap tabel yang memang sudah append-only.
6. **Kompatibilitas**: keluaran JSON deterministik (R 51 §4.1 pattern — tanpa field presentasi waktu di --json), sama pola yang sudah diputuskan untuk `crates/cli`.

## 4. Proyeksi metrik terukur (SEMUA = proyeksi yang akan saya buktikan dengan harness; bukan angka klaim)

| Metrik | Proyeksi | Bukti yang akan saya tempel saat implementasi |
|---|---|---|
| Latensi Item-pertama | < 10 ms utk eksekusi 100k edge (SEARCH idx_lookup_query + G-4) | harness /usr/bin/time, 5 run median |
| RSS stream 100k edge | < 80 MB (window 1.000 item ≈ 64 KB/inv; materi besar tetap di spill, kerangka D28) | /usr/bin/time -v, Max RSS |
| Verifikasi digest | < 10 ms per 100k edge (BLAKE3 ≈ 1,4 GB/s atas 64-128 B/item) | bench internal, di-pin seperti sha2 =0.10.8 |
| Kompleksitas baca | O(n) baris + O(log n) per lookup — nol N+1 yang tersembunyi (semua JOIN memakai indeks yang SUDAH terukur) | EXPLAIN QUERY PLAN disitir |
| Hard-cap 500 MB | TIDAK DILANGGAR — replay streaming; tidak pernah me-materialisasi seluruh eksekusi | sama seperti gate agent6 wasmtime (<500MB) |

Semua angka di tabel = PROYEKSI sampai harness berdiri; pola yang sama dengan pengukuran 100k G-4 saya (#1401: angka + catatan jujur bahwa probe sintetis).

## 5. Rencana Verifikasi Mutan (Ruling 27 — setiap mutan WAJIB membunuh ≥1 test)

| Mutan | Mutasi | Test yang harus GAGAL |
|---|---|---|
| M-R1 | `salt_exec` DIHAPUS dari recompute digest | replay_reject_corrupt_digest (digest mismatch terdeteksi) |
| M-R2 | baris erasure_log DILEWATI | replay_gdpr_redacts_destroyed (materi ter-erasure muncul polos) |
| M-R3 | PERSONAL mengeluarkan content_proof_plain | replay_personal_never_plain (Ruling 18: PERSONAL → NULL) |
| M-R4 | urutan diurutkan ulang (bukan seq_start) | replay_order_matches_seq_start (order deterministik) |
| M-R5 | spill_path salah → Item skip diam-diam | replay_fails_loud_on_missing_spill (fail-closed) |
| M-R6 | keluaran memuat duration_ms/digest durasi | replay_json_deterministic (R-2; 2 run seed sama = byte identik) |

## 6. Apa yang SUDAH ada di disk (referensi baris — Pelajaran 20)

- `001_initial_schema.sql:134,139` — `node_output` … `spill_path TEXT`
- `001_initial_schema.sql:148,160` — `spill_intent`, `spill_gc` (CASD — milik sayembara agent3, dipakai apa adanya)
- `004_item_lineage.sql:12` — `lineage_edge` (repr CHECK 4-cabang G-2, content_hash 32B BLAKE3-256 salted)
- `004_item_lineage.sql:33` — `erasure_log` append-only
- `004_item_lineage.sql:75-77` — `lineage_ext` (data_class CHECK, content_proof_plain NULL-able, Ruling 18)
- `004_item_lineage.sql:36` (G-4) + `:70` (idx_lookup_query utk LIN-1) — indeks TERUKUR (#1401: 100k, SEARCH vs SCAN)
- `kernel-asli crates/data-plane/src/spill.rs:177` — `FileSpillStore`; `:253` deserialize_item; `:276` load_and_verify-header (checksum SHA-256)
- `crates/kernel/src/context.rs` — LogFields::put_credential (656e265) — pola redaksi replay
- `lineage.rs` LIN tests (L1-L3, 19/19, mutan M1a/M1c/M3b/G-2 MATI)

## 7. Keputusan yang saya butuhkan (jujur — saya TIDAK menjawabnya sendiri)

1. **Lokasi modul**: (a) subcommand `crates/cli` (agent2 sedang membangun cli — koordinasi), (b) modul read-only di `crates/storage` (kepemilikan agent2), (c) crate tipis baru `crates/replay` — perlu pengangkatan "stop new crates" #1125 oleh @matt. Rekomendasi netral saya: (a) dengan lapisan baca di storage — mengikuti pola lapisan existing.
2. **Scope erasure**: replay menampilkan `{redacted, destroyed_at}` untuk item ter-erasure — konfirmasi ini yang diminta (bukan menampilkan apa pun yang tersisa, dan bukan menciptakan ulang).
3. **Antarmuka**: CLI `--json` deterministik + `--dot` — cukup untuk v1? (Web App PRD-2 §9 bisa memakai --json sebagai feed visual — pilihan masa depan, bukan scope sekarang.)
4. **Prioritas vs sayembara lain**: proposal ini murni fitur baca; bila sayembara memprioritaskan Pilar 1 (performa), saya tetap submit karena leverage-nya tinggi (biaya implementasi kecil: satu modul baca + harness ukur; seluruh data sudah ada).

## 8. Rencana eksekusi (bila diterima)

1. Harness ukur (§4) di pohon verifikasi saya — angka dipublikasikan SEBELUM kode fitur (ukur dulu, jangan asumsikan).
2. Modul baca + 6 gerbang INTEG (R1-R6 di atas; pola gate falsifiable RFC-EXEC) — tiap gerbang = mutan yang sudah ditentukan.
3. Publikasi tree + sha + 3-baris R42 + mutan tempel — pola yang sudah jadi standar hari ini.
4. Review 2/2 + otorisasi (jalur §3.4-style) — tidak melompati gerbang.

— agent10, ROLE_COMPLIANCE, 2026-09-09 ~16:15 WIB
