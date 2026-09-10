# PRD — n8n-rust v3.0
## Workflow Engine Rust-Native: n8n-Compatible, Skala Super-Besar, Custom Node 8 Tier

**Versi:** 3.0 — **DISETUJUI**
**Tanggal:** 2026-09-10
**Penyusun:** Arena Agent (drafting) + Pemilik Produk (semua keputusan kunci)
**Status:** Menggantikan PRD v0.2 (`workspace matt/PRD.md`, status "publik + multi-tenant" DICCABAT)
**Sumber keputusan:** Sesi `grill-with-docs` ronde 1–2 (2026-09-10) · ADR 0001–0007 · `SPEC-N8N-RUST-V3-EXECUTION.md` · `CONTEXT.md`

---

## 1. Ringkasan Eksekutif

Membangun engine otomasi workflow **Rust-native** yang:
1. **Kompatibel n8n** — file workflow n8n asli (versi paku **2.39.0**) bisa diimpor, dijalankan, dan diekspor kembali (round-trip), dengan tampilan kanvas bergaya n8n dan dukungan MCP sebagai standar.
2. **Skala super-besar** — menjalankan **ratusan ribu hingga 1 juta node** dengan resource kecil dan efisien; fitur dasar harus jalan di **PC 1 vCPU / 500 MB RAM** tanpa kendala pada workflow panjang-berat.
3. **Kebebasan custom node** — 8 tier pembuatan node (dari script JS satu baris sampai crate Rust native, sampai node yang digenerate AI), jauh melampaui model single-tier (paket JS/TS) milik n8n asli.
4. **Untuk diri sendiri, self-hosted** — satu pengguna, di-host di VPS pribadi (2 vCPU/2 GB/50 GB + swap 4 GB) dan PC kecil.

Diferensiasi utama terhadap n8n asli: **efisiensi resource terukur** (bukan diklaim), **budget guard per-eksekusi**, **replay/time-travel deterministik**, dan **sandboxing WASM sejati** — semua dengan bukti benchmark di repo.

Bukti awal yang sudah ada (warisan tim sebelumnya, terverifikasi): payload **753 MB diproses dengan peak RSS 50,4 MB** oleh spill store, sementara pendekatan inline n8n **OOM-kill di 1,68 GB**.

**Hukum proyek (tidak berubah):** Correctness → Measurable Performance → Compatibility. Tanpa benchmark tidak ada klaim. Zero fake work: verifikasi dua arah.

---

## 2. Latar Belakang & Masalah

n8n asli (Node.js) memiliki batasan struktural untuk kebutuhan Pemilik:
| Masalah n8n asli | Dampak | Solusi di n8n-rust |
|---|---|---|
| Runtime Node.js + GC | OOM/latensi tak prediktif di payload besar (bukti: OOM 1,68 GB) | Rust + zero-GC + spill store (bukti: 753 MB @ RSS 50,4 MB) |
| Node = paket JS/TS + framework mereka | Membuat node custom berat, butuh rebuild, tanpa isolasi | 8 tier (termasuk WASM tersandbox, hot-reload, AI-generated) |
| Tidak ada pembatas per-eksekusi | Satu workflow gila membunuh seluruh proses (OOM-kill buta) | Budget guard per-eksekusi (konfigurabel) |
| Retry = jalankan ulang dari awal | Debug workflow panjang mahal | Journal + replay/time-travel |
| Queue mode wajib Redis | Beban ekstra di mesin kecil | Queue lokal zero-broker (SQLite WAL) |
| Minimum praktis ±1 GB RAM | Tidak muat di PC 1-core/500 MB | Target B0: 1 vCPU/500 MB |

**State proyek saat ini (2026-09-10):** 100 keputusan arsitektur (D1–D100) sudah dibuat; workspace `agent-workspace/rust-engine` punya 9–12 crate (kernel, executor, storage, nodes-core, nodes-wasm, mcp, rosetta, cli, dll.); tiket tracer-bullet TB-01…TB-10 tertulis; audit lama: "31/39 DONE tapi tidak ada satu workflow pun yang bisa dijalankan" → **TB-01 adalah tiket pertama yang harus lulus** (tracer bullet menembus semua lapisan).

---

## 3. Pengguna & Lingkungan

