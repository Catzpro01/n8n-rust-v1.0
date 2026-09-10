# PRD-2 — n8n VERSI RUST: ADAPTASI & PENINGKATAN

**Dokumen:** PRD teknis untuk reimplementasi n8n dalam Rust
**Versi:** 1.4 — DRAFT, belum disetujui
**Tanggal:** 2026-09-09
**Dependen pada:** `PRD-1-N8N-ANALYSIS.md` (analisis n8n asli), `KERNEL-SPEC.md`, `BENCH-A03.md`, `ANALISIS-KORPUS-NODE.md` (bukti daftar node §7.3), `SINTESIS-SPILLSTORE.md` (bukti status Fase 0 §12)
**Riwayat:** v1.0 draf awal · v1.1 menyerap audit adversarial F1–F21 (15 diterima penuh, 3 diterima sebagian, 3 ditolak) — lihat `ADJUDIKASI-F1-F21.md` · v1.2 menambah §7.3.1 + T-12/T-13 dari temuan korpus terverifikasi (node deprecated `function`/`cron` masih dipakai template publik) — lihat `VERIFIKASI-DELIVERABLE-TIM.md` · **v1.3 menulis ulang §7.3 berbasis analisis korpus 171 workflow** (`ANALISIS-KORPUS-NODE.md`): mengoreksi klaim v1.2 yang tidak benar bahwa daftar node diturunkan dari data nyata, menambah T-14 (janji kompatibilitas produk), memperbarui T-12/T-13 dengan angka korpus penuh · **v1.4 mengoreksi §12 Fase 0 berdasarkan eksekusi nyata `check-freeze.sh` di VPS**: status diubah ✅ SELESAI → ⚠️ BELUM SELESAI (gate gagal, exit 1), deskripsi stub dikoreksi, ditambah kualifikasi cakupan test dan kegagalan SEC-07 — lihat `SINTESIS-SPILLSTORE.md`
**Status kernel:** PROVISIONAL-FREEZE, bukan beku (revisi audit F10, lihat §12 Fase 0)
**Target produk:** Self-hosted, single-tenant, seperti n8n Community Edition

> ⚠️ **DRAFT. Keputusan di §14 masih terbuka dan milik Anda.**
> Dokumen ini sengaja memisahkan "rekomendasi teknis" dari "keputusan".

---

## 1. Tesis Produk

**Satu kalimat:** Engine otomasi workflow yang kompatibel dengan workflow n8n, ditulis di Rust, yang menjalankan beban kerja besar di perangkat keras kecil tempat n8n kehabisan memori.

**Tiga klaim, dan status buktinya:**

| Klaim | Status | Tingkat bukti | Bukti & reproduksi |
|---|---|---|---|
| Lebih hemat memori secara struktural | ✅ TERBUKTI | **KERNEL, komponen terisolasi** — bukan produk jadi | `BENCH-A03.md`, 2026-09-09, sandbox 2 core / 1.985 MB RAM / **swap 0**: 5 juta item / 753 MB payload dalam 50,4 MB peak RSS; jalur inline di-OOM-kill pada 1,68 GB |
| Kompatibel dengan workflow n8n | ❌ BELUM | tidak ada | Butuh korpus uji + differential testing (§10) |
| Setara jumlah integrasi | ❌ TIDAK | — | Strategi berbeda: subset + codegen (§8.4). n8n = 694 node terverifikasi (PRD-1 §18) |

> **⚠️ Kualifikasi wajib (audit F2).** Angka 50,4 MB diukur pada crate `kernel`
> yang berdiri sendiri — **belum** mencakup HTTP server, SQLite, QuickJS, maupun
> node apa pun. Ia membuktikan *mekanisme spill* hemat memori. Ia **tidak**
> membuktikan produk jadi hemat memori. Target produk adalah **≤ 150 MB** (§6.5)
> dan itu **proyeksi, bukan pengukuran**. Dilarang mengutip 50,4 MB sebagai
> angka produk di materi publik.
>
> **Reproduksi (audit F3):** `BENCH-A03.md` §2 mencatat lingkungan; benchmark ada
> di `rust-n8n-core/crates/kernel/benches/`, git commit `d3bcff0`.

Klaim pertama adalah satu-satunya yang sudah terbukti, **dan hanya pada tingkat
kernel**. Itu yang jadi ujung tombak pemasaran — dengan kualifikasi di atas, bukan tanpa.

---

## 2. Prinsip Desain

Urutan prioritas. Kalau dua prinsip bertabrakan, yang lebih atas menang.

| # | Prinsip | Konsekuensi praktis |
|---|---|---|
| 1 | **Korektness di atas kecepatan** | `InDoubt` untuk side-effect yang tidak pasti; tidak pernah menebak |
| 2 | **Batas memori eksplisit** | Tidak ada struktur data yang tumbuh O(N) tanpa disadari; spill bukan opsional |
| 3 | **Kompatibel n8n di semantik, bukan di bug** | Perilaku n8n yang jelas bug tidak ditiru, tapi dicatat di katalog deviasi. Kriteria operasional di §2.2 |
| 4 | **Satu binary, nol service eksternal untuk mulai** | SQLite default; Redis/Postgres opsional, bukan syarat |
| 5 | **Kontrak sebelum implementasi** | Crate `kernel` **provisional-freeze** (lihat §12 Fase 0); perubahan butuh ADR singkat |
| 6 | **Setiap klaim punya cara ukur** | Tidak ada NFR tanpa metode verifikasi |
| 7 | **Kode tak tepercaya selalu di-sandbox** | Code node & expression tidak pernah menyentuh FS/host langsung |

### 2.1 Yang secara sadar TIDAK kita lakukan

| Tidak melakukan | Alasan |
|---|---|
| Mendukung community node npm | Node npm = kode JS. Mendukungnya berarti menarik seluruh runtime Node.js + dependency tree ke produk — **menghancurkan keunggulan memori** dan keunggulan binary tunggal |
| Meniru 694 node dengan tangan | Tidak realistis solo. Diganti subset + codegen |
| Multi-tenant | Anda memutuskan self-hosted single-tenant (§3.3 PRD induk) |
| Mengganti expression JS dengan DSL sendiri | Akan memecah kompatibilitas. QuickJS adalah satu-satunya jalur realistis |
| Membangun UI dulu | UI ~30% effort; engine harus benar dulu |

### 2.2 Kriteria operasional: "semantik" vs "bug" (audit F5)

Prinsip #3 tidak bisa dijalankan tanpa definisi. Aturan putusannya:

| Kasus | Putusan | Contoh |
|---|---|---|
| Perilaku **terdokumentasi** n8n berbeda dari kita | **DEVIASI** — harus dicatat di katalog, idealnya diperbaiki | `executionOrder` v0 tidak didukung |
| Perilaku n8n **bertentangan dengan dokumentasi n8n sendiri** | **BUG n8n** — kita ikuti dokumentasi, catat sebagai deviasi | — |
| Perilaku n8n **merusak/menghasilkan data salah** | **BUG n8n** — tidak ditiru, catat | kehilangan `pairedItem` saat error |
| Perilaku n8n **tidak terdokumentasi** dan tidak merusak | **TIRU** — itu semantik de-facto, pengguna mungkin bergantung padanya | coercion JS tertentu |

**Pemilik katalog deviasi (§10.3):** satu orang yang ditunjuk eksplisit —
default: pemilik proyek. Merge entri `DEV-xxx` butuh keputusan pemilik, **bukan**
persetujuan penulis kode. Ini sengaja: kegagalan asli proyek ini adalah
recommender sekaligus approver.

---

## 3. Arsitektur Sistem

### 3.1 Topologi target (Fase 1-4)

```
┌──────────────────────────────────────────────────────┐
│                  SATU BINARY RUST                    │
│                                                      │
│  ┌────────┐  ┌──────────┐  ┌─────────────────────┐   │
│  │ HTTP   │  │ Scheduler│  │ Executor            │   │
│  │ axum   │  │ cron     │  │ (tokio, multi-thread│   │
│  │ UI+API │  │ webhook  │  │  work-stealing)     │   │
│  │ webhook│  │          │  │                     │   │
│  └───┬────┘  └────┬─────┘  └──────────┬──────────┘   │
│      │            │                   │              │
│      └────────────┴───────────────────┘              │
│                   │                                  │
│      ┌────────────▼─────────────┐                    │
│      │  kernel (kontrak, beku)  │                    │
│      └────────────┬─────────────┘                    │
│                   │                                  │
│  ┌────────┬───────┼────────┬──────────┬───────────┐  │
│  │ storage│ spill │ expr   │ nodes    │ blobstore │  │
│  │ SQLite │ disk  │QuickJS │ +codegen │ fs/S3     │  │
│  └────────┴───────┴────────┴──────────┴───────────┘  │
└──────────────────────────────────────────────────────┘
```

**Bandingkan dengan n8n queue mode yang butuh 5 service** (main, worker, webhook, PostgreSQL, Redis). Kita mulai dari 1 proses + SQLite.

### 3.2 Struktur crate

```
rust-n8n-core/
├── crates/
│   ├── kernel/          ⚠️ SEBAGIAN — kontrak murni, 0 dep internal, 19/19 test;
│   │                          tapi impl disk ada di examples/ (tak teruji) & gagal SEC-07
│   ├── workflow/           parse/validate n8n JSON → model internal
│   ├── storage/            SQLite (sqlx), migrasi, repository
│   ├── data-plane/         SpillStore disk, BlobStore, codec biner
│   ├── expr/               ExpressionEngine trait impl
│   ├── expr-quickjs/       QuickJS runtime + helper n8n + sandbox
│   ├── scheduler/          cron, task queue, retry, governor
│   ├── executor/           WorkflowExecute equivalent
│   ├── nodes-core/         ~25 node inti
│   ├── nodes-openapi/      hasil codegen
│   ├── openapi-codegen/    generator (build-time, bukan runtime)
│   ├── api-types/          tipe DTO dibagi FE↔BE (padanan @n8n/api-types)
│   ├── api/                REST axum + WebSocket push
│   ├── auth/               session, API key, enkripsi kredensial
│   ├── editor-ui/          frontend (Fase 3) — mungkin bukan Rust
│   ├── testkit/            in-memory impl semua trait kernel
│   └── cli/                binary utama: run/start/worker/import
└── scripts/
    ├── check-freeze.sh  ✅ SELESAI (skrip berfungsi; saat ini exit 1 — lihat §12)
    └── compat-corpus/      differential testing vs n8n nyata
```

