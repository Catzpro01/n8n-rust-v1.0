# SISTEM DISTRIBUSI TUGAS OTOMATIS SWARM (SOP & PROTOKOL DESENTRALISASI)
**Versi:** 1.1.0 | **Tanggal:** 2026-09-09 | **Otoritas:** Chief Supervisor (fern)
**Status:** MANDAT RESMI ARSITEKTUR SWARM | **Target:** Seluruh Agen (matt, agent1 s/d agent10)

---

## 1. TUJUAN & PRINSIP UTAMA

Sistem ini menetapkan tata kelola pembagian kerja otomatis tanpa ketergantungan pada satu titik koordinasi terpusat (No Single Point of Failure).

1. **Beban Merata & Terdistribusi**: Mencegah bottleneck di mana satu agen (misal matt atau agent1) kelebihan beban sementara agen lain menganggur (*idle*).
2. **Kesesuaian Kompetensi (*Role-Affinity*)**: Tugas diprioritaskan kepada agen dengan keahlian utama (*Primary Role*), dengan fallback ke agen sekunder (*Secondary Role*).
3. **Ketersediaan Tenaga Kerja Dinamis**: Penugasan memperhitungkan status aktif (*liveness/heartbeat*) dan beban kerja saat ini (*active load*).
4. **Klaim Atomik (*Work-Stealing*)**: Menggunakan transaksi basis data bersama dengan jaminan *First-Claim-Wins* untuk mencegah pengerjaan ganda.
5. **Penyesuaian Peran Kosong Otomatis (*Dynamic Vacant-Role Adoption*)**: Ketika sebuah peran kosong akibat batas token/rate-limit atau agen offline, agen komunitas yang tersedia (*idle*) secara otomatis mengadopsi peran tersebut berdasarkan pohon suksesi kapabilitas.
6. **Kepatuhan Zero-Code & Hard-Cap 500MB**: Semua tugas tetap terikat pada standar mutu bersama, verifikasi gerbang (L1 s/d L5), dan batas memori 500MB.

---

## 2. MATRIKS KUALIFIKASI & KAPABILITAS AGEN

Setiap agen memiliki peran spesialisasi utama serta kapabilitas cadangan:

| Agen | Primary Role | Domain Keahlian Utama | Secondary Role / Fallback |
| :--- | :--- | :--- | :--- |
| **@matt** | `ROLE_ARCHITECT` | Crate Cohesion, PRD Synthesis, Cargo Workspace | Kernel verification, AST review |
| **@agent1** | `ROLE_CORE` | Tokio DAG Runtime, Concurrency, Timeline Replay | MCP Server tools, Canary execution |
| **@agent2** | `ROLE_STORAGE` | SQLite WAL, SpillStore disk streaming, PITR | Database migrations, Storage indexing |
| **@agent3** | `ROLE_EXPRESSION` | QuickJS Bytecode Cache (EBC), CASD dedup | Expression evaluation, Benchmarking |
| **@agent4** | `ROLE_SCHEMA` | Schema Compiler, Workflow Rosetta, Hub Manifest | Node alias mapping, AST Normalization |
| **@agent5** | `ROLE_SECURITY_QA` | Differential QA, Security Gates (SEC-01..11), Corpus | Taint tracking, Fuzz testing |
| **@agent6** | `ROLE_WASM` | WASM Community Node Bridge (WCB), wasmtime | Sandboxing, Linear memory limits |
| **@agent7** | `ROLE_AI_MCP` | MCP Protocol Integration, JIT Routing (`n8n://`) | Token budgeting, AI Skill capsules |
| **@agent8** | `ROLE_PERF` | Performance profiling, Cgroups, Memory stress | Valgrind, Heap allocation audit |
| **@agent9** | `ROLE_INTEGRATION` | OpenAPI Codegen, Webhook lifecycle, Ingress API | Network protocols, HTTP idempotency |
| **@agent10** | `ROLE_COMPLIANCE` | Audit Trail, Execution Envelope, Lisensi, PII | Crypto-shredding, Hash chains |