| Lingkungan | Spesifikasi | Peran |
|---|---|---|
| **VPS** (idcloudhost) | 2 vCPU / 2 GB / 50 GB, **+ 4 GB swap** (perintah Pemilik), Ubuntu 24.04, IP publik saat ini `103.55.37.234` (pernah `103.171.85.230`) | **Runtime produksi** (systemd + guard); build TIDAK di sini |
| **PC kecil** | **1 vCPU / 500 MB RAM** | Target fitur dasar B0 — workflow panjang-berat tanpa kendala |
| **Sandbox agent** | 2 vCPU / 4 GB; egress terbatasi (hanya GitHub, npm, PyPI) | Editing sumber + commit/push |
| **GitHub Actions runner** | 2 vCPU / 7 GB, internet penuh | **Mesin build + test** (dipicu push, hasil ke branch `ci-results`) |

Pengguna: **satu orang** (Pemilik). Auth = kunci/API key lokal; bind loopback/LAN/VPN. Multi-pengguna TIDAK dalam scope (lihat §5).

---

## 4. Tujuan Produk & Metrik Sukses (terukur)

| # | Tujuan | Metrik |
|---|---|---|
| G1 | Workflow super-besar jalan stabil | B1: 100.000 node, peak RSS < 500 MB (mesin 1 vCPU/500 MB) · B2: 500.000 node terukur · B3: 1.000.000 node terukur (swap hanya guard) · B4: konkurensi 100rb+ node simultan, throughput + RSS dilaporkan |
| G2 | Fitur dasar ringan | B0: workflow panjang-berat harian jalan di 1 vCPU/500 MB tanpa OOM/swap-thrash |
| G3 | Paritas n8n (tanpa pemangkasan) | Round-trip JSON n8n 2.39.0 lulus; 6 perintah CLI paritas Stage-1; UI kanvas; MCP standar; katalog node 2.39.0 tercakup bertahap (roadmap §10) |
| G4 | Custom node mudah | Setiap tier yang dibangun punya ≥1 contoh jadi di repo + lolos budget guard + dokumentasi 5 menit "node pertama Anda" |
| G5 | Keamanan terjamin | Kredensial AES-GCM; secrets 0 di log; sandbox per tier; audit log; hardening VPS (key-only SSH, fail2ban) |
| G6 | Error mudah dilacak + auto-solve | Setiap error = envelope terstruktur (node/item/stage/rantai sebab) + jurnal; auto-retry per-node; error workflow (parity n8n) |

---

## 5. Scope

### 5.1 IN SCOPE
- Segala hal di §6 (fitur fungsional) dan §7 (non-fungsional)
- 8 tier custom node (urutan build: ① JS → ③ WASM → ⑦/⑧ → ② Python → ④ Rust → ⑤ Proses eksternal → ⑥ AI-generated)
- Suite fitur tambahan §6.3 (Scrape Orchestrator, AI Agent Node, Workflow Hub, HTTP Orchestrator, konektor, sub-workflow)
- Deploy & operasi VPS (systemd, swap 4 GB, metrics, hardening)

### 5.2 OUT OF SCOPE (dipotong, pintu arsitektur dibiarkan)
- Multi-tenant, RBAC multi-pengguna, tim/sharing, mobile, marketplace publik
- (Kolom `tenant_id` default `local` tetap ada di skema storage — bisa dibuka lagi tanpa rewrite)

### 5.3 Aturan "Tanpa Pemangkasan Fitur Asli n8n" (keputusan Pemilik, dibatasi agar jelas tak error)
- **MCP = standar** sejak awal (bukan fitur ekstra)
- **Tampilan** = kanvas gaya n8n (familiar); boleh ditingkatkan
- **Katalog node n8n 2.39.0 = target paritas bertahap**: fase 1 inti (disepakati) → fase 2 konektor populer → fase 3 long-tail ditutup 8 tier custom node + MCP
- Single-user ≠ pemangkasan: semua fitur dinikmati sebagai satu pengguna; yang tak berlaku hanya fitur multi-pengguna

---

## 6. Requirements Fungsional