### 3.3 Aturan dependensi (di-enforce oleh CI, bukan konvensi)

```
kernel ← {workflow, storage, data-plane, expr, scheduler, executor,
          nodes-*, api, auth, cli}
```

- `kernel` bergantung pada **tidak ada** crate internal
- Tidak ada cycle — diverifikasi otomatis oleh `scripts/check-freeze.sh` §4
- Crate implementasi boleh bergantung satu sama lain hanya lewat trait kernel

**Lokasi registry node (klarifikasi audit F8).** `ParameterSchema` **dan** metode
`validate()` sudah ada di `kernel` (`crates/kernel/src/params.rs:22,44`), jadi
crate `workflow` **tidak** perlu bergantung pada `nodes-core` untuk memvalidasi —
tidak ada cycle yang dipaksa. Yang perlu ditegaskan:

- `NodeRegistry` = **trait di `kernel`**; implementasi konkret di `cli` (composition root)
- `workflow` memvalidasi terhadap `ParameterSchema` yang **diterima sebagai input**, bukan yang di-import
- Registrasi node terjadi di `cli` saat startup → arah dependensi tetap `cli → {workflow, nodes-*}`, tidak pernah `workflow → nodes-*`

Tanpa klarifikasi ini, implementasi Fase 1 cenderung menarik `nodes-core` ke
dalam `workflow` dan baru ketahuan saat `check-freeze.sh` menolak di tengah fase.

> **Kenapa ini penting:** n8n mencapai graf acyclic lewat disiplin konvensi di
> antara banyak kontributor. Kita mencapainya lewat enforcement kompilator + CI.
> Untuk proyek solo, disiplin konvensi akan bocor; kompilator tidak.

### 3.4 Status kernel hari ini

| | |
|---|---|
| Baris kode | 2.644 (src) + 833 (test) + 653 (benchmark) |
| Test | 19/19 hijau |
| Clippy | 0 warning |
| `unsafe` | 0 (`#![forbid(unsafe_code)]`) |
| Dependency eksternal | 4: serde, serde_json, async-trait, thiserror |
| Dependency internal | 0 |

**Utang yang harus dibereskan:** workspace `Cargo.toml` mereferensikan 4 crate tapi hanya 2 manifest ada → `cargo build` di level workspace gagal. Tiga direktori stub (`data-plane`, `testkit`, `nodes-core`) kosong. **Ini blocker pertama Fase 1.**

---

## 4. Pilihan Teknologi

| Layer | n8n | Kita | Alasan |
|---|---|---|---|
| Bahasa engine | TypeScript/Node.js ≥22 | **Rust stable, edition 2024, MSRV 1.85** | Kontrol memori, tanpa GC, tanpa heap JS. Edition 2024 sudah stable; toolchain terpasang 1.98.1. MSRV dinyatakan eksplisit agar CI bisa menegakkannya (audit F11) |
| Async runtime | Node event loop (single-thread) | **tokio** (multi-thread work-stealing) | Paralelisme nyata dalam satu proses |
| HTTP server | Express 5.1 | **axum** | Terintegrasi tokio, tower middleware |
| Database | TypeORM + SQLite/PostgreSQL | **sqlx** (compile-time checked query) + SQLite | Query diverifikasi saat compile; tanpa ORM overhead |
| Migrasi | TypeORM migrations | **sqlx migrate** atau **refinery** | — |
| Job queue | Bull + Redis | **TIDAK ADA di MVP.** In-process scheduler + SQLite | Prinsip #4. Queue eksternal ditambahkan hanya jika terbukti perlu |
| Expression engine | `@n8n/expression-runtime` (JS sandbox) | **QuickJS** via `rquickjs` | Satu-satunya cara kompatibel semantik JS |
| Code node JS | task-runner (proses Node terpisah) | **QuickJS** (sama, sandbox lebih ketat) | Tanpa dependency Node.js |
| Code node Python | task-runner Python | **Fase lanjut / mungkin tidak ada** | Butuh runtime Python terpisah; evaluasi ulang |
| Serialisasi | JSON | **serde_json** + **postcard** untuk spill | ⚠️ **BELUM TERVERIFIKASI.** Klaim "~3x lebih kecil, ~5x lebih cepat" tidak punya sumber dan **melanggar prinsip #6 dokumen ini**. Wajib diukur di spike Fase 1 sebelum dipakai sebagai alasan keputusan (audit F15) |
| Kompresi spill | — | **zstd** (opsional, level rendah) | Trade CPU vs disk |
| Crypto | Node crypto AES-256 | **ring** atau **aes-gcm** | Enkripsi kredensial at-rest |
| Cron | node-cron | **cron** + **chrono-tz** | Timezone IANA |
| Frontend | Vue 3 + Pinia + Vue Flow + Element Plus | **PUTUSAN TERBUKA — lihat §9** | ~30% effort total |
| WebSocket push | ws | **tokio-tungstenite** / axum ws | Live execution viewer |
| Observability | Prometheus client | **metrics** + **tracing** | — |
| Error handling | `@n8n/errors` | **thiserror** (sudah di kernel) | — |
| Config | `@n8n/config` + Zod | **serde** + **figment**/`config` + validasi eksplisit | — |

### 4.1 Kenapa sqlx, bukan ORM

TypeORM memberi n8n fleksibilitas lintas DB tapi membayar dengan query yang baru gagal saat runtime. `sqlx` memverifikasi SQL terhadap skema nyata **saat compile**. Untuk proyek solo tanpa tim QA, memindahkan kelas bug dari runtime ke compile time adalah trade yang jelas menguntungkan.

Konsekuensi: dukungan multi-DB (Postgres) jadi lebih mahal. **Rekomendasi: SQLite dulu, Postgres nanti.**

**Konsekuensi CI yang wajib diputuskan di muka (audit F12).** Verifikasi
compile-time `sqlx` membutuhkan `DATABASE_URL` yang hidup **saat build**. Tanpa
strategi, CI akan gagal atau — lebih buruk — developer mematikan pemeriksaan itu.
Keputusan: pakai **`.sqlx` offline cache yang di-commit** (`cargo sqlx prepare`),
sehingga build tidak butuh DB hidup. Trade-off: cache bisa basi terhadap skema,
jadi CI wajib menjalankan satu job `--offline` **dan** satu job dengan DB nyata
untuk mendeteksi cache yang kedaluwarsa.

### 4.2 Kenapa QuickJS, bukan menulis expression engine sendiri

Expression n8n adalah JavaScript nyata dengan puluhan helper dan perilaku edge yang tidak terdokumentasi lengkap (PRD-1 §8). Menulis DSL sendiri berarti:
- Setiap workflow n8n yang memakai fitur JS di luar DSL kita akan gagal
- Kita harus memelihara kompatibilitas JS selamanya

QuickJS memberi semantik JS lengkap dalam ~700 KB, embeddable, deterministic. Biayanya: butuh FFI (satu-satunya tempat `unsafe` masuk — di crate terpisah, bukan di kernel).

---

## 5. Model Data & Penyimpanan

### 5.1 Prinsip pemisahan

Kesalahan struktural n8n: **payload dan metadata disatukan dalam satu blob JSON** di `execution_entity.data` (PRD-1 §12.4). Ini penyebab bloat Postgres dan tekanan memori.

Kita pisahkan tiga hal:

| Data | Tempat | Siklus hidup |
|---|---|---|
| **Definisi workflow** | SQLite `workflow` | Permanen, versioned |
| **Metadata eksekusi** (status, waktu, node mana jalan) | SQLite `execution` + `event_log` | Permanen s/d retention |
| **Payload item** | **Spill file di disk**, direferensikan `SpillPath` | Sementara, GC setelah eksekusi terminal |
| **Binary/file** | BlobStore (fs atau S3) | Sesuai konfigurasi |

Konsekuensi: database tetap kecil walau payload besar. Pruning tidak lagi berarti membuang data — ia berarti menghapus file spill yang sudah tidak dirujuk.

### 5.2 Skema inti (draf)

```sql
workflow(id, name, active, version, nodes_json, connections_json,
         settings_json, static_data, created_at, updated_at)
workflow_version(workflow_id, version, snapshot_json, created_at)

execution(id, workflow_id, workflow_version, mode, status,
          started_at, finished_at, error_code, error_message)

-- Append-only. Ini pengganti blob JSON n8n.
event_log(execution_id, seq, ts, schema_version, event_json)
          PRIMARY KEY(execution_id, seq)

task(id, execution_id, node_id, input_index, attempt, priority,
     status, hints_json, lease_holder, lease_expires_at,
     created_at, started_at, finished_at, error_json, wake_at)

checkpoint(id, execution_id, schema_version, completed_ids,
           in_flight_ids, waiting_json, event_seq, created_at)

-- Referensi ke spill file, BUKAN payload.
node_output(execution_id, node_id, output_index,
            storage_kind, spill_path, item_count, total_bytes)

-- audit F20: enc_key_id + enc_algo WAJIB sejak migrasi 1.
-- PRD-1 §11.4 mencatat n8n sudah pernah terluka di rotasi/legacy key
-- (commit 44d9d90, 4b36c51). Tanpa kolom ini, rotasi key membuat SEMUA
-- credential tak bisa didekripsi dan tidak ada jalur re-encrypt bertahap.
credential(id, name, type, data_encrypted,
           enc_key_id, enc_algo,          -- <-- TAMBAHAN
           created_at, updated_at)
encryption_key(id, algorithm, created_at, revoked_at)  -- riwayat key
variable(key, value, type)
spill_gc(spill_path, execution_id, ref_count, created_at)  -- untuk A-21

-- audit F19: catat INTENT lebih dulu, baru tulis file.
-- Tanpa ini, crash di antara "file tertulis" dan "baris DB tercatat"
-- menghasilkan file yatim yang TIDAK PERNAH ditemukan mark-sweep
-- (mark-sweep berjalan dari baris DB; file tanpa baris tak terlihat).
spill_intent(spill_path, execution_id, node_id, created_at, committed)
```