---

## 3. ATURAN BEBAN KERJA & KETERSEDIAAN TENAGA KERJA

### 3.1. Batas Konkurensi Tugas (*Workload Cap*)
Untuk memastikan fokus kualitas dan pemerataan beban:
* **Maksimal Tugas Aktif**: Setiap agen HANYA BOLEH memegang **1 Active Task (IN_PROGRESS)** dalam satu waktu.
* **Maksimal Tugas Review**: Setiap agen HANYA BOLEH memegang **1 Review/QA Task** dalam satu waktu.
* Total kapasitas serentak per agen = **2 tugas** (1 Implementasi + 1 Peer-Review).
* Upaya mengklaim tugas baru saat kuota penuh akan DITOLAK oleh sistem antrean.

### 3.2. Deteksi Ketersediaan (*Liveness & Heartbeat*)
* Status agen diklasifikasikan menjadi:
  * `IDLE` : Aktif dan tidak memiliki tugas aktif (siap mengambil tugas baru).
  * `BUSY` : Aktif dan sedang memegang 1 tugas aktif.
  * `OFFLINE` : Tidak memperbarui heartbeat atau tidak ada aktivitas interaksi selama **> 10 menit**.
* **Protokol Auto-Release**: Jika agen yang memegang tugas `IN_PROGRESS` menjadi `OFFLINE` selama >10 menit tanpa laporan progres, tugas otomatis diubah kembali menjadi `UNCLAIMED` dan dicatat di `#error-log` sebagai `ERR-TASK-TIMEOUT`.

---

## 4. PROTOKOL PENYESUAIAN PERAN KOSONG OTOMATIS (DYNAMIC VACANT-ROLE ADOPTION)

Ketika sebuah agen mengalami kendala teknis (rate limit, kehabisan token konteks, atau offline), peran yang dipegangnya menjadi `VACANT`. Swarm mengadopsi peran tersebut secara otonom tanpa harus menunggu intervensi manual manusia:

### 4.1. Pohon Suksesi Kapabilitas Komunitas (Succession Tree)
Jika peran utama tidak aktif/tersedia, urutan suksesi otomatis yang berhak mengadopsi peran tersebut adalah:

1. **`ROLE_SECURITY_QA` Kosong**:
   * Tingkat 1: `ROLE_COMPLIANCE` (@agent10) — Mengawal gate kripto, isolasi keamanan, dan audit lisensi.
   * Tingkat 2: `ROLE_EXPRESSION` (@agent3) — Mengawal testkit dan uji diferensial.
   * Tingkat 3: `ROLE_CORE` (@agent1).
2. **`ROLE_STORAGE` Kosong**:
   * Tingkat 1: `ROLE_CORE` (@agent1) — Mengawal data-plane dan Tokio actor.
   * Tingkat 2: `ROLE_EXPRESSION` (@agent3) — Mengawal CASD dedup.
3. **`ROLE_AI_MCP` Kosong**:
   * Tingkat 1: `ROLE_INTEGRATION` (@agent9) — Mengawal HTTP ingress/egress.
   * Tingkat 2: `ROLE_CORE` (@agent1) — Mengawal tool and resource registry.
4. **`ROLE_WASM` Kosong**:
   * Tingkat 1: `ROLE_EXPRESSION` (@agent3) — Mengawal bytecode dan sandboxing.
   * Tingkat 2: `ROLE_SECURITY_QA` (@agent5 / @agent10).
5. **`ROLE_SCHEMA` Kosong**:
   * Tingkat 1: `ROLE_INTEGRATION` (@agent9) — Mengawal skema OpenAPI node.
   * Tingkat 2: `ROLE_CORE` (@agent1).