### 6.1 Engine Inti
- **FR-ENG-01** Eksekusi graph workflow: parse → trigger → jalankan node via trait `Node` (kernel) → keluaran item JSON; layanan belum diimplementasi = **fail-closed** (penolakan eksplisit, bukan perilaku diam)
- **FR-ENG-02** Representasi graph statis = adjacency ringkas di arena (bukan objek per-node); state per-node di luar memori via **spill store** (disk-backed)
- **FR-ENG-03** Eksekusi **streaming per-item** (tidak mematerialisasi seluruh daftar item di RAM)
- **FR-ENG-04** **Budget Guard** per eksekusi: max RSS / max waktu / max item; dilanggar = kill + laporan. **Bukan pemangkasan**: konfigurabel per workflow, bisa dinaikkan/dinonaktifkan per eksekusi (jejak audit)
- **FR-ENG-05** **Journal eksekusi deterministik** → replay/time-travel: jalankan ulang sampai node ke-N dengan input berbeda
- **FR-ENG-06** **Workflow-as-tool** dengan batasan jelas: kedalaman rekursi maks 5 (conf 1–10); maks 1.000 panggilan nested; budget diwarisi; **siklus terdeteksi saat compile** (error compile + rantai panggilan, bukan runtime)
- **FR-ENG-07** Queue lokal zero-broker (SQLite WAL); Redis opsional untuk HA (bukan wajib)
- **FR-ENG-08** Metrics endpoint satu pintu (`/metrics`): RSS, antrean, item/s, status guard

### 6.2 Paritas n8n (baseline fitur dasar)
- **FR-N8N-01** Round-trip: `import:workflow` → `execute` → `export:workflow` dari file ekspor n8n **2.39.0 asli** (fixture dari ekspor NYATA, bukan ditulis agar cocok)
- **FR-N8N-02** Node inti fase 1: Manual Trigger, Cron Trigger, Webhook Trigger, Set (kedua bentuk: `fields.values[]` legacy + `assignments.assignments[]` V2), IF, Switch, Merge, SplitInBatches, HTTP Request, Code (JS), NoOp, Respond + **MCP tool node**
- **FR-N8N-03** Kredensial: tersimpan terenkripsi **AES-GCM** (kunci dari env/file, tidak di DB)
- **FR-N8N-04** Schedule (cron) + Webhook inbound
- **FR-N8N-05** CLI paritas Stage-1: 6 perintah (`--help` root, `start`, `import:workflow`, `list:workflow`, `execute`, `export:workflow`) — nama & flag persis n8n (lihat CLI-PARITY-SPEC); 18 perintah sisanya = utang sadar
- **FR-N8N-06** REST API + **UI web kanvas gaya n8n** (daftar/buat/sunting/jalankan/lihat hasil)
- **FR-N8N-07** Riwayat eksekusi + retry + **Error Trigger workflow** (parity n8n)
- **FR-N8N-08** Divergensi hanya boleh "Divergensi Sadar" yang didaftarkan (diputuskan Pemilik); selain itu = cacat paritas

### 6.3 Custom Node — 8 TIER (FR-NODE-01…08)
| # | Tier | Medium | Build ulang? | Sandbox |
|---|---|---|---|---|
| 1 | Script Node | JS tertanam (rquickjs) | tidak | ya |
| 2 | Python Node | Python tertanam (PyO3) | tidak | ya (resource limit) |
| 3 | WASM Node | Rust/JS/→wasm (wasmtime) | **tidak (hot-load)** | **ya, kuat** |
| 4 | Native Rust crate | implement trait `Node` | ya | tidak (dipercaya) |
| 5 | External Process | bahasa apa pun, IPC lokal | tidak | ya (proses terpisah) |
| 6 | AI-Generated Node | deskripsi → engine tulis kode+test+node | sesuai hasil | sesuai hasil |
| 7 | Sub-Workflow Node | workflow lain sebagai node | tidak | ya (budget diwarisi) |
| 8 | MCP Tool Node | tool server MCP apa pun | tidak | ya |

Ketentuan lintas tier: kontrak item sama; budget guard wajib; **≥1 contoh jadi per tier**; SDK + dokumentasi "node pertama Anda ≤5 menit".

