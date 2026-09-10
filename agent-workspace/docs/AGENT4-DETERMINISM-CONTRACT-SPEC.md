# DETERMINISM CONTRACT & REPORT — spesifikasi (draf agent4)

Status: DITERIMA masuk katalog inovasi PRD-3 (fern #399, wave-2); deliverable W1-DETERM-SPEC
(queue swarm-task, 2026-09-09). Dokumen pelengkap pesan #368. Riwayat: v1 → v1.1 (tambah §7).
Basis: DETERMINISM-SOURCE-INVENTORY.md (agent5), spec engine v0.4.3 §6.2.1 determinism-record (agent1),
temuan upstream cron random-second (#264), N-rules normalizer, DEVIATION-CATALOG §6 K1-K10 (agent5).

## 1. Tujuan
Tiap workflow punya kontrak determinisme eksplisit yang:
(a) dibuat SAAT IMPOR (analisis statis, zero biaya runtime),
(b) ditegakkan saat eksekusi (mode deterministik / deteksi pelanggaran),
(c) dilaporkan ke pengguna (REPORT) + ke receipt eksekusi (jejak pelanggaran).

## 2. Output impor: DETERMINISM REPORT (JSON kecil, disimpan sbg metadata)
```json
{
  "schema": 1,
  "workflow_sha": "…",
  "verdict": "DETERMINISTIC | CONDITIONAL | NON_DETERMINISTIC",
  "sources": [
    {"node": "code-1", "type": "clock", "detail": "Date.now() di jsCode (baris ~2)", "class": "N1"},
    {"node": "http-2", "type": "external", "detail": "httpRequest GET api.x", "class": "E"}
  ],
  "external_points": ["http-2"],          // utk replay: wajib direkam
  "deterministic_mode": {"eligible": true, "seed_size_bytes": 8}
}
```
Kelas sumber nondeterminisme (urutan prioritas by frekuensi korpus, agent5):
E eksternal (74% file) -> jam/clock (28 file) -> cron trigger (14 by regex/13 node) -> RNG (3)
-> execution.id/webhookUrl (3) -> iterasi/urutan objek (lintas-runtime K3) -> format angka (K2)
-> potongan string UTF-16 vs UTF-8 (K6). Dua kelas terakhir = DEVIATION antar-runtime, bukan
replay-run-sama; tetap masuk report sbg catatan L3.

## 3. Mode eksekusi & degradasi jujur
1. Default: eksekusi normal; RUNTIME VIOLATION LOG mencatat akses clock/RNG liar di luar kontrol
   (node code) -> masuk receipt eksekusi (bukan gagal, bukan senyap).
2. Deterministic mode: PRNG ter-seed (8B, dari execution seed), jam beku (wall-clock eksekusi),
   urutan iterasi ditetapkan, external_points via record-replay. Dua run seed sama -> byte-identik
   (gate, lihat §4).
3. NON_DETERMINISTIC verdict + pengguna minta deterministic: ditolak dgn alasan eksplisit
   (daftar sumber), bukan dijalankan lalu bohong.

## 4. Gate falsifiable (utk QA L2/L3 & fitur lain)
- G1: 7 workflow RUNNABLE subset exec-diff -> mode deterministik, 2 run seed sama = output kanon
      identik 7/7; seed beda -> output boleh beda TAPI violation/seed tercatat.
- G2: workflow NON_DETERMINISTIC sintetis (Date.now di code) -> report verdict benar; mode
      deterministik ditolak dgn alasan.
- G3: violation log: code node panggil Math.random TANPA ijin di mode deterministik -> tercatat,
      bukan divergen senyap (ini yg membuat TIMELINE/ENVELOPE jujur).
RAM hot-path: 0 byte struktural (report di disk; seed 8B di konteks). CPU: analisis statis sekali
saat impor; cek violation = 1 flag check per panggilan proxy (agent3 QuickJS proxy).

## 5. Relasi dokumen lain
- Agent5: DETERMINISM-SOURCE-INVENTORY = data sumber; DEVIATION-CATALOG §6 = aturan antar-runtime.
- Agent1: spec engine §6.2.1 determinism-record = isi record replay; ini REPORT versi runtime.
- Agent4: N-rule & temuan cron = dasar kelas sumber; ROSETTA = penghapus sumber via migrasi
  (cron random-second dibuang ke detik=0 eksplisit).

## 6. Keputusan terbuka (utk PRD-3)
- O1: otoritas V8-vs-QuickJS saat beda (usul: V8 = acuan kompat; bug V8 -> N8N-BUG di katalog).
- O2: kategori N8N-NONDETERMINISTIC utk cron (detik=0 eksplisit) — menunggu matt.
      **UPDATE 2026-09-09: tertangani PRD-3 v3.2 (matt) §5 L3 matriks** — engine↔engine
      PIN; n8n↔engine NORMALISASI. Verdict NON_DETERMINISTIC report §2 tetap berlaku sbg
      klasifikasi impor; perlakuan diff mengikuti matriks.
- O3: kebijakan default mode utk workflow CONDITIONAL (rekomendasi: default normal + report;
      deterministic mode opt-in via flag/UI).

## 7. Konsumen REPORT determinisme (2026-09-09, konvergensi Hub)

- Workflow Hub auto-update (AGENT4-WORKFLOW-HUB-MANIFEST §3): verdict NON_DETERMINISTIC =
  tidak layak auto-update senyap; (template_sha256, content_version) masuk determinism-record
  (agent1 #536, OPEN-HUB-1); otoritas jadwal = engine-scheduler (poll klien = transport-only).
- Eksekusi-dua-versi (live vs candidate, OPEN-HUB-1) = aplikasi langsung mode deterministik §3:
  kedua run seed sama → pembandingan output kanon valid (gate diff #374).
- dry_run.est_token Hub = turunan metadata statis, BUKAN eksekusi (H-3b) — tidak menyentuh report.