6. **`ROLE_INTEGRATION` Kosong**:
   * Tingkat 1: `ROLE_SCHEMA` (@agent4) — Mengawal kontrak Rosetta dan manifest Hub.
   * Tingkat 2: `ROLE_AI_MCP` (@agent7).
7. **`ROLE_ARCHITECT` Kosong**:
   * Tingkat 1: `ROLE_CORE` (@agent1) — Mengawal integrasi engine Tokio.
   * Tingkat 2: `ROLE_SCHEMA` (@agent4).

### 4.2. Mekanisme Klaim Peran Otomatis (Autonomous Stealing)
* Agen yang berstatus `IDLE` secara berkala memeriksa antrean tugas (`swarm-task next`).
* Jika ada tugas dari peran yang sedang `VACANT`, sistem otomatis memberikan tugas tersebut kepada agen aktif berikutnya dalam *Pohon Suksesi*.
* Agen mengadopsi tugas tersebut sebagai **`Acting Role`** tanpa mengubah identitas akun dasarnya.

### 4.3. Pemulihan Peran (Role Hand-Back)
* Begitu pemegang peran asli aktif kembali (sesi baru terhubung), pemegang asli mengirimkan pesan pengumuman `[AGENT_ID - ONLINE]`.
* Agen pengganti menyelesaikan tugas yang sedang berjalan (*graceful completion*), lalu menyerahkan kendali domain kembali ke pemilik peran utama.

---

## 5. STRUKTUR & LIFECYCLE TUGAS

### 5.1. Tingkat Prioritas
* **P0 (Blocker/Substrat)**: Prasyarat wajib bagi gelombang berikutnya.
* **P1 (Fitur Utama)**: Komponen fitur inti sesuai dependensi wave.
* **P2 (Penyempurnaan & Inovasi)**: Fitur tambahan, optimasi performa, integrasi sekunder.
* **P3 (Hardening/Refactor)**: Dokumentasi, stress testing ekstrem, audit kepatuhan pasca-implementasi.

### 5.2. Siklus Hidup Tugas (Lifecycle State Machine)
```
[UNCLAIMED] ──(klaim otomatis oleh agen IDLE / Suksesi)──> [CLAIMED]
     │
     └──(mulai pengerjaan)──> [IN_PROGRESS]
                                   │
                                   ├──(timeout >10m)──> [UNCLAIMED] (Auto-Release)
                                   │
                                   └──(selesai kode & tes lokal)──> [REVIEW_READY]
                                                                          │
                        [DONE] <──(diverifikasi peer-reviewer & QA)───────┘
```

---

## 6. PERALATAN TERMINAL SWARM (`swarm-task`)

Setiap agen di terminal VPS dapat berinteraksi dengan sistem antrean melalui utilitas `/usr/local/bin/swarm-task`:

* `swarm-task list` : Menampilkan seluruh antrean tugas beserta status dependensinya.
* `swarm-task next` : Menampilkan tugas terbaik berikutnya (termasuk tugas dari peran kosong yang diadopsi).
* `swarm-task claim <task_id>` : Mengklaim tugas secara atomik.
* `swarm-task heartbeat <task_id>` : Memperbarui liveness agar tugas tidak di-release.
* `swarm-task complete <task_id> --artifact <path>` : Menyerahkan tugas ke status `DONE` dengan melampirkan bukti artefak verifikasi.
* `swarm-task status` : Menampilkan visualisasi beban kerja 11 agen dan deteksi peran kosong secara real-time.

---

## 7. ESKALASI DAN RESOLUSI KONFLIK

1. **Deadlock / Blocker Antar-Tugas**: Jika suatu tugas terhambat oleh dependensi eksternal, tandai status `BLOCKED` dan umumkan rinciannya ke kanal spesifik terkait.
2. **Hak Veto Supervisor**: Chief Supervisor (`fern`) memegang hak veto untuk memindahkan, memprioritaskan ulang, atau membatalkan tugas jika terjadi anomali beban atau pelanggaran aturan batas 500MB RAM.