### 6.4 Suite Fitur Tambahan (fase 2–3)
- **FR-SCR-01 Scrape Orchestrator** — node scraping, mode **otomatis/manual** per eksekusi:

  | Backend | Jenis | API key/biaya | Fase |
  |---|---|---|---|
  | Crawlee + Playwright | open-source lokal | tidak | 1 |
  | Camofox | open-source anti-detect | tidak | 1 |
  | Scrapy | open-source lokal | tidak | 1 |
  | Firecrawl | hosted API | **YA** (free tier terbatas) | 2 |
  | Bright Data | proxy/data API | **YA** (berbayar) | 3 |
  | OxyLabs | proxy/data API | **YA** (berbayar) | 3 |

  Node berfungsi dari hari pertama dengan backend tanpa key; backend berbayar aktif setelah key dikonfigurasi.
- **FR-AI-01 AI Agent Node — 5 cabang:**
  - **Engine**: default bawaan, atau agent luar (Hermes, OpenClaw, opencode CLI, Claude, dst.) **melalui MCP**
  - **Memory**: multi-layer (>1 layer); backend bawaan n8n / Obsidian / graphify / dll; **local-first — otomatis tercatat tanpa habiskan token** (token hanya saat query); lapisan: working/session → project → long-term
  - **Skill**: tempel link GitHub langsung, atau dari folder skills global/project/local
  - **MCP**: sama seperti tools n8n biasa
  - **Output**: bentuk keluaran configurable
- **FR-HUB-01 Workflow Hub (fase AKHIR)** — konsep playstore: daftar workflow (sumber: GitHub) → klik → download otomatis ke canvas/list project
- **FR-HTTP-01 HTTP Orchestrator** — varian HTTP Request: beberapa client/router, mudah ditambah/dikurangi
- **FR-CON-01 Konektor** — GitHub, Gmail (pola konektor = template, mudah ditambah)

### 6.5 Keamanan (melintang)
- **FR-SEC-01** Sandboxing per tier (WASM/JS/Python terisolasi; node rusak tak meruntuhkan engine)
- **FR-SEC-02** Kredensial AES-GCM; **secrets tak pernah di log** (redaksi)
- **FR-SEC-03** Audit log (termasuk nonaktifkan-guard)
- **FR-SEC-04** Auth single-user (kunci/API key); bind loopback/LAN/VPN; VPS: key-only SSH + fail2ban + swap 4 GB
- **FR-SEC-05** Allowlist dependensi (patokan gate lama: 4 dependensi allowlist dijaga)

### 6.6 Error Model — "auto-solve / mudah dilacak"
- **FR-ERR-01** Envelope error terstruktur: node, item, stage, rantai sebab, redaksi secrets
- **FR-ERR-02** Journal → replay (FR-ENG-05)
- **FR-ERR-03** Auto-retry per-node: jumlah, backoff, kondisi (konfigurabel)
- **FR-ERR-04** Error workflow (Error Trigger) — parity n8n
- Definisi "auto-solve" = **auto-retry + fallback + jejak lengkap** (bukan magis)

---

## 7. Requirements Non-Fungsional

| # | Requirement | Target |
|---|---|---|
| NFR-1 | **Skala (bertahap & terukur)** | B0: harian @1vCPU/500MB · B1: 100rb node <500MB · B2: 500rb terukur · B3: 1jt terukur (swap=guard) · B4: konkurensi 100rb+ |
| NFR-2 | Benchmark harness permanen | Crate `bench`: generate graph deterministik (N node; pola linier/tree/DAG acak), ukur RSS via `/proc` (VmHWM), waktu, item/s; output masuk commit per fase |
| NFR-3 | Performa | Zero GC pause; latensi prediktif; throughput diukur per milestone |
| NFR-4 | Reliabilitas | Guard budget; fail-closed; crash satu eksekusi ≠ crash proses; systemd auto-restart |
| NFR-5 | Observability | `/metrics`, journal, audit log, redaksi secrets |
| NFR-6 | Portabilitas | Jalur CLI (headless) & UI; deploy biner tunggal ke Linux x86_64 |

---

## 8. Arsitektur (ringkas)

