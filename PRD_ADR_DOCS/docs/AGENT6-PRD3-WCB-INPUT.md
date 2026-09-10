# AGENT6-PRD3-WCB-INPUT — Masukan agent6 untuk Rakitan PRD-3 (Pilar WCB / WASM)

**Penulis:** agent6 (ROLE_WASM) · **Tanggal:** 2026-09-09 · **Status:** INPUT siap-serap
**Tujuan:** memberi matt (pemilik rakitan `PRD-3-PERFECTION-CHECKLIST.md`) paket ringkas pilar WCB: keputusan terkunci, dokumen sumber, usulan butir checklist, ketergantungan, dan pertanyaan terbuka. Patuh freeze #386/#513 (nol kode).

---

## 1. Ringkasan pilar (untuk diletakkan di PRD-3)

**WCB (WASM Community Node Bridge)** = lapisan ekstensibilitas node komunitas engine: node/adaptor/transform pihak-ketiga dikompilasi ke WebAssembly dan dijalankan di **wasmtime** dengan isolasi **≤32 MiB linear memory per instance**, **pooling allocator** (Engine+modul shared, Store reuse), tanpa WASI/ambient authority pada MVP (antarmuka `wcb:*` milik sendiri), egress jaringan hanya via host function ber-allowlist per manifest. Menutup gap 694-node n8n vs ~26 MVP **tanpa** menarik dependency npm/node_modules, dan menjaga tesis memori (host-streaming item spilled via `read_at`/`read_range`, tidak materialisasi penuh ke guest). WCB = **substrate eksekusi**, BUKAN endpoint/MCP/expression/native-connector.

## 2. Keputusan terkunci (pemilik/supervisor/adjudikasi, 2026-09-09)

| ID | Keputusan | Lokasi rincian |
|---|---|---|
| Kunci-1 | Runtime = **wasmtime** (component model penuh + pooling + async; wasmer partial/WASIX lock-in) | feasibility #467 |
| Kunci-2 | **MAX 32 MiB/instance** = batas linear-memory guest; pooling wajib; RSS = ranah ukur (bukan batas) | #431 fern; definisi #470 agent1; akuntansi ERR-029 |
| Kunci-3 | MVP ABI = **wcb:* minimal tanpa WASI** (permukaan kecil); component-model/WIT = pasca-MVP (D-W2) | Q3 #470; AGENT6-WCB-SPEC §4 |
| Kunci-4 | Egress jaringan MVP = **allowlist statis per manifest** + host function `wcb:net/request` | Q2 #492; SEC-WCB-03 |
| Kunci-5 | SEC-WCB-01..04 kanon = bingkai agent6 (menyerap agent3 #429) | #488 agent5 |
| Kunci-6 | Boundary: **two-gate admission** (API agent1 → sandbox agent6) + **exec_endpoint = agent9+agent1**; agent6 = substrate | #577/#579; AGENT6-WCB-SPEC §2.1 |
| Kunci-7 | WCB = extensibility NODE; TIDAK menggantikan QuickJS/EBC (agent3) atau native connectors (agent9) | #573/#575 |
| Kunci-8 | Posisi crate `wcb/` di LUAR kernel (aturan §3.3 PRD-2 tetap); desain thd **trait** kernel | #467; spec §2 |
| Kunci-9 | **Satu kerangka manifest_sig Ed25519** utk Hub-template & WCB-plugin; trust-anchor registri agent7 (`n8n://plugin/{id}/manifest`, D-W6); rotasi dual-key (SEC-TRUST-01); TOCTOU dua-tahap | #611/#624, #602/#612, #621, #619/#622 |

## 3. Dokumen sumber pilar (verifikasi sha256 di server)

- `AGENT6-WCB-SPEC.md` (induk; v0.4 — `f02326cf0d6ab2784aaa4918b70046e6a1043671086fd05fb97e204874de9d92`)
- Konsep WCB: pesan #375 agent3 (forum) · adjudikasi #431 (forum) · feasibility #467 (forum)
- Sayembara PRG #493 → lampiran kebijakan (spec §9)
- Klaim Hub-substrate + HUB-ADAPTER-WASM #531 · WCB-TRANS #574 (usulan katalog)

## 4. Usulan butir checklist PRD-3 (WCB)

| ID | Butir | Gate falsifiable (diukur agent8 / di-audit agent5) |
|---|---|---|
| W-1 | Runtime wasmtime + pooling, batas 32 MiB/instance | Modul alokasi >32MiB → dibatasi/di-kill; konkuren > cap → antre |
| W-2 | SEC-WCB-01 import allowlist + validasi statis modul | Import fd_*/sock_*/proc_*/wasi-clock → ditolak saat VALIDATE; hash ≠ manifest → ditolak |
| W-3 | SEC-WCB-02 resource enforcement (fuel/timeout/cap) | Infinite loop → fuel/timeout mematikan |
| W-4 | SEC-WCB-03 no ambient authority (secret injeksi, egress allowlist) | Baca secret tanpa injeksi → GAGAL; egress non-allowlist → DIBLOKIR |
| W-5 | SEC-WCB-04 supply-chain + audit — manifest_sig Ed25519 terpadu (kontrak agent10), rotasi dual-key, TOCTOU dua-tahap; receipt | Sig invalid/revoked → ditolak; re-verify first-load (verify-or-refuse); receipt memuat hash modul + batas (koordinasi Envelope agent10: metadata, bukan field berubah-runtun) |
| W-6 | Host-streaming ItemList spilled (`read_at`/`read_range` + cursor) | WCB tidak materialisasi list spilled penuh ke guest (anti single-blob n8n) |
| W-7 | Determinisme (seed `execution_id` via `wcb:entropy/seed`) | Verifikasi di harness qa/ agent5 |
| W-8 | Spike ukur: baseline RSS engine, delta/instance, cold/warm call | Angka + batas memori eksplisit (BENCH-REPRO agent5) |

## 5. Ketergantungan & posisi roadmap

- **Wave 1** (setelah PRD-3 dibuka; tidak butuh K-1/L0/CASD): WCB spike + implementasi inti. Prasyarat non-teknis = pembukaan zero-code. Toolchain shared tersedia (#504, rustc/cargo 1.98.1).
- Konsumen substrate: engine agent1 (per-invokasi), kompresor agent1+3, Hub-adaptor komunitas, WCB-TRANS.
- Titik koordinasi: CASD (dedup spill WASM) — agent2/3; Envelope/audit (SEC-WCB-04) — agent10; Hub — agent4/7/9; gate QA — agent5; perf — agent8.

## 6. Pertanyaan terbuka (D-W*, untuk rakitan)

| ID | Pertanyaan | Usul |
|---|---|---|
| D-W1 | Contoh tipe node komunitas pertama (MVP) | transform/set/parse; sinkron dengan node-list (T-13) |
| D-W2 | ABI: module-model core dulu vs component-model/WIT | module dulu; component cadangan pasca-MVP |
| D-W3 | Paket: 1 file `.wasm`+manifest vs terpisah; registri lokal | 1 file; registri lokal dulu |
| D-W4 | WCB mono-repo `crates/wcb/` vs repo terpisah utk SDK | mono-repo saat implementasi |
| D-W5 | Bahasa SDK v1 (Rust / AssemblyScript / keduanya) | keduanya bila toolchain mendukung |
| D-W6 | Namespace registri trust-anchor plugin | **DIPUTUSKAN #624**: `n8n://plugin/{id}/manifest` di registri agent7 |

— agent6 (ROLE_WASM)
