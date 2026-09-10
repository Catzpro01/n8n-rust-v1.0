# ADR-0004: Baseline fitur dasar (paritas inti n8n, fleksibel)

**Status:** DITERIMA (keputusan Pemilik, 2026-09-10)

## Garansi keras
1. **Round-trip JSON n8n asli**: import → jalankan → export file ekspor n8n 2.39.0
   (fixture dari ekspor NYATA, bukan ditulis agar cocok).
2. **Jalan di 1 vCPU / 500 MB RAM** dengan workflow panjang-berat tanpa kendala (ADR-0002 B0).

## Isi baseline
- Node inti fase 1: Manual/Cron/Webhook Trigger, Set, IF, Switch, Merge, SplitInBatches,
  HTTP Request, Code (JS), NoOp, Respond + **MCP tool node** (tidak dipotong — ADR-0006).
- Kredensial (tersimpan terenkripsi AES-GCM; kunci dari env/file, tidak di DB).
- Schedule + Webhook; CLI (6 perintah paritas Stage-1: `--help`, `start`,
  `import:workflow`, `list:workflow`, `execute`, `export:workflow`) + REST API.
- UI web minimal **bergaya kanvas n8n** (daftar/buat/sunting/jalankan/hasil = M3–M4).
- Riwayat eksekusi + retry error + error workflow (Error Trigger) — n8n parity.

## Batas fase
Node katalog n8n di luar fase 1 dipenuhi bertahap (parity roadmap, ADR-0006)
dan/atau via 8 tier custom node (ADR-0007) — bukan janji satu-besar.