> **`event_log.schema_version`** menyelesaikan audit A-16: tanpa ini, upgrade
> engine membuat log lama tidak bisa di-replay dan merusak crash recovery untuk
> eksekusi yang sedang berjalan saat upgrade.

> **Rotasi `event_log` (audit F18, dikoreksi).** Volume realistisnya **puluhan
> sampai ratusan baris per execution** (transisi task + node-run), bukan
> "puluhan juta per execution" seperti klaim awal audit — meleset ~5-6 orde.
> Tapi kekhawatiran dasarnya sah: satu tabel append-only yang tak pernah
> dirotasi tetap tumbuh tanpa batas, dan `VACUUM` di tabel besar mahal.
> Keputusan: rotasi/partisi per rentang waktu + **uji pada 10 juta baris
> kumulatif** di Fase 2.

> **Orphan sweeper (audit F19).** Selain write-ahead intent di atas, wajib ada
> pemindai direktori spill yang membandingkan isi disk dengan DB dan menghapus
> file yang tidak dirujuk. Gate "nol leak setelah 1.000 eksekusi" **tidak
> cukup** — harus ada uji `kill -9` tepat di tengah penulisan spill.

### 5.3 Retensi

| Data | Default | Bisa dikonfigurasi |
|---|---|---|
| Execution metadata | 30 hari | ya |
| Event log | mengikuti execution | ya |
| Spill file | **dihapus saat execution terminal** | ya (tahan untuk debugging) |
| Binary/blob | mengikuti konfigurasi | ya |

> **⚠️ Kontradiksi yang harus dipecahkan (audit F21).** Default di atas
> menghapus spill saat execution terminal — tapi differential testing (§10.2)
> membandingkan output **setelah** eksekusi selesai. Dengan default ini, harness
> uji **secara desain tidak bisa bekerja**: outputnya sudah terhapus sebelum
> dibandingkan. Karena itu wajib ada mode **`retain-outputs`**:
>
> | Mode | Perilaku | Dipakai untuk |
> |---|---|---|
> | `default` | spill dihapus saat terminal | produksi |
> | `retain-outputs` | spill **ditahan** sampai dihapus eksplisit | korpus differential testing, debugging |
> | `retain-failed` | spill ditahan hanya untuk execution gagal | triase insiden |
>
> Mode ini prasyarat Fase 1, bukan fitur tambahan.

---

## 6. Execution Engine

### 6.1 Tiga lapisan (sudah ada di kernel)

```
WorkflowExecution   "workflow #100 sedang jalan"     ← container akar
      └── NodeExecution  "node HTTP #5 sedang jalan"  ← record domain
             └── Task      "attempt 1 dari node #5"    ← primitif scheduling
```

Pemisahan ini yang memungkinkan pindah dari 1 proses ke N worker **tanpa mengubah model eksekusi**. n8n harus menambah seluruh subsistem `scaling/` untuk queue mode; kita cukup mengganti implementasi queue di belakang trait yang sama.

### 6.1.1 Konfigurasi runtime yang eksplisit (audit F7)

PRD ini menyebut "tokio multi-thread work-stealing" tanpa satu pun angka. Itu
celah nyata: **thread × chunk_size × item_size adalah perkalian yang bisa
menjebol budget governor**, dan governor tidak akan melihatnya kalau ia hanya
menghitung per-task.

| Parameter | Default | Batas |
|---|---|---|
| `worker_threads` | `min(2, nproc)` | ≤ 4 pada mesin 2 GB |
| `max_concurrent_tasks` | 8 | admission control, bukan sekadar antrian |
| `chunk_size` (item per batch) | 1.000 | sama seperti benchmark |
| Budget memori efektif | `worker_threads × chunk_size × item_size` | **harus** < budget governor |

Rumus pengikat: `worker_threads × max_concurrent_tasks × chunk_size × item_size
≤ memory_budget`. Kalau dilanggar, yang dikurangi adalah `max_concurrent_tasks`,
**bukan** budget.

**Wajib diuji:** governor pada `worker_threads` maksimum, bukan pada 1 thread.
Benchmark BENCH-A03 dijalankan **single-threaded** — jadi angka 50,4 MB **tidak**
berlaku untuk runtime multi-thread penuh. Ini alasan tambahan kenapa §1
memakai kualifikasi "tingkat kernel".

### 6.2 Siklus eksekusi

```
1. Trigger → buat Execution (status=RUNNING) + event ExecutionCreated
2. Planner: hitung Task untuk node yang dependency-nya terpenuhi
3. Scheduler: urutkan berdasar priority (D56) + admission control
       [D56 = "Adaptive priority", terdefinisi di KERNEL-SPEC.md:463
        (sudah diimplementasikan sebagai `pub priority: i8`) dan
        AUDIT-D1-D100.md:331; diringkas di rangkuman-rust-workflow-os.md:495]
4. Executor (tokio task):
     a. [Fase 5] Lease task (worker_id + expires_at)
        Fase 1: LEWATI - lihat catatan di bawah
     b. Bangun NodeContext (input view, prior outputs, services)
     c. Resolusi parameter → eval expression via QuickJS
     d. Panggil node.execute()
     e. Hasil → ItemList::from_vec / spill_from_iter (policy governor)
     f. Tulis node_output (referensi spill, bukan payload)
     g. Append event TaskCompleted
     h. Release lease
5. Ulangi 2-4 sampai tidak ada Task Ready
6. Status akhir via execution_status() → Success/Failed/NeedsReview
7. GC spill file untuk execution terminal
```

> **Lease di Fase 1 (audit F6, diterima sebagian).** Di topologi single-process,
> `lease_holder` redundan: saat restart, **semua** task berstatus `Running` pasti
> milik proses yang baru mati, jadi tidak perlu identifikasi pemegang. Aturan
> recovery Fase 1: **restart ⇒ semua `Running` = curiga**, lalu diproses lewat
> `recovery_action()` (§6.3) berdasarkan `SideEffect`-nya.
>
> Field `lease` **tetap ada di kernel** dan **tidak dihapus**. Alasannya: ia
> sudah `Option<Lease>` (`task.rs:44`) sehingga biayanya nol saat tidak dipakai,
> dan menghapusnya berarti memecah kontrak yang sudah teruji demi penghematan
> yang tidak terukur. Fase 1 cukup **tidak mengisinya**. Field ini baru dipakai
> di Fase 5 (distributed workers).

### 6.3 Crash recovery

Sudah dirancang di kernel (`Task::recovery_action()`):

| Status saat crash | Side effect node | Aksi |
|---|---|---|
| Pending/Ready | — | Requeue |
| Running | `None` | Requeue |
| Running | `Idempotent` | Requeue **dengan idempotency key** |
| Running | `NonIdempotent` | **`InDoubt`** → execution `NEEDS_REVIEW` → alert |
| Waiting | — | Restore wait dari `wake_at` |
| Terminal | — | Noop |

**Ini perbaikan nyata atas n8n** (PRD-1 §17.5). n8n tidak bisa membedakan "side effect sudah terjadi" dari "belum" — kita bisa, dan kita menolak menebak.

### 6.4 Memory Governor

| Level | Ambang | Aksi |
|---|---|---|
| `Normal` | < 60% budget | Tidak ada |
| `Pressure` | 60–85% | Turunkan `max_inline_items`, paksa spill list baru |
| `Critical` | > 85% | Spill list inline terbesar, tolak admission task baru, tunda yang bisa ditunda |

Budget default: **50% dari RAM terdeteksi**, dengan cap absolut. Di mesin 2 GB → budget ~1 GB, sisanya untuk OS dan headroom.

Governor membaca `ItemList::ram_footprint()` yang sudah ada di kernel — angka itu dirancang untuk ini.

### 6.5 Konsumsi memori target

| Komponen | Budget |
|---|---|
| Binary idle | ≤ 60 MB |
| Per execution aktif (metadata + context) | ≤ 5 MB |
| Per task berjalan | ≤ chunk_size × item_size |
| Spill index | 8 byte/item (terukur: 38,1 MB untuk 5 juta item) |

**Terukur saat ini:** 5 juta item → 50,4 MB peak RSS *(tingkat kernel, single-thread — lihat §1 dan §6.1.1)*. Target engine penuh: **≤ 150 MB untuk workload yang sama** (proyeksi, naik karena ada HTTP server, SQLite, QuickJS).

#### ⚠️ Index spill adalah komponen memori DOMINAN pada N besar (audit F4)

Angka 8 byte/item terdengar kecil. Ia tidak kecil. Dihitung ulang dari BENCH-A03:

| N | Index | Peak RSS | Index sebagai % peak |
|---|---|---|---|
| 200.000 | 1,5 MB | 14,0 MB | 11% |
| 1.000.000 | 7,6 MB | 20,0 MB | 38% |
| 5.000.000 | 38,1 MB | 50,4 MB | **76%** |

Proyeksi linear 8 byte/item:

| N | Index saja | |
|---|---|---|
| 10.000.000 | 76,3 MB | masih aman |
| 50.000.000 | 381,5 MB | mulai memakan budget |
| 100.000.000 | **762,9 MB** | **menghancurkan tesis "jalan di VPS 2 GB"** |

Artinya: pada N besar, **yang habis lebih dulu bukan payload — melainkan index
kita sendiri.** Payload sudah di disk; index masih di RAM.

Ini **bukan** pelanggaran prinsip #2 seperti klaim audit — prinsip itu berbunyi
"tumbuh O(N) **tanpa disadari**", dan pertumbuhan ini dinyatakan eksplisit di
tabel atas. Tapi ia **batas yang belum berpagar**, dan itu kesalahan yang sama
parahnya.

**Pagar yang wajib ada:**

| Budget RAM | N-maks (index ≤ 25% budget) |
|---|---|
| 512 MB | ~16 juta item |
| 1 GB | ~32 juta item |
| 2 GB | ~65 juta item |