- **Kernel** (pure types + traits, frozen per D115: hanya serde/serde_json/async-trait/thiserror) → kontrak item, Node, SpillStore, error, id
- **Executor** — pipeline parse→walk→run; streaming per-item; fail-closed
- **Data-plane** — `FileSpillStore` (disk-backed, 690 baris, kanonik) + blob
- **nodes-core / nodes-openapi / nodes-wasm / mcp / rosetta (katalog+binding n8n) / storage / cli**
- **Graph statis** = arena adjacency; **state** = spill (bukti 50,4 MB)
- **Budget guard** di level executor (baca VmHWM per cek-point)
- **Journal** = append-only (dasar replay)

### 8.1 Build & Deploy (ADR-0005, amendemen 2 — keputusan Pemilik "langsung di GitHub saja")
```
Agent edit di sandbox ──push per tiket──> GitHub (source of truth, branch arena/**)
        GitHub Actions runner (2vCPU/7GB, internet penuh)
        ├─ cargo build --workspace + cargo test
        └─ hasil lengkap ──push──> branch `ci-results` (dibaca agent via git)
        Biner hijau ──deploy──> VPS (systemd + 4GB swap + guard)  [saat akses SG terbuka]
```
- Sandbox TIDAK bisa build Rust (egress: hanya GitHub/npm/PyPI — fakta terukur)
- VPS = **runtime saja**
- Progress: **1 tiket selesai = 1 commit + push**; tag per milestone

---

## 9. Diferensiasi vs n8n Asli (jujur: poin performa baru sah setelah benchmark lulus)

1. Skala 100rb–1jt node @ RAM kecil (n8n: OOM — bukti empiris di repo)
2. Jalan di 1 vCPU/500 MB (n8n praktis ≥1 GB)
3. 8 tier custom node (n8n: 1 tier)
4. Budget guard per-eksekusi (n8n: tidak ada)
5. Replay/time-travel (n8n: retry = dari awal)
6. Sandbox WASM sejati (n8n: Code node in-process)
7. Queue tanpa broker di mesin kecil (n8n: wajib Redis)
8. Scrape Orchestrator multi-backend built-in
9. AI Agent Node 5-cabang + memory multi-layer local-first
10. Workflow Hub pribadi (bisa offline)
11. Hot-reload node tanpa restart
12. Zero GC pause — latensi prediktif

---

## 10. Roadmap & Milestone

| MS | Isi | Kriteria lulus (terukur) |
|---|---|---|
| **M0** | CI loop GitHub Actions + **TB-01** (tracer: 1 workflow 2-node menembus semua lapisan) | Runner hijau; `tb01` cetak JSON valid; 1 mutan membuktikan test bisa gagal |
| **M1** | TB-02: paritas impor/jalankan/ekspor n8n 2.39.0 asli (6 perintah CLI) | Round-trip lulus dengan fixture ekspor nyata; `--help` cocok |
| **M2** | Suite node inti fase 1 (FR-N8N-02) + kredensial + schedule + webhook | Test per node + korpus n8n (12 workflow + 215 expression edge-cases) hijau |
| **M3** | REST API + UI kanvas gaya n8n (M3–M4 tiket lama) | CRUD workflow via UI + jalan + lihat hasil |
| **M4** | **Skala B0** (1 vCPU/500 MB) + Budget Guard + benchmark harness | Output harness: RSS < 500 MB pada workload panjang-berat; guard kill+laporan teruji |
| **M5** | Custom node tier ① JS, ③ WASM (hot-reload), ⑦ sub-workflow, ⑧ MCP-as-node | Contoh jadi per tier + test sandbox (node rusak tak crash engine) |
| **M6** | Suite: Scrape Orchestrator (fase-1 backends), AI Agent Node (5 cabang), konektor GitHub+Gmail, HTTP Orchestrator, tier ②/④/⑤ | Per fitur: contoh jadi + test; scrape jalan tanpa key |
| **M7** | **Skala B1→B3** (100rb/500rb/1jt node) + replay/time-travel + konkurensi B4 | Angka harness masuk commit; swap hanya guard (bukan jalan tol) |
| **M8** | **Workflow Hub** + parity long-tail via tier/MCP + hardening VPS final | Hub: klik→masuk canvas; SG final: IP Pemilik + IP agen saja |

Tiket existing TB-01…TB-10 (`TICKETS-TRACER-BULLET.md`) = tulang punggung M0–M3; tiket baru untuk M4–M8 dibuat via `to-tickets` saat masing-masing milestone mulai (1 tiket = 1 commit + push).

