# 🛡️ SOP PENCEGAHAN KERJA TUMPANG TINDIH SWARM
## The Zero-Collision & Exclusive-Ownership Multi-Agent Engineering Protocol
**Versi:** 1.0.0-CANONICAL  
**Status:** ✅ RESMI DISAHKAN (Mandat Pemilik Proyek - 2026-09-09)  
**Otoritas:** Chief Supervisor & Technical Lead (`fern`) & Lead Architect (`matt`)  
**Tujuan:** Menjamin tidak ada dua agen yang mengerjakan tugas yang sama, menyunting berkas yang sama, atau memicu konflik Git (Nol Tumpang Tindih).

---

## 1. Latar Belakang Masalah

Dalam eksekusi swarm 11 agen paralel, tumpang tindih (*work collision*) dapat terjadi melalui 3 celah:
1. **Klaim Sebelum Eksekusi Tidak Ditegakkan**: Agen mulai menulis kode sebelum mencatatkan klaim resmi di antrean kerja.
2. **Tabrakan Direktori (Cross-Domain Mutation)**: Dua agen berbeda mengedit file yang sama di dalam satu crate bersamaan.
3. **Konflik Integrasi Git**: Penggabungan kode secara langsung ke branch utama tanpa isolasi branch fitur (*shadow worktree*).

Untuk meniadakan risiko ini secara permanen, seluruh agen wajib mematuhi **5 Pilar Protokol Anti-Tabrakan**.

---

## 2. Lima Pilar Pencegahan Kerja Tumpang Tindih

```
+-------------------------------------------------------------------------+
|                  ZERO-COLLISION MULTI-AGENT PROTOCOL                    |
+-------------------------------------------------------------------------+
       |
       +---> [PILAR 1] KUNCI MUTEX ATOMIK DI ANTREAN (TASK QUEUE LOCK)
       |     - Eksekusi klaim via transaksi database atomik
       |     - Status IN_PROGRESS eksklusif: 1 Task = 1 Agen (Max 1 Active)
       |     - Aturan Besi: NOL KODE SEBELUM OUTPUT `swarm-task claim` SUKSES
       |
       +---> [PILAR 2] PARTISI DOMAIN & HAK TULIS DIREKTORI (PATH OWNERSHIP)
       |     - Setiap crate memiliki pemilik peran tunggal (Primary Role Owner)
       |     - DILARANG menyunting crate di luar domain tanpa koordinasi tertulis
       |
       +---> [PILAR 3] KONTRAK INTERFACE TERLEBIH DAHULU (SPEC-FIRST HANDSHAKE)
       |     - Kunci Trait, Struct, dan Enum di dokumen kontrak bersama
       |     - Implementor & Caller bekerja paralel 100% independen
       |
       +---> [PILAR 4] ISOLASI RUANG KERJA (SHADOW WORKTREES & FEATURE BRANCHES)
       |     - Setiap pekerjaan diisolasi di direktori pribadi agen (/home/<agent>/)
       |     - Merge ke pohon kanonik hanya melalui 1 pintu (Lead Architect / Gate)
       |
       +---> [PILAR 5] AUDIT TRAIL KLAIM REAL-TIME (TASK_CLAIM_LOG)
       |     - Setiap mutasi klaim dicatat di tabel append-only permanen
       |     - Deteksi instan dan otomatis jika terjadi anomali klaim ganda
```

---

## 3. Matriks Kepemilikan Direktori Eksklusif (Path Ownership)

Setiap agen memiliki yurisdiksi tulis primer. Agen lain dilarang menyunting file di direktori tersebut kecuali peran terkait berstatus `VACANT` dan telah diadopsi secara resmi melalui `swarm-task`:

| Direktori / Crate | Peran Pemilik Primer | Agen Pemegang | Lingkup Tanggung Jawab |
| :--- | :--- | :---: | :--- |
| `crates/kernel/`, `crates/executor/` | `ROLE_CORE` | `@agent1` | Siklus eksekusi Tokio, DAG runner, lifecycle |
| `crates/storage/`, `crates/data-plane/`| `ROLE_STORAGE` | `@agent2` | `FileSpillStore`, SQLite WAL L0, disk streaming |
| `crates/expr/`, `crates/expr-quickjs/` | `ROLE_EXPRESSION` | `@agent3` | QuickJS runtime, Expression Bytecode Cache (EBC) |
| `crates/workflow/`, `hub-templates/` | `ROLE_SCHEMA` | `@agent4` | Rosetta AST Parser, Workflow Hub catalog & schema |
| `crates/nodes-wasm/`, `crates/wcb/` | `ROLE_WASM` | `@agent6` | WASM Community Node Bridge (32MB isolation) |
| `crates/nodes-mcp/`, `crates/api/` | `ROLE_AI_MCP` | `@agent7` | Protocol MCP server, tools, JSON-RPC transport |
| `crates/nodes-openapi/`, `adapters/` | `ROLE_INTEGRATION` | `@agent9` | OpenAPI codegen, Camofox/Scrapling integration |
| `crates/auth/`, `crates/envelope/` | `ROLE_COMPLIANCE` | `@agent10` | Execution Envelope, audit trail, RFC 3161 anchor |
| `crates/testkit/`, `qa/` | `ROLE_SECURITY_QA` | `@agent5` / Plt. | Differential tests, fuzzing, CI quality gates |

---

## 4. Prosedur Operasional Standar (SOP) Langkah Kerja Agen

Sebelum mulai bekerja pada tugas apa pun:
1. **Langkah 1 (Cek Tugas)**: Jalankan `/usr/local/bin/swarm-task next` untuk menemukan tugas yang cocok dan bebas dependensi.
2. **Langkah 2 (Klaim Eksklusif)**: Jalankan `/usr/local/bin/swarm-task claim <TASK_ID>`.
   * Jika output `[-] GAGAL`, **BERHENTI SEKETIKA**. Tugas sedang dipegang agen lain atau kuota penuh.
   * Jika output `[+] SUKSES`, kunci mutex telah Anda pegang resmi.
3. **Langkah 3 (Kerja Terisolasi)**: Tulis kode hanya di direktori pribadi atau branch fitur terisolasi. Jaga detak jantung via `/usr/local/bin/swarm-task heartbeat <TASK_ID>` setiap <10 menit.
4. **Langkah 4 (Penyelesaian & Serah Terima)**: Jalankan pengujian mandiri. Jika lolos 100%, serahkan via `/usr/local/bin/swarm-task complete <TASK_ID> <path/to/artifact>`.

---

## 5. Protokol Resolusi Jika Terjadi Potensi Tabrakan
Jika dua agen secara tidak sengaja mengincar tugas atau area yang sama:
1. **Otoritas Database**: Agen yang pertama kali mencatatkan status `IN_PROGRESS` di `task_queue` adalah pemilik sah tugas tersebut.
2. **Pemisahan Peran Implementor vs Verifikator**: Agen kedua yang datang otomatis dialihkan menjadi *Independent QA / Reviewer* untuk tugas tersebut (seperti yang diterapkan pada `W0-SPILL-IMPL` oleh `@agent2` dan `W0-SPILL-TEST` oleh `@agent1`). Ini mengubah potensi konflik menjadi mekanisme jaminan mutu (*Separation of Concerns*).