Di atas N-maks: **wajib index berjenjang di disk** (sparse index di RAM, offset
per-item di disk, dibaca saat dibutuhkan) — bukan menambah RAM. Kalau fitur itu
belum dibangun, engine harus **menolak** execution yang melampaui N-maks dengan
pesan eksplisit, bukan diam-diam menukar memori.

**Status: index berjenjang BELUM dirancang.** Masuk backlog Fase 1 sebagai
keputusan T-9.

---

## 7. Node System

### 7.1 Trait (sudah ada di kernel)

```rust
#[async_trait]
pub trait Node: Send + Sync {
    fn descriptor(&self) -> &NodeDescriptor;
    async fn execute(&self, ctx: &mut NodeContext<'_>) -> Result<NodeOutput, NodeError>;
}
```

`NodeDescriptor` memuat: `kind`, `version`, `display_name`, `group`, `hints` (ResourceHint), `params` (ParameterSchema), `credentials`, `inputs`, `outputs`, `execute_once`.

### 7.2 ResourceHint — yang tidak dimiliki n8n

```rust
pub struct ResourceHint {
    pub buffering: BufferingMode,      // Batch (default) | Streaming
    pub side_effect: SideEffect,       // None | Idempotent | NonIdempotent
    pub weight: Weight,                // estimasi CPU/memori
    pub max_concurrency: Option<u16>,  // rate limiting per node type
}
```

Ini yang memungkinkan:
- Scheduler membuat keputusan admission tanpa menjalankan node
- Recovery tahu apakah aman retry (A-10)
- Governor tahu node mana yang akan rakus memori

**Node author mendeklarasikan, engine memutuskan.** Node tidak pernah memilih untuk spill sendiri.

### 7.3 Daftar node MVP — direvisi berbasis korpus 171 workflow

> ⚠️ **OPEN-3 / T-13 / T-14: daftar final butuh keputusan Anda.**
>
> **Koreksi kejujuran:** versi v1.2 bagian ini mengklaim daftar "diturunkan dari
> node yang paling sering muncul di workflow nyata". **Klaim itu tidak benar
> saat ditulis** — daftar itu diturunkan dari penalaran tentang node apa yang
> umumnya penting. Sekarang klaim itu bisa diuji, dan hasilnya tidak mendukung
> optimisme dokumen ini. Analisis lengkap: `ANALISIS-KORPUS-NODE.md`
> (reproducible via `probe6/analisis_node.py`, deterministik, tanpa network).

**Korpus:** 171 template publik n8n.io (10 sampel terverifikasi byte-identik
dengan API n8n.io — `VERIFIKASI-DELIVERABLE-TIM.md` §5). 1.810 instance node,
185 tipe unik.

#### 7.3.1 Temuan yang mengubah roadmap

**1. Kurva marginal datar — tidak ada shortcut.** Dengan 26 node, hanya
**23/171 workflow (13%)** jalan penuh. 54 workflow lain dihalangi oleh **42
tipe node berbeda**, dan **35 dari 42 (83%) hanya menutup satu workflow
masing-masing**. Tidak ada node tunggal yang membuka seperempat korpus.

| Target coverage | Node yang harus diimplementasi |
|---:|---:|
| 20% | 28 |
| 30% | 40 |
| 45% | 59 |
| 60% | 85 |
| 75% | 111 |
| **90%** | **153** |

n8n punya 694 node (terverifikasi, §18.4 PRD-1). Jadi 153 node ≈ 22% katalog
n8n hanya untuk menjalankan 90% dari **171 template sampel** — dan n8n terus
menambah node, jadi targetnya bergerak.

**2. Kesenjangan instance vs workflow.** 26 node mencakup **63% instance** node
sejati tapi hanya **13% workflow** jalan penuh. Selisih 50 poin itu harga long-tail.
**Jangan pernah mengutip "63% node didukung" sebagai indikator kesiapan** —
angka yang jujur adalah 13% workflow.

**3. `stickyNote` wajib bisa di-parse.** Muncul di **79/171 file (46%)** — tipe
dengan jumlah file tertinggi di korpus. Ia anotasi kanvas, bukan node eksekusi:
tidak punya `execute()`, tidak masuk governor, tidak ikut topological sort.
Tapi parser **harus** mengenalnya dan menyimpan `content`/`position`/`color`
supaya bisa dirender ulang. Bila parser menolak tipe tak dikenal, hampir
separuh korpus gagal impor karena **catatan tempel** — bug yang tidak akan
pernah muncul di diskusi "node apa yang harus diimplementasi".

**4. Ada dua pasar, dan biayanya hampir sama.**

| Pasar | Workflow | Node unik | Node untuk 90% |
|---|---:|---:|---:|
| AI / LangChain | 39 (23%) | 101 | **98** |
| non-AI | 132 (77%) | 127 | **106** |

58 node eksklusif AI. Artinya **fokus ke pasar AI tidak menghindari masalah
long-tail** — ia hanya memindahkannya ke subtipe node lain. Workflow AI n8n
bukan "workflow biasa plus satu node LLM"; ia memakai seluruh subsistem
LangChain (`agent`, `lmChatOpenAi`, `outputParserStructured`,
`memoryBufferWindow`, `chainLlm`, `documentDefaultDataLoader`, vector store,
reranker, splitter). Tidak ada jalur "fokus AI supaya cepat selesai".

**5. Greedy murni menyesatkan.** Greedy max-coverage memberi 21% (vs 13%),
tapi set pilihannya memuat `bitwarden`/`bubble`/`copper`/`interval` yang
masing-masing muncul di 1–2 file — **overfitting ke korpus 171 template**. Lebih
parah, greedy melewati `scheduleTrigger` (13% workflow) karena gain
marginalnya nol pada iterasi itu. Padahal tanpa `scheduleTrigger` **tidak ada
otomasi terjadwal sama sekali**. Greedy mengoptimalkan "jumlah workflow
lengkap", bukan "kapabilitas produk" — dan metrik pertama menipu.

#### 7.3.2 Daftar 26 node (revisi)

Metodologi: *capability-critical* (wajib, apapun gain marginalnya) → frekuensi
→ ambang anti-overfit (buang node ≤2 file kecuali capability-critical).
**26 slot = 23 implementasi nyata + 3 alias registry.**

| Node | Alasan |
|---|---|
| `manualTrigger` | entry point dasar |
| `scheduleTrigger` | otomasi terjadwal — tanpanya tidak ada cron |
| `webhook` | otomasi event-driven |
| `respondToWebhook` | pasangan wajib webhook |
| `httpRequest` | pintu ke semua API eksternal (**universal**) |
| `code` | escape hatch (**universal**) |
| `set` | transform dasar |
| `if` | branching |
| `switch` | multi-branch |
| `merge` | gabung branch |
| `filter` | seleksi item |
| `limit` | paginasi |
| `splitOut` | ekspansi array |
| `splitInBatches` | batching (11 file) |
| `wait` | delay / retry / rate-limit |
| `noOp` | placeholder |
| `errorTrigger` | error workflow |
| `executeWorkflow` | sub-workflow |
| `googleSheets` | 29 file |
| `googleDrive` | 9 file |
| `spreadsheetFile` | 9 file |
| `slack` | 15 file |
| `telegram` | 16 file |
| `cron` | **ALIAS** → `scheduleTrigger` |
| `function` | **ALIAS** → `code` |
| `functionItem` | **ALIAS** → `code` |

**Dibuang dari v1.2:** `agent`, `openAi`, `outputParserStructured` (masuk
subsistem LangChain, butuh ~98 node — tidak bisa "sedikit-sedikit"),
`aggregate`, `removeDuplicates`, `sort`, `executeCommand` (frekuensi 1–7 file,
bukan capability-critical).

**Ditambah:** `cron`/`function`/`functionItem` (alias, menutup 15% korpus),
`googleDrive`, `splitInBatches`, `spreadsheetFile`.

Coverage set ini: **13% workflow jalan penuh, 63% instance node sejati**. Set v1.2:
11% pada pasar non-AI. Perbaikan nyata tapi kecil — **karena masalahnya bukan
pemilihan node, masalahnya long-tail.**

#### 7.3.3 Node deprecated — bukti untuk T-12

| Node | Workflow memuat | % | Bila ditolak |
|---|---:|---:|---|
| `cron` | 13 | 8% | gagal impor |
| `function` | 12 | 7% | gagal impor |
| `functionItem` | 9 | 5% | gagal impor |
| **Total (≥1 deprecated)** | **26** | **15%** | gagal impor |

Pada sampel 13 template sebelumnya `cron` terlihat langka (2x). Pada 171
template ternyata **13 workflow** — setara `splitOut` dan `switch`. **Ini
mengubah bobot T-12 dari "nice to have" menjadi hampir wajib.**

Biayanya kecil: `cron` → `scheduleTrigger` dan `function` → `code` adalah
**pemetaan registry**, bukan implementasi node baru.

> **Deviasi yang wajib dicatat di §10.3:** `function` di n8n lama bisa
> `require()` paket npm eksternal. Alias ke `code` **tidak** memberi kemampuan
> itu, dan sandbox V8 di desain kita memang melarangnya. Jadi alias harus
> **gagal eksplisit dengan pesan jelas** bila mendeteksi `require()` — bukan
> diam-diam berjalan dengan perilaku berbeda. Semantik akses `items` juga
> berbeda antara `function` dan `code`.

#### 7.3.4 Pisahkan kompatibilitas impor dari kapabilitas eksekusi

Karena paritas node tidak terjangkau (§7.3.1), rekomendasi teknisnya:

1. **Parser menerima SEMUA 694 tipe node** tanpa gagal. Yang tidak dikenal
   disimpan sebagai node *opaque* + peringatan, **bukan** menolak impor.
   Biayanya kecil (tidak ada `execute()`), manfaatnya besar: tidak ada workflow
   yang "rusak" saat dibuka.
2. **Eksekusi didukung bertahap** sesuai daftar §7.3.2.
3. **`httpRequest` + `code` paling awal** — keduanya universal. Setiap API bisa
   dicapai dengan `httpRequest`, setiap transformasi dengan `code`. Node khusus
   (`googleSheets`, `slack`, ...) adalah **kemudahan**, bukan **kapabilitas**.