---

## 11. Risiko & Mitigasi

| # | Risiko | Mitigasi |
|---|---|---|
| R1 | **Akses VPS tertahan Security Group** (status 2026-09-10: IP agent diblokir di semua port; server 100% siap) | Dev loop TIDAK tergantung VPS (GitHub Actions); VPS hanya untuk M-runtime; pintu: SG `0.0.0.0/0` sementara → IP agen terekam di log → dipersempit permanen |
| R2 | Workspace warisan "hijau palsu" (patokan: commit d3bcff0 klaim 19/19 tapi tak bisa di-reproduksi) | Zero fake work: setiap klaim = perintah lengkap + output di commit; CI GitHub = gate independen |
| R3 | API drift executor↔kernel setelah lama | CI M0 langsung mengetesnya; kernel frozen (D115) meminimalkan permukaan |
| R4 | Build 1jt node membebani runner 2-core | Benchmark bertahap (B1 dulu); streaming + spill sudah ada; B3 di VPS (runtime) bila perlu |
| R5 | Dependensi berbayar scrape (Bright Data/OxyLabs) | Fase 3; node berfungsi tanpa key sejak fase 1 |
| R6 | Scope creep "tanpa pemangkasan" | Dibatasi §5.3: target paritas = katalog 2.39.0 bertahap + MCP + UI; divergensi harus terdaftar |
| R7 | Klaim performa tanpa bukti | NFR-2: harness permanen; hukum proyek |

---

## 12. Acceptance Criteria Utama (gate rilis)

1. M0–M8 lulus sesuai tabel §10 (setiap kriteria terukur, bukti di commit)
2. Round-trip n8n 2.39.0 lulus dengan fixture ekspor nyata
3. B0 lulus di mesin 1 vCPU/500 MB (output harness)
4. ≥1 contoh jadi per tier custom node yang dibangun, lolos budget guard
5. VPS: systemd + swap 4 GB + metrics hidup; SG final = IP Pemilik + IP agen
6. Secrets 0 di log (audit redaksi); kredensial AES-GCM terverifikasi
7. `npx skills update` / CI tetap hijau (regresi)

---

## 13. Keputusan Terbuka (butuh input Pemilik / pihak luar)

| # | Item | Status |
|---|---|---|
| O1 | Akses VPS: Security Group IDCloudHost untuk IP `103.55.37.234` (port 22) | **TERBUKA — memblokir M-runtime** (dev loop jalan via GitHub Actions) |
| O2 | Daftar katalog node 2.39.0 final per fase parity | Akan diinventarisasi saat M2 (fakta dari repo n8n upstream) |
| O3 | API keys scrape fase 2–3 (Firecrawl, Bright Data, OxyLabs) | Saat M6–M8 |
| O4 | Provider memory AI Agent Node (bawaan/Obsidian/graphify) — default awal | Rekomendasi: bawaan (SQLite+vector) + graphify; Obsidian fase lanjut |
| O5 | Rotasi password `fern` (pernah lewat chat) + finalisasi `PasswordAuthentication no` | Saat akses VPS terbuka |

---

## 14. Dokumen Rujukan

| Dokumen | Lokasi |
|---|---|
| ADR 0001–0007 (+amendemen) | `docs/adr/` |
| Spec v3 eksekusi | `PRD_ADR_DOCS/docs/spec/SPEC-N8N-RUST-V3-EXECUTION.md` |
| Glosarium & konteks | `CONTEXT.md` |
| Tiket tracer-bullet TB-01…TB-10 | `PRD_ADR_DOCS/docs/spec/TICKETS-TRACER-BULLET.md` |
| CLI parity + Divergensi Sadar | `PRD_ADR_DOCS/docs/spec/CLI-PARITY-SPEC.md` |
| Keputusan D1–D100 | riwayat sesi + `PRD_ADR_DOCS/` |
| PRD v0.2 (diganti, jejak) | `workspace matt/PRD.md` |
| Workflow CI | `.github/workflows/rust-ci.yml` |
| Aturan agent & skill | `CLAUDE.md`, `docs/agents/`, `docs/agents/skill-shortcuts.md` |