4. **Generator kredensial** (dari skema JSON n8n) lebih berharga daripada node
   ke-50, karena ia membuka `httpRequest` ke semua layanan berautentikasi.

**T-14 (§16)** menanyakan janji kompatibilitas produk ini kepada Anda, karena
temuan di atas membuat janji "kompatibel dengan n8n" tidak bisa dipenuhi dalam
waktu dekat.

### 7.4 Codegen OpenAPI — strategi menutup jarak integrasi

**Masalah:** menulis node dengan tangan ~1-3 hari per integrasi (OAuth2 saja bisa seharian). 400 node = tidak realistis solo.

**Solusi:** generate node dari OpenAPI spec.

```
OpenAPI spec (publik, ribuan tersedia)
        ↓  openapi-codegen (build-time)
   NodeDescriptor { kind, params: ParameterSchema, credentials }
        +
   impl Node for GeneratedNode { execute → HTTP call deklaratif }
```

**Kenapa ini mungkin:** n8n sendiri sudah punya *declarative HTTP routing* (PRD-1 §7.3) — parameter dipetakan ke request tanpa kode. Itu persis bentuk output yang bisa kita generate.

**Skala target yang jujur (terverifikasi PRD-1 §18):** n8n punya 694 node dan **445 credential type**. Credential — bukan node — adalah pekerjaan beratnya, karena 445 tipe itu masing-masing butuh skema + alur auth (OAuth2 paling mahal). Codegen OpenAPI menutup sisi node relatif murah; sisi credential tetap mahal dan tidak bisa digenerate dari OpenAPI spec karena spec tidak memuat alur OAuth.

**Keunggulan atas n8n:** node hasil generate kita adalah **kode Rust terkompilasi**, bukan metadata yang diinterpretasi saat runtime. Zero overhead JS.

**Tantangan nyata (R8 di PRD induk):** kualitas OpenAPI spec publik sangat bervariasi — banyak yang tidak lengkap, ambigu, atau salah. Codegen yang menghasilkan node rusak **lebih buruk daripada tidak ada node**.

**Mitigasi:**
1. Mulai dari 10-20 spec berkualitas tinggi (Stripe, GitHub, Slack punya spec bagus), bukan 1.000 spec acak
2. Setiap node hasil generate harus lolos test terhadap API nyata atau mock
3. Kurasi manual — ada daftar "verified nodes", bukan "generated nodes"
4. **Jangan klaim jumlah.** 50 node terverifikasi > 2.000 node rusak

**Jadwal:** prototipe di Fase 2, produksi di Fase 5.

---

## 8. Expression Engine

### 8.1 Cakupan yang harus didukung

Dari PRD-1 §8.2 — semua ini wajib untuk kompatibilitas:

`$json`, `$input.all/first/item`, `$binary`, `$('Node')`, `$items()`, `$node[]`,
`$now`, `$today`, `$execution.*`, `$workflow.*`, `$itemIndex`, `$vars`, `$env`,
`$runIndex`, `$prevNode`

Plus helper: `$jmespath`, `$parseJson`, `$toDateTime`, `$convertDateTime`,
`$isEmailValid`, `$isValidJSON`, `$base64Encode/Decode`, `$urlEncode/Decode`,
`$hash`, `$roundTo`, `$randomInt`, `$uuid`, `$randItem`, `$shuffle`,
`$difference`, `$intersection`, `$merge`, `$ifEmpty`, `$ifNot`, `$isEmpty`, `$not`

Plus: **seluruh Luxon DateTime API** dan fungsi bawaan JS.

### 8.2 Arsitektur

```
ExpressionEngine (trait di kernel — sudah ada)
        ↓
expr-quickjs (impl)
   ├── QuickJS runtime, 1 context di-reuse per batch
   ├── Injection scope: $json, $input, $now, ...
   ├── PriorOutputs bridge → $('Node') baca dari spill
   ├── Helper functions (Rust, terdaftar ke JS)
   └── Sandbox: no FS, no net, no process, memory limit, timeout
```

**Kritis untuk performa:** `eval_batch` sudah ada di trait kernel. Setup context QuickJS ratusan mikrodetik — **tidak boleh dibayar per ekspresi per item**. Untuk 1 juta item × 5 ekspresi, itu perbedaan antara 0,5 detik dan ratusan detik.

### 8.3 `$env` — keamanan

n8n gate `$env` dengan allowlist. Kita tiru **lebih ketat**: default **kosong**, harus eksplisit di konfigurasi. `CredentialValue` di kernel sudah punya redaction Debug/Display — pola yang sama berlaku untuk env.

### 8.4 Risiko terbesar di seluruh proyek

**⚠️ R3 di PRD induk.** Expression adalah tempat klaim "kompatibel" paling sering mati:

1. Ini JavaScript nyata — setiap perilaku JS adalah kontrak (coercion, `null` vs `undefined`, formatting tanggal, regex flavor)
2. Helper tidak terdokumentasi lengkap — perilaku edge hanya diketahui dari kode sumber n8n
3. QuickJS ≠ V8. Ada perbedaan: tidak ada JIT, beberapa API modern hilang, formatting angka/tanggal bisa beda

**Mitigasi wajib:** differential testing (§10.2) sejak Fase 1, bukan Fase 3.

---

## 9. Frontend — keputusan terbesar yang belum dibuat

> ⚠️ **OPEN-6 di PRD induk. Ini kemungkinan keputusan dengan leverage terbesar
> di seluruh proyek.**

### 9.1 Tiga opsi

| Opsi | Deskripsi | Effort | Risiko |
|---|---|---|---|
| **A. Headless dulu** | CLI + REST API + dokumentasi baik. JSON untuk definisi workflow. Rilis publik tanpa UI. | ~0 | Adopsi terbatas ke pengguna teknis |
| **B. UI minimal sendiri** | Canvas sederhana (Vue Flow atau library lain), form dari `ParameterSchema`, execution viewer dasar | 4-8 bulan | Underestimate — PRD-1 §10.5 |
| **C. UI lengkap setara n8n** | NDV penuh, expression editor dengan autocomplete, resource locator, pin data, undo/redo, i18n | 12-24 bulan | Hampir pasti gagal solo |

### 9.2 Rekomendasi teknis (bukan keputusan)

**Opsi A untuk rilis pertama, lalu evaluasi.**

Alasannya:
1. **Gate validasi pasar (PRD induk §12.2) bisa diuji tanpa UI.** Kalau tidak ada yang peduli pada engine headless yang hemat RAM, UI tidak akan mengubah itu. Menemukan ini setelah 4 bulan, bukan setelah 18 bulan.
2. Persona P1 (hobbyist teknis) nyaman dengan JSON + API + dokumentasi.
3. `ParameterSchema` di kernel **sudah dirancang** untuk membangkitkan form — jadi UI nanti bisa dibangun di atas fondasi yang sudah ada, bukan dimulai dari nol.
4. Opsi B tetap mungkin nanti tanpa membuang pekerjaan A.

### 9.3 Kalau opsi B dipilih

**Jangan tulis canvas dari nol.** Pakai library graph yang ada (Vue Flow dipakai n8n sendiri; alternatif: React Flow, Cytoscape.js, atau Svelte-based).

Stack yang masuk akal: **Vue 3 + Vite + Pinia + Vue Flow** — sama seperti n8n, karena:
- Pola yang sudah terbukti untuk masalah ini
- `ParameterSchema` → form mapping bisa meniru pola `INodeProperties` → NDV
- Lebih mudah merekrut/minta bantuan karena stack umum

**Konsekuensi yang harus diakui:** ini berarti proyek punya **dua bahasa** (Rust + TypeScript). Untuk solo developer itu biaya konteks yang nyata.

---

## 10. Strategi Kompatibilitas

### 10.1 Tiga tingkat kompatibilitas

| Tingkat | Arti | Target |
|---|---|---|
| **L1 — Format** | JSON workflow n8n bisa diparse & disimpan | 100% (parser toleran) |
| **L2 — Eksekusi** | Workflow yang hanya memakai node didukung menghasilkan output identik | ≥95% dari korpus |
| **L3 — Expression** | Semua expression n8n terevaluasi identik | ≥90% dari korpus |

Workflow yang memakai node tidak didukung harus **gagal impor dengan pesan eksplisit yang menyebut node mana** — bukan diam-diam salah.

### 10.2 Differential testing — wajib dibangun di Fase 1

```
korpus/*.json  (workflow n8n nyata)
      │
      ├──▶ jalankan di n8n asli (Docker) ──▶ output A
      │
      └──▶ jalankan di engine kita       ──▶ output B
                        │
                   bandingkan item-per-item
                        │
              ┌─────────┴──────────┐
         identik              berbeda
                                 │
                        katalog deviasi (§10.3)
```

**Syarat korpus:** minimal 50 workflow n8n **nyata** — dari template publik n8n, workflow Anda sendiri, dan kontribusi. **Bukan ditulis tangan untuk lulus.** Korpus yang ditulis untuk lulus tidak menguji apa pun.

### 10.3 Katalog deviasi — dokumen publik

Setiap perbedaan yang disengaja dicatat dan **dipublikasikan**:

```markdown
## DEV-001: Node X tidak didukung
Alasan: butuh runtime Python
Dampak: workflow memakai node ini gagal impor dengan pesan eksplisit

## DEV-004: Perbedaan coercion pada expression
n8n: {{ 0 == "" }} → true (V8)
Kita: {{ 0 == "" }} → true (QuickJS) — SAMA, terverifikasi

## DEV-011: Bug n8n yang sengaja tidak ditiru
n8n: [deskripsi perilaku]
Kita: [perilaku benar]
Alasan: [penjelasan]
```

Dokumen ini yang membuat klaim kompatibilitas bisa dipercaya. Tanpa itu, "kompatibel" adalah klaim kosong.

---

## 11. Keamanan

| Area | Desain |
|---|---|
| **Kredensial at-rest** | AES-256-GCM, key dari `ENGINE_ENCRYPTION_KEY` (env atau file). Key derivation: Argon2id dari passphrase, atau raw key 32-byte |
| **Redaksi** | `CredentialValue` di kernel sudah redact Debug/Display — diverifikasi test `credential_value_never_leaks_in_debug` |
| **Least privilege** | Node hanya bisa baca kredensial yang dideklarasikan di `NodeDescriptor::credentials` (D92) |
| **Expression sandbox** | QuickJS: tanpa FS, tanpa network, tanpa process spawn, memory limit, execution timeout |
| **Code node** | Sandbox yang sama. **Tidak ada** padanan `NODE_FUNCTION_ALLOW_EXTERNAL` di MVP — itu pintu ke arbitrary code |
| **Spill file** | Permission 0600, direktori per-instance, GC wajib (A-21) |
| **Webhook** | Rate limiting, HMAC verification opsional, timeout |
| **Auth** | Session cookie (Secure, HttpOnly, SameSite) + API key. Argon2id untuk password |
| **Audit** | Log akses kredensial & perubahan workflow |
| **Dependency** | `cargo audit` di CI |

### 11.1 Pelajaran dari audit VPS Anda

Audit agent5 di VPS menemukan dua kegagalan isolasi CRITICAL (ERR-001, ERR-002):
`/var/lib/agent-comm/comm.db` ber-mode **0666** dan `/var/lib/agent-memory/memory.db`
ber-mode **0777**, sehingga kontrol akses kanal privat dan working memory hanya
ditegakkan di layer aplikasi dan bisa dilewati lewat `sqlite3` langsung.

Temuan itu **terverifikasi dan sudah diremediasi** oleh agent5 pada 2026-09-09
03:34 (jejak di log sudo): dibuat grup `agent-team`, `chown root:agent-team`,
direktori `2770`, file `0660`. Bukti eksploitasi agent5 bukan teoretis — uji
tulis dilakukan dengan `BEGIN IMMEDIATE` lalu `ROLLBACK` agar tidak mengubah
data, dan sebuah spoof-test berhasil memalsukan pesan yang tercatat dikirim
oleh user lain. Keadaan sekarang:

```
2770 root:agent-team /var/lib/agent-comm      660 root:agent-team comm.db
2770 root:agent-team /var/lib/agent-memory    660 root:agent-team memory.db
```

**Tapi remediasi itu belum menutup masalahnya, dan ini bagian yang penting
untuk desain kita.** Verifikasi lanjutan menemukan:

```
/etc/sudoers.d/99-all-nopasswd:  ALL  ALL=(ALL) NOPASSWD: ALL
```

Setiap user di mesin itu punya root tanpa password. Konsekuensinya: **izin
file adalah perlindungan lemah di host semacam ini**, karena siapa pun bisa
`sudo` dan membaca apa pun. Mode `0600` menghentikan akses langsung, tapi
tidak menghentikan jalur lewat root.

Prinsip yang saya tarik, dan sudah masuk desain:

> **Enforcement di tingkat tipe/sistem, bukan disiplin programmer — dan jangan
> asumsikan izin file adalah batas keamanan.**

- Spill file dibuat dengan `0600` oleh konstruktor, bukan oleh konvensi
- Scoping kredensial lewat `NodeDescriptor` yang diverifikasi kompilator
- `#![forbid(unsafe_code)]` di kernel — sudah aktif dan diverifikasi CI
- **Ancaman model harus mengasumsikan penyerang punya root lokal.** Kalau
  engine menyimpan kredensial pengguna (OAuth token, API key) di disk, izin
  file saja tidak cukup: perlu enkripsi at-rest dengan kunci **di luar
  filesystem** (membaca key dari env tidak dihitung — lihat §11 baris "Kredensial at-rest"),
  atau vault eksternal. `ENGINE_ENCRYPTION_KEY` yang dibaca dari file di
  host yang sama memberi perlindungan nol terhadap penyerang ber-root.

> **Catatan proses (jujur):** adjudikasi pertama saya atas temuan agent5
> keliru. Saya mengamati mode `660` pada 05:24 dan menyimpulkan laporan `0666`
> agent5 salah — padahal agent5 sendiri yang sudah memperbaikinya pada 03:34.
> Saya menguji klaim tentang keadaan *masa lalu* dengan mengamati keadaan
> *sekarang*, tanpa memeriksa riwayat perubahan. Ralat penuh ada di
> `#blockers` #324. Pelajaran proseduralnya dicatat di
> `VERIFIKASI-DELIVERABLE-TIM.md` §8, dan berlaku untuk audit apa pun di
> proyek ini: **sebelum membantah klaim tentang keadaan sistem, periksa log
> perubahan dan sebutkan waktu pengamatan di sebelah waktu klaim.**

### 11.2 Lisensi & merek

- Kita **clean-room**: tidak menyalin kode n8n. Idealnya yang membaca spec ≠ yang menulis kode
- **Tidak boleh** memakai nama/logo "n8n" (merek dagang n8n GmbH)
- Klaim kompatibilitas harus didukung katalog deviasi §10.3
- **OPEN-4 (PRD induk):** lisensi produk kita sendiri belum diputuskan

---

## 12. ROADMAP LENGKAP

### Ringkasan

| Fase | Nama | Keluaran bisa dipakai | Ukuran | Kumulatif (solo FT) |
|---|---|---|---|---|
| 0 | Fondasi | kernel crate + bukti benchmark | ✅ selesai | 0 |
| 1 | Engine headless | CLI yang benar-benar menjalankan workflow n8n | XL | 3-6 bln |
| 2 | API & operasi | REST API + auth + kredensial | L-XL | 5-9 bln |
| 3 | UI minimum | Editor visual dasar | XL | 9-17 bln |
| 4 | Produk | Installer, docs, onboarding, lisensi | L | 10-19 bln |
| 5 | Scale & ekosistem | Codegen OpenAPI massal, komunitas | — | 12-24 bln+ |

> **⚠️ Estimasi ini kasar dan bisa meleset 2-3x.** Asumsi: solo developer,
> full-time, tanpa pengalaman sebelumnya membangun canvas editor. Saya tidak
> punya data historis Anda untuk mengkalibrasi. Perlakukan sebagai urutan
> magnitudo.

---

### FASE 0 — Fondasi ⚠️ BELUM SELESAI (koreksi v1.4)

**Keluaran:** crate `kernel` + bukti empiris.

> **Koreksi v1.4 — verifikasi di VPS, lihat `SINTESIS-SPILLSTORE.md`.**
> Bagian ini sebelumnya bertanda **✅ SELESAI**. Itu tidak konsisten dengan gate
> keluar Fase 0 sendiri di bawah, yang mensyaratkan "workspace yang build
> bersih" — dan workspace **tidak** build bersih. Jadi tandanya diubah menjadi
> ⚠️. Yang benar-benar selesai adalah **crate `kernel`**; **workspace** belum.

**Yang terverifikasi lulus** (diukur di VPS, rustup 1.98.1, `--offline`,
setelah `members` diperbaiki — lihat utang di bawah):

| Item | Hasil |
|---|---|
| Test unit + integrasi | `ok. 19 passed; 0 failed` |
| Doc-test | `ok` |
| `cargo clippy --all-targets` | 0 warning, 0 error |
| `cargo build --release` | `Finished in 17.78s` |
| `#![forbid(unsafe_code)]` | aktif di `lib.rs:47` — ditegakkan kompilator |
| Dependency | 4 (serde, serde_json, async-trait, thiserror), 0 internal |

Benchmark: 5 juta item / 753 MB payload dalam **50,4 MB peak RSS**, index 38,1 MB.
**Kualifikasi penting:** angka ini dihasilkan `FileSpillStore` yang berada di
`examples/spill_bench.rs`, dan `cargo test` **tidak menjalankan examples**.
Ke-19 test kontrak semuanya memakai `MemSpillStore` (HashMap di RAM). Jadi
bukti empiris terpenting proyek ini **tidak dijaga test**. Ia valid sebagai
pengukuran, tapi belum sebagai jaminan.

**Utang yang belum beres (blocker Fase 1):**

- **Workspace gagal load, bukan hanya gagal build.** `Cargo.toml` mendeklarasikan
  4 member; `cargo metadata` keluar dengan **exit 101** sebelum kompilasi apa pun:
  `failed to load manifest for workspace member crates/data-plane — no targets
  specified in the manifest`.
- **Deskripsi lama bagian ini tidak akurat dan kini dikoreksi.** Sebelumnya
  tertulis "hanya 2 manifest ada" dan "tiga direktori stub kosong".
  Keadaan sebenarnya (diverifikasi via `git ls-tree -r d3bcff0`):

  | Member | Direktori | Manifest | File `.rs` |
  |---|---|---|---:|
  | `crates/kernel` | ada | ada | **12** |
  | `crates/data-plane` | ada | ada | **0** |
  | `crates/testkit` | **tidak pernah ada** | — | — |
  | `crates/nodes-core` | **tidak pernah ada** | — | — |

  Jadi bukan tiga stub kosong — hanya **satu** direktori ber-manifest-tanpa-target
  (`data-plane`), dan **dua** member yang dideklarasikan tanpa pernah dibuat.
- **Perbaikan minimal sudah diuji berhasil:** buang tiga member bermasalah dari
  `members`, sisakan `"crates/kernel"` → semua item di tabel atas lulus.
- **Kernel gagal kriteria SEC-07 yang sudah disetujui tim.** `grep` untuk
  `PermissionsExt|from_mode|set_permissions|.mode(` di seluruh crate kernel:
  **nol hasil**. `open_rw` memakai `OpenOptions` tanpa `.mode()`, sehingga file
  spill lahir `0666 & ~umask` = **0644** (umask default) atau **0666**
  (umask 000). Implementasi `data-plane` agent2 **lulus** kriteria ini
  (`from_mode(0o600)` + `set_permissions` di jalur tulis). Menyerapnya jadi
  **wajib**, bukan opsional.
- **MSRV membuat jalur apt mustahil.** Kernel mendeklarasikan `rust-version =
  "1.80"`; apt menyediakan 1.75. Toolchain harus rustup, dan per-akun.
- **OPEN-7 PRD induk:** lengkapi atau hapus — belum diputuskan.

**Status freeze kernel — DIREVISI (audit F10).** Kernel sebelumnya dinyatakan
**BEKU**. Itu keliru dan kini diubah jadi **PROVISIONAL-FREEZE**.

Alasannya: kernel dibekukan sebelum punya **satu pun konsumen**. Test internal
(19/19) hanya membuktikan kernel konsisten dengan dirinya sendiri — bukan bahwa
API-nya enak atau benar dipakai. API yang dibekukan tanpa konsumen hampir pasti
salah di titik yang tidak terduga, dan membekukannya membuat kesalahan itu
mahal untuk diperbaiki.

| | BEKU (lama) | PROVISIONAL-FREEZE (baru) |
|---|---|---|
| `check-freeze.sh` (cycle, unsafe, dep count) | ✅ tetap jalan | ✅ tetap jalan |
| Ubah signature API kernel | ❌ dilarang | ✅ boleh, dengan ADR singkat |
| Kapan jadi beku sungguhan | sekarang | setelah `executor` + **≥3 node nyata** memakainya |

**Kriteria unfreeze → freeze permanen:** (1) `executor` berjalan end-to-end,
(2) minimal 3 node nyata (HTTP Request, Set, IF) memakai `NodeContext` tanpa
workaround, (3) nol ADR terbuka, (4) differential test Fase 1 lolos ≥70%.

**Gate keluar:** `scripts/check-freeze.sh` PASSED dengan workspace yang build
bersih.

> **Status gate: TIDAK TERPENUHI — terukur, bukan disimpulkan.**
> `scripts/check-freeze.sh` dijalankan langsung di VPS pada kedua keadaan:
>
> | Keadaan | Hasil skrip | exit |
> |---|---|---:|
> | Ter-commit (`d3bcff0`), apa adanya | `FREEZE CHECK FAILED` | **1** |
> | Setelah 3 member bermasalah dibuang dari `members` | `FREEZE CHECK PASSED` | **0** |
>
> Pada keadaan ter-commit, bagian 6 skrip (`build + clippy + tests`) gagal
> seluruhnya karena workspace tidak bisa load. Bagian 1–5 dan 7 tetap OK
> (acyclic, 0 dep internal, no unsafe, 9 simbol kontrak ada). Setelah
> perbaikan: `cargo build --all-targets` OK, `cargo clippy` 0 warning,
> `cargo test — 19 tests passed`.
>
> `crates/kernel/Cargo.toml` **tidak punya** bagian `[workspace]` sendiri, jadi
> cargo selalu naik ke root — tidak ada cara menjalankan gate ini tanpa
> membereskan manifest root lebih dulu.
>
> Gate ini baru tertutup penuh setelah (a) manifest dibereskan, **dan** (b)
> `FileSpillStore` dipindah dari `examples/` ke `crates/data-plane/src/` agar
> dijangkau `cargo test`, **dan** (c) permission 0600 + checksum diserap dari
> implementasi `data-plane` agent2 supaya lulus SEC-07. Urutan lengkap:
> `SINTESIS-SPILLSTORE.md` §6.1.

**Spike wajib yang ditarik maju ke Fase 1 (audit F13, F14):**

| Spike | Kenapa di Fase 1, bukan nanti |
|---|---|
| **Build statis musl + `rquickjs`** | `rquickjs` meng-compile kode C (QuickJS). Target musl-static di Fase 4 adalah risiko build laten — kalau baru ketemu di Fase 4, itu **12 bulan terlambat** dan bisa membatalkan pilihan QuickJS seluruhnya |
| **Ukur postcard vs serde_json** | Menutup klaim tanpa sumber di §4 (audit F15) |
| **Test cron DST** (02:30 ambigu) | Perilaku rujukan n8n belum diketahui; kandidat entri katalog deviasi (audit F14) |
| **`kill -9` di tengah tulis spill** | Satu-satunya cara membuktikan F19 tertutup |

**Jebakan lingkungan yang sudah terverifikasi (hemat waktu, jangan diulang):**

- `rustdoc` di PATH default adalah **1.75.0 dari apt** sementara `cargo`/`rustc`
  **1.98.1 dari rustup**. Skew ini membuat doc-test gagal dengan
  `error: the -Z unstable-options flag must also be passed to enable the flag
  check-cfg` — pesan yang **terlihat seperti kegagalan kode** padahal murni
  lingkungan. Solusi: taruh `~/.rustup/toolchains/<tc>/bin` di depan `PATH`.
- Direktori kerja harus **dimiliki akun yang menjalankan cargo**; kalau tidak,
  langkah persiapan gagal `PermissionError` dan kegagalan itu mudah disalahartikan
  sebagai kegagalan kode.
- Toolchain rustup **tidak bisa dipinjam lintas akun** (permission denied). Akun
  yang belum mengunduh rustup tidak bisa build sama sekali — kendala nyata untuk
  CI nanti, dan alasan kenapa disk (86% penuh) harus dibereskan lebih dulu.

---

### FASE 1 — Engine Headless

**Keluaran:** `engine run workflow.json` benar-benar mengeksekusi workflow n8n.

**Tidak ada UI. Tidak ada API.** Input file JSON, output log + hasil.

#### 1.1 Crate yang dibangun

| Crate | Isi | Ukuran |
|---|---|---|
| `workflow` | Parser JSON n8n → model internal; validasi (node ada, koneksi valid, parameter valid); versioning | M |
| `storage` | sqlx + SQLite; skema §5.2; migrasi; repository | M |
| `data-plane` | `SpillStore` disk produksi (format biner + index offset); `BlobStore` filesystem; codec postcard; GC | L |
| `expr` + `expr-quickjs` | QuickJS runtime; injection scope; helper; sandbox; `PriorOutputs` bridge | **XL** |
| `scheduler` | Cron; task queue in-process; priority; retry+backoff; lease; recovery | L |
| `executor` | `WorkflowExecute` equivalent; context building; parameter resolution; execution order v0/v1 | **XL** |
| `nodes-core` | 13 node Tier 1 | L |
| `testkit` | In-memory impl semua trait kernel | S |
| `cli` | `run`, `import`, `export`, `list` | S |

#### 1.2 Artefak verifikasi (WAJIB, bukan opsional)

1. **Korpus workflow uji** — min. 50 workflow n8n nyata
2. **Differential test harness** — jalankan korpus di n8n Docker vs engine kita
3. **Benchmark head-to-head** — engine kita vs n8n pada workload identik, mengukur peak RSS
4. **Katalog deviasi** v0.1

> Tanpa keempatnya, semua klaim di materi publik tidak bisa diverifikasi — dan
> itu persis kesalahan yang ditemukan audit A-03 pada blueprint asli.

#### 1.3 Gate keluar Fase 1

- [ ] 10 workflow n8n **nyata** (diimpor, bukan ditulis tangan) jalan benar end-to-end
- [ ] Differential test: ≥70% korpus identik (target naik di fase berikutnya)
- [ ] Benchmark head-to-head: peak RSS ≤ 25% dari n8n pada workload identik
- [ ] Crash recovery teruji: SIGKILL saat running → restart melanjutkan tanpa duplikasi side-effect
- [ ] Node `NonIdempotent` yang crash → `InDoubt` → `NEEDS_REVIEW` (teruji)
- [ ] Nol leak spill file setelah 1.000 eksekusi
- [ ] `check-freeze.sh` tetap PASSED

#### 1.4 Risiko Fase 1

| Risiko | Mitigasi |
|---|---|
| QuickJS ≠ V8 pada perilaku edge | Differential testing sejak awal; katalog deviasi |
| Expression helper kurang lengkap | Enumerasi dari kode sumber n8n, bukan dari dokumentasi |
| Execution order v0/v1 salah | Test spesifik per mode dengan workflow multi-branch |
| Scope creep (nambah node terus) | Bekukan di 13 node Tier 1 sampai gate lolos |

---

### FASE 2 — API, Auth & Operasi

**Keluaran:** Semua operasi Fase 1 bisa dilakukan lewat REST API.

| Komponen | Isi | Ukuran |
|---|---|---|
| `api` | REST axum: workflow CRUD, execution list/get/cancel, credential CRUD, variables, health, metrics | L |
| WebSocket push | Live execution status ke klien | M |
| `auth` | Session, API key, Argon2id, enkripsi AES-256-GCM | M |
| `nodes-core` | +8 node Tier 2 | M |
| `openapi-codegen` | **Prototipe** — 5-10 spec berkualitas | M |
| Observability | tracing, metrics Prometheus, structured log | S |

**Gate keluar:**
- [ ] Public API setara fungsional n8n untuk operasi dasar
- [ ] Kredensial terenkripsi at-rest; **tidak pernah** muncul di log/error/response (teruji otomatis)
- [ ] `cargo audit` bersih di CI
- [ ] Differential test: ≥85% korpus identik
- [ ] Prototipe codegen menghasilkan ≥5 node yang lolos test terhadap API nyata

---

### FASE 3 — UI Minimum *(opsional — lihat OPEN-6)*

**Keluaran:** Pengguna bisa membuat workflow sederhana tanpa menyentuh JSON.

| Komponen | Isi |
|---|---|
| Canvas | Library graph (Vue Flow atau setara), drag-drop, koneksi, pan/zoom |
| Node palette | Dari `NodeDescriptor` registry |
| Form parameter | **Dibangkitkan dari `ParameterSchema`** — termasuk `DisplayCondition` |
| Expression editor | CodeMirror, autocomplete `$json`/`$('Node')`, preview hasil |
| Execution viewer | Per-node run data, status live via WebSocket |
| Credential UI | Form per credential type |

**Gate keluar:**
- [ ] Pengguna yang bukan Anda bisa membuat workflow 5-node tanpa menyentuh JSON
- [ ] `ParameterSchema` → form terbukti: 1 node end-to-end (item checklist KERNEL-SPEC §11 yang masih ⚠️)
- [ ] Undo/redo untuk operasi canvas

> **Ini fase yang paling sering di-underestimate.** PRD-1 §10.5: canvas + NDV
> dengan ~15 tipe parameter termasuk `fixedCollection` bersarang + expression
> editor + execution viewer adalah produk frontend tersendiri.

---

### FASE 4 — Produk yang Bisa Didistribusikan

| Komponen | Isi |
|---|---|
| Packaging | Docker image (multi-stage, distroless), binary statis (musl), install script |
| Dokumentasi | Quickstart, referensi node, referensi expression, katalog deviasi, operasi |
| Onboarding | Setup wizard, workflow contoh |
| Lisensi | Aktivasi/enforcement — **tergantung OPEN-4** |
| Upgrade | Migrasi skema otomatis, rollback (D99) |
| Telemetry | **Opsional, default OFF, disclosure jelas** |
| Error reporting | Opt-in, tanpa payload |

**Gate keluar:**
- [ ] Onboarding < 30 menit dari download ke workflow pertama jalan — **diukur pada ≥5 orang yang bukan Anda**
- [ ] Upgrade dari versi sebelumnya + rollback teruji
- [ ] Dokumentasi cukup untuk pengguna P1 tanpa bertanya

---

### FASE 5 — Validasi Pasar & Ekosistem

**⛔ Gate masuk: G6 dari PRD induk §12.2 harus lolos dulu.**

Jangan masuk fase ini tanpa minimal 3 dari 5 sinyal pasar. Membangun ekosistem untuk produk yang tidak ada penggunanya adalah cara termahal menghabiskan waktu.

| Komponen | Isi |
|---|---|
| Codegen OpenAPI produksi | 50-100 node **terverifikasi** (bukan ribuan node rusak) |
| Node SDK | Untuk pihak ketiga menulis node Rust |
| Postgres support | Untuk yang butuh skala lebih |
| Distributed workers | Ganti in-process queue dengan Redis/Postgres-backed — **hanya jika terbukti perlu** |
| Komunitas | Template, forum, kontribusi |

---

## 13. Metrik & Verifikasi

| ID | Metrik | Target | Status |
|---|---|---|---|
| M1 | Peak RSS vs n8n, workload identik | ≤ 25% | Harness belum ada (Fase 1) |
| M2 | 5 juta item di mesin 2 GB | Lolos | ✅ Terbukti di tingkat kernel |
| M3 | Overhead spill vs inline | ≤ 25% | ✅ Terukur 19% |
| M4 | Korpus impor sukses | ≥ 95% (L2) | Korpus belum ada |
| M5 | Expression identik | ≥ 90% (L3) | Korpus belum ada |
| M6 | Binary idle RSS | ≤ 60 MB | Belum diukur |
| M7 | Cold start | ≤ 500 ms | Belum diukur |
| M8 | Onboarding time | < 30 menit, ≥5 orang | Fase 4 |
| M9 | Crash recovery tanpa duplikasi side-effect | 100% | Test belum ada |
| M10 | Leak spill file | 0 setelah 1.000 eksekusi | Test belum ada |

**Aturan:** metrik tanpa cara ukur bukan metrik. Setiap baris di atas punya metode di §12 fase terkait.

---

## 14. Keputusan Terbuka (khusus dokumen ini)

Menambah OPEN-1…9 di PRD induk.

| ID | Keputusan | Rekomendasi teknis (bukan keputusan) |
|---|---|---|
| **T-1** | Status 3 crate stub + workspace rusak | Bereskan **sebelum** Fase 1. Opsi: lengkapi `testkit`+`data-plane` (berguna) atau hapus |
| **T-2** | SQLite saja, atau SQLite+Postgres dari awal | SQLite saja sampai ada kebutuhan nyata. sqlx multi-DB mahal |
| **T-3** | Code node Python di MVP? | **Tidak.** Butuh runtime Python terpisah, menghancurkan keunggulan binary tunggal |
| **T-4** | Daftar final node Tier 3 | Turunkan dari workflow nyata Anda, bukan tebakan saya |
| **T-5** | Frontend: opsi A/B/C (§9) | A (headless dulu) |
| **T-6** | Bahasa frontend kalau opsi B | Vue 3 + Vue Flow (pola terbukti, sama seperti n8n) |
| **T-7** | Telemetry default | OFF, dengan disclosure |
| **T-8** | Nama produk | Butuh cek merek dagang. **Jangan** mirip "n8n" |
| **T-9** | Index spill berjenjang (§6.5) — bangun sekarang atau tolak execution di atas N-maks? | **Tolak dulu** (fail-fast dengan pesan eksplisit), bangun index berjenjang hanya kalau ada pengguna nyata yang menabrak batas. Alasannya: N-maks 2 GB ≈ 65 juta item sudah jauh melampaui workflow nyata; membangun index berjenjang sekarang adalah optimasi untuk beban yang belum ada |
| **T-10** | Kapan kernel pindah PROVISIONAL-FREEZE → BEKU permanen | Setelah 4 kriteria di §12 Fase 0 terpenuhi. **Jangan** bekukan sebelum ada ≥3 node nyata yang memakainya (audit F10) |
| **T-11** | Algoritma checksum `node_output`: BLAKE3 vs SHA-256 | Tunda sampai ada pengukuran. BLAKE3 diklaim lebih cepat dan tetap kriptografis (audit F17), tapi belum diukur di workload kita |
| **T-12** | Node deprecated (`function`, `cron`, `functionItem`) — dukung sebagai alias atau tolak? (§7.3.3) | **Dukung sebagai alias tipis.** Korpus 171 workflow: `cron` 13 wf (8%), `function` 12 wf (7%), `functionItem` 9 wf (5%) — **total 26/171 (15%) gagal impor** bila ditolak. Pada sampel 13-template `cron` terlihat langka (2x); di korpus penuh ternyata setara `splitOut`/`switch`. Bobot naik dari "nice to have" menjadi hampir wajib. Biaya rendah: pemetaan registry, bukan node baru. **Wajib** dicatat di katalog deviasi §10.3: `function` lama bisa `require()` npm, sandbox V8 kita melarang — alias harus gagal eksplisit, bukan diam-diam berbeda perilaku |
| **T-13** | Kapan daftar node §7.3 dibekukan | **Syarat korpus ≥50 workflow SUDAH TERPENUHI (171).** Daftar sudah direvisi berbasis bukti di §7.3.2 — tapi **jangan bekukan dulu**, karena temuan §7.3.1 menunjukkan pemilihan node bukan masalah utamanya (kurva marginal datar: 26 node = 13% coverage, 90% butuh 153 node). Bekukan hanya setelah T-14 dijawab, sebab janji kompatibilitas menentukan berapa node yang perlu ada di daftar |
| **T-14** | **BARU — apa janji kompatibilitas produk ini kepada pengguna?** (§7.3.1, §7.3.4, `ANALISIS-KORPUS-NODE.md` §11) | Tiga opsi. **A**: "impor workflow n8n Anda, jalankan apa yang didukung" — biaya sedang, risiko pengguna kecewa saat 87% workflow tak jalan. **B**: "engine workflow hemat-memori, kompatibel *format* n8n" — biaya rendah, tapi bukan pengganti n8n dan pasar lebih sempit. **C**: "paritas penuh n8n" — butuh 153+ node, tidak tercapai bertahun-tahun, target bergerak. **Rekomendasi teknis: B**, dengan parser menerima semua node (§7.3.4) supaya A tetap mungkin bertahap tanpa janji di depan. Alasan: diferensiator yang sudah terbukti empiris adalah **memori** (BENCH-A03: 38,1 MB index / 50,4 MB peak pada 5 juta item, vs n8n OOM) — itu nyata, terukur, tidak bergantung jumlah node. Menjanjikan paritas node berarti bertanding di arena yang n8n menangkan dengan 694 node dan ratusan kontributor |

---

## 15. Langkah Berikutnya

1. **Tutup T-1** — perbaiki workspace (blocker teknis nyata, 2 menit kerja)
2. **Tutup OPEN-6 / T-5** — headless atau UI? Ini mengubah roadmap 4-18 bulan
3. **Tutup T-4** — daftar node Tier 3 dari workflow nyata Anda
4. Bangun 4 artefak verifikasi Fase 1 §1.2 **sebelum** menulis node apa pun
5. Baru mulai Fase 1

> **Urutan ini disengaja.** Artefak verifikasi dibangun lebih dulu karena
> tanpa mereka, kita tidak akan tahu apakah engine kita benar sampai berbulan-bulan
> kemudian — dan kesalahan audit A-03 pada blueprint asli persis itu: klaim
> tanpa cara verifikasi.

---

## 16. Ringkasan Jujur

**Yang sudah terbukti:**
- Keunggulan memori struktural, terukur di perangkat keras target yang sebenarnya
- Kernel kontrak yang compile bersih, teruji, tanpa unsafe, tanpa cycle
- Desain recovery yang bisa membedakan side-effect yang tidak pasti

**Yang belum terbukti dan merupakan mayoritas pekerjaan:**
- Kompatibilitas expression (risiko teknis terbesar)
- Kompatibilitas workflow nyata (belum ada korpus)
- Engine eksekusi lengkap (belum ada satu baris pun)
- Seluruh UI (belum diputuskan bahkan apakah perlu)
- Ekosistem integrasi (26 node vs 694 node terverifikasi = 3,7%)
- **Bahwa ada orang yang mau memakai atau membayar ini**

**Perbandingan yang harus dihadapi:** n8n punya 24.019 commit, ~204.000 bintang, 1.700+ template, dokumentasi matang, dan tim penuh. Mereka bahkan sedang membangun engine v2 (`@n8n/engine`) — artinya rewrite engine itu sulit bahkan untuk tim yang memiliki kodenya.

Kita punya keunggulan memori yang nyata dan terukur. **Itu cukup untuk produk yang berguna bagi orang tertentu. Itu tidak cukup untuk menggantikan n8n.** Kedua pernyataan itu harus dipegang bersamaan.

---

*Dokumen ini DRAFT v1.3. Bukan persetujuan. §14, §16 (T-1…T-14) dan OPEN-1…9 di PRD induk harus ditutup oleh Anda sebelum implementasi dimulai.*
