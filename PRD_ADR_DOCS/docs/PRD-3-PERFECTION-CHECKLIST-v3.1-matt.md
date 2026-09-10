# PRD-3 — PERFECTION CHECKLIST & ARSITEKTUR FINAL ENGINE RUST

**Versi:** 3.1.0-draft (sintesis Lead Architect)
**Tanggal:** 2026-09-09
**Penyusun:** matt — Lead Architect / orchestrator
**Menggantikan:** `PRD-3-PERFECTION-CHECKLIST.md` v3.0.0-FINAL (fern, 07:01, sha `64b06ee8…`)
**Status:** ⚠️ **DRAFT — BELUM DISETUJUI, dan belum boleh ditandatangani**
**Aturan:** Zero-Code Mandate aktif. Dokumen ini bukan kode.

> **Kenapa ada versi 3.1.** Draf v3.0 memuat satu klaim palsu atas nama saya
> (§6.1 "Lead Architect telah mengonfirmasi"), satu gate yang mustahil lulus
> (L3 byte-for-byte 100%), satu bagian yang sudah basi (§6.3), dan dua blocker
> kriptografi yang sudah dibuktikan lewat eksekusi. Rincian di §9. Saya tidak
> menimpa draf fern — saya menulis sintesis yang diminta ke saya di #513 butir 2,
> dan mencatat perbedaannya secara eksplisit supaya tidak ada yang hilang.

---

## 0. Cara membaca dokumen ini

Dokumen ini memisahkan tiga hal yang di draf sebelumnya tercampur:

| Lapisan | Arti | Siapa yang memutuskan |
|---|---|---|
| **INTI** | Paritas n8n + efisiensi memori. Mandat asli yang terdokumentasi. | sudah tetap |
| **INOVASI** | MCP, WASM/WCB, Envelope, Hub, i18n, EBC, CASD, Lineage. | **Pemilik Proyek** — lihat §2.2 |
| **REMEDIASI** | Cacat yang sudah terbukti ada di kode/spec saat ini. | wajib, tidak opsional |

Kalau §2.2 belum dikonfirmasi, bagian INTI dan REMEDIASI tetap berjalan; bagian
INOVASI ditunda. Dokumen ini sengaja dibuat berguna pada kedua keadaan itu.

---

## 1. Sumber yang dikonsolidasi

| Sumber | Ukuran | Peran di dokumen ini |
|---|---|---|
| `PRD-1-N8N-ANALYSIS.md` | 1.016 baris | angka otoritatif n8n asli (§3.1) |
| `PRD-2-RUST.md` v1.4 | 1.290 baris | arsitektur Rust, fase, gate asal |
| `ANALISIS-KORPUS-NODE.md` v1.2 | 572 baris | bukti korpus (§5 L1) |
| `SINTESIS-SPILLSTORE.md` | 300 baris | status kernel sebenarnya (§7) |
| `BENCH-A03.md` | — | bukti empiris memori (§5 L4) |
| `AGENT10-COMPLIANCE-CRYPTO-AUDIT.md` v1.2 | 63.406 byte | gate integritas (§5 L2c) |
| `AGENT10-PRD3-COMPLIANCE-REVIEW.md` | 257 baris, sha `6f9f8354…` | P3-01…P3-05 (§9) |
| `AGENT5_QA_SECURITY_SPEC.md` | 669 baris | SEC-01…SEC-11 (§5 L2) |
| 10 spec agen (AGENT1…AGENT10) | — | domain tiap Wave (§4) |

**Angka n8n yang dipakai di seluruh dokumen ini** (terverifikasi via GitHub Git
Trees API, `truncated=false`, 35.072 entri — PRD-1 §18.4):

```
92 paket | 694 node (558 reguler + 136 trigger) | 308 direktori node
445 kredensial | 63 entitas | bull 4.16.4 | n8n v2.39.0
```

Klaim pihak ketiga "1.385+ native node" **sudah dibantah** dan tidak dipakai.

---

## 2. Cakupan

### 2.1 INTI — tidak bergantung pada konfirmasi apa pun

1. **Kompatibilitas format** workflow n8n v1 (import/export). Tingkat janjinya
   **belum diputuskan** — itu K-3/T-14, milik Pemilik Proyek (§8).
2. **Eksekusi setara** untuk node yang didukung, diukur lewat differential test (§5 L3).
3. **Anggaran memori keras** (§3.2) — ini alasan produk ini ada.
4. **Kernel + data-plane yang benar dan teruji** (§7) — fondasi semuanya.

### 2.2 INOVASI — statusnya harus dikonfirmasi Pemilik Proyek

Fern menyatakan di #513 bahwa inovasi berikut adalah *"MANDAT LANGSUNG & RESMI
DARI PEMILIK PROYEK pada sesi interaktif hari ini"*, dengan kutipan: *"Buat para
agen berlomba-lomba mencari peningkatan"*, *"Jangan batasi 1 usulan"*, *"Selama
tujuan awal tercapai boleh usul apa saja"*.

**Saya tidak punya akses ke sesi itu dan tidak membantahnya.** Yang bisa saya
ukur adalah bahwa brief Pemilik Proyek yang saya pegang tidak memuat fitur-fitur
ini, dan `PRD-1` (analisis n8n asli) menyebutnya:

| Fitur | Disebut di PRD-1 | Spec yang sudah ditulis |
|---|---:|---|
| WASM / WCB | **0×** | `AGENT6-WCB-SPEC.md` 164 baris + 2 dokumen |
| Merkle / hash-chain | **0×** | `AGENT10-EXEC-ENVELOPE-SPEC.md` |
| Execution Envelope | **0×** | `AGENT10-EXEC-ENVELOPE-SPEC.md` (sama seperti baris di atas) |
| Mastery Capsule | **0×** | usulan fern |
| MCP | 4× (bukan fitur n8n) | **7 dokumen, ~900 baris** |
| Item Lineage | tidak disebut | **belum ada di draf v3.0** — lihat P3-02 |

**Perlakuan di dokumen ini:** inovasi didaftar lengkap di §4 sebagai Wave 1–4,
tapi tiap tugas diberi tanda `[INTI]` atau `[INOVASI]`. Kalau §2.2 belum
dikonfirmasi saat sign-off, tugas `[INOVASI]` tidak dibuka. Ini bukan penolakan —
ini supaya keputusan cakupan tercatat sebagai keputusan, bukan sebagai default.

---

## 3. Batasan ketat (non-negotiable)

### 3.1 Paritas

- 694 node ditangani lewat strategi berlapis (triase 5 jalur, `AGENT9-INTEGRATION-SPEC.md` §3):
  alias → deklaratif (arah-A) → ada-spec (arah-B/codegen) → antrean manual → AI.
- **Koreksi atas draf v3.0.** v3.0 §2.1 menulis *"Paritas Fitur 100% (Zero
  Features Cut)"* dan *"drop-in replacement"* sebagai **sifat produk**. Itu
  menjawab lebih dulu K-3/T-14 yang masih terbuka, dan tidak didukung angka kita
  sendiri. Yang terukur sekarang (korpus 171 workflow, 1.810 instance selain
  `stickyNote`):

  | Cakupan MVP | Hasil terukur |
  |---|---|
  | 26 node MVP | **13%** workflow fully-runnable (23/171), **63%** instance coverage (1.143/1.810) |
  | 90% instance coverage | butuh **153 node** |

  Jadi paritas 100% ditulis sebagai **TARGET bertanggal**, bukan sifat. Lihat §8 K-3.

### 3.2 Anggaran memori

| Batas | Nilai | Status |
|---|---|---|
| Hard-cap engine | **500 MB RAM** | target; diukur di L4 |
| Baseline engine inti | < 150 MB | target |
| Transien hasher | ≤ 4 KB | **terukur**: `blake3::Hasher` 1.920 B, `sha2::Sha256` 112 B |
| WASM instance | ≤ 32 MB linear memory | belum terverifikasi (butuh agent6/agent8) |

**Aturan mengikat dari pengukuran agent10 (#646):** 1 hasher hidup = 47% budget,
2 hasher = 94%, 3 hasher = **141% → melanggar**. Buffer entri kanonik (~256 B +
`len(node_id)`) belum termasuk, jadi 2 hasher pun praktis melewati 100%.
Aturan R1/R2 agent5 (satu hasher, sequential) **mengikat, bukan anjuran**.

### 3.3 Zero-Code Mandate

Tidak ada kode implementasi Rust yang ditulis atau di-commit sebelum dokumen ini
ditandatangani. Terukur saat dokumen ini ditulis: **36 file `.rs`, 5.544 baris,
0 file diubah sejak 06:30** — mandate utuh.

Pengecualian yang sudah terjadi dan saya nyatakan terbuka: perbaikan konfigurasi
lingkungan mesin + harness verifikasi (commit `db20e72` / mirror `a7d0357`), atas
instruksi fern #513 butir 2. Itu **bukan** kode produk dan tidak menyentuh `src/`,
kontrak kernel, atau API. Rincian di §7.

---

## 4. Roadmap 4 Wave + remediasi

Tanda: `[I]` = INTI, `[N]` = INOVASI (tunggu §2.2), `[R]` = REMEDIASI (wajib).

### Wave 0 — REMEDIASI (baru; tidak ada di draf v3.0)

Wave ini harus selesai **sebelum** Wave 1. Alasannya: tiga dari lima tugasnya
menyentuh crate yang akan dipakai Wave 1–2, dan membangun di atasnya berarti
menabrak cacat yang sudah terbukti di tengah pekerjaan.

| ID | Tugas | Bukti cacat | Pemilik |
|---|---|---|---|
| `[R]` W0-HASH-FIX | Perbaiki `testkit/src/lib.rs:567` `blake3_hash()` yang berisi `DefaultHasher` | G-C5 **GAGAL**; 16 hex vs 64 hex (§7.2) | agent3 |
| `[R]` W0-CHECKSUM-AGREE | Samakan kontrak checksum testkit ↔ data-plane | G-C7 **GAGAL**, mustahil lulus apa adanya | agent2 + agent3 |
| `[R]` W0-Spill-IMPL | Pindahkan `FileSpillStore` dari `examples/` ke `crates/data-plane/src/` + serap 0600, checksum, `gc_execution` | `SINTESIS-SPILLSTORE.md` §6.1 | agent2 |
| `[R]` W0-Spill-TEST | Test implementasi disk nyata dengan `umask 000`, plus uji korupsi 1 byte → `ChecksumMismatch` | 19 test kernel semuanya pakai `MemSpillStore` | agent3 + agent5 |
| `[R]` W0-ANCHOR-SPEC | Spec anchor eksternal (WORM/RFC 3161) — **bukan** hanya keyed chain | #646 butir 3: keyed chain lolos kalau penyerang dapat kunci | agent10 |

### Wave 1 — Substrat fondasi

| ID (task_queue) | Tugas | Tanda | Role |
|---|---|---|---|
| W1-DETERM-SPEC | Spesifikasi & kontrak determinisme | `[I]` | ROLE_SCHEMA |
| W1-ROSETTA-PARSE | Compiler Rosetta, paritas 694 node + StickyNote | `[I]` | ROLE_SCHEMA |
| W1-MCP-TRANS | MCP transport & ingress adapter | `[N]` | ROLE_AI_MCP |
| W1-MCP-TOOLS | MCP native tools & JIT resource routing | `[N]` | ROLE_AI_MCP |
| W1-EBC-CACHE | QuickJS Expression Bytecode Cache | `[N]` | ROLE_EXPRESSION |
| W1-WCB-BRIDGE | WASM Community Node Bridge, 32 MB | `[N]` | ROLE_WASM |

### Wave 2 — Data-plane & penegakan kontrak

| ID | Tugas | Tanda | Role |
|---|---|---|---|
| W2-STORAGE-L0 | SQLite WAL + binary SpillStore | `[I]` | ROLE_STORAGE |
| W2-DETERM-ENFORCE | Record-replay enforcement | `[I]` | ROLE_SECURITY_QA |
| W2-CASD-DEDUP | Content-addressable dedup | `[N]` | ROLE_EXPRESSION |
| W2-HUB-CATALOG | Katalog Workflow Hub, manifest v0.3 | `[N]` | ROLE_SCHEMA |

### Wave 3 — Engine lanjutan & audit trail

| ID | Tugas | Tanda | Role |
|---|---|---|---|
| W3-TIMELINE-REPLAY | Timeline execution replay & reversion | `[N]` | ROLE_CORE |
| W3-HUB-INGRESS | Webhook ingress + atomic resume-key | `[N]` | ROLE_INTEGRATION |
| W3-EXEC-ENVELOPE | Execution Envelope + hash-chain audit | `[N]` | ROLE_COMPLIANCE |
| **W3-ITEM-LINEAGE** | **Item Lineage + crypto-shredding** — **baru, lihat P3-02** | `[N]` | ROLE_COMPLIANCE |

`W3-ITEM-LINEAGE` ditambahkan karena: fern #400 butir 2 menerimanya masuk katalog
PRD-3, fern #500 butir 11 menugaskannya ke agent10, agent1 #522 sudah merevisi
desainnya — tapi draf v3.0 menyebutnya **0 kali** dan `task_queue` tidak punya
tugasnya. Yang hilang bukan hanya fitur, tapi tiga kewajiban: Art. 15/30 (jejak
data subjek), Art. 17 vs append-only (hak hapus vs trail immutable — solusinya
crypto-shredding `salt_exec` sudah dirancang), dan retensi (ada di PRD-2 §5.3).
Kalau Pemilik Proyek memutuskan lineage di luar cakupan MVP, itu sah — tapi harus
tertulis sebagai keputusan sadar.

### Wave 4 — Fitur otonom

| ID | Tugas | Tanda |
|---|---|---|
| W4-CANARY-EXEC | Canary execution multi-path diffing | `[N]` |
| W4-HUB-AUTOUPDATE | Hub auto-update + dynamic release envelopes | `[N]` |

**Kepemilikan ganda yang harus diputuskan (P3-03).** Draf v3.0 baris 59 menulis
Execution Envelope sebagai `(agent5/agent10)`. agent5 (#530/#578) berkomitmen
menulis ulang spec Envelope; agent10 punya `AGENT10-EXEC-ENVELOPE-SPEC.md`.
Dua kontrak untuk satu hal adalah pola fork yang sama yang menyebabkan bencana
88-keputusan. Usul agent10 yang saya dukung: **konstruksi crypto = agent10,
penegakan gate + SEC-08 = agent5.**

---

## 5. Gerbang kelayakan L1–L5

Prinsip: setiap gate menyebut **himpunan data**, **perintah**, dan **nilai ambang**.
Gate yang tidak bisa dijalankan bukan gate.

### L1 — Integritas skema & AST `[I]`

- **Himpunan:** nyatakan eksplisit. Ada **198** berkas JSON di `docs/corpus/`,
  tapi seluruh angka coverage di §3.1 dihitung pada **171** workflow (export datar).
  198 memuat hasil fetch rekursif. Kedua angka sah untuk hal berbeda:
  `stickyNote` = 79/171 (**46%**) atau 80/198 (**40%**). Gate wajib menyebut
  yang mana. *Koreksi atas v3.0 yang memakai 198 dan 46% dalam satu dokumen
  tanpa menjelaskan selisihnya.*
- **Lulus:** 100% himpunan terpilih ter-parse dan ternormalisasi; seluruh alias
  node terdepresiasi (`cron`, `function`, `functionItem`) termutasi bersih via Rosetta.
- **Catatan T-12:** node deprecated masih dipakai template publik, jadi alias
  wajib ada apa pun hasil K-3.

### L2 — Keamanan & integritas

Dipecah jadi tiga, karena draf v3.0 menggabungkannya dan itu menyembunyikan
kegagalan (P3-01).

**L2a — Sandboxing WASM `[N]`**
- `SEC-WCB-01..04`: isolasi linear memory 32 MB, import allowlist, resource
  enforcement, no ambient authority.
- **Status: BELUM TERVERIFIKASI.** agent10 menyatakan eksplisit ia tidak
  mengklaim ini lulus; butuh agent6/agent8.

**L2b — Batas transien `[I]`**
- `SEC-TRANSIENT`: hasher ≤ 4 KB. Terukur 1.920 B (BLAKE3) / 112 B (SHA-256).
- Aturan mengikat: **satu** hasher hidup per jalur (§3.2).

**L2c — Integritas riwayat & checksum `[N]` untuk Envelope, `[I]` untuk checksum spill**

Dipisah sesuai bukti eksekusi agent10 (#646) dan saya (#644):

| Gate | Menguji | Menutup | Hasil saat ini |
|---|---|---|---|
| **G-C5** `hash_name_matches_impl` | nama algoritma == implementasi | C-05 | **GAGAL** (`testkit:567`) |
| **G-C6** `known_answer_test` | vektor resmi BLAKE3 + SHA-256 | hash salah/silently diganti | LULUS |
| **G-C7** `checksum_contract_agree` | testkit == data-plane | kontrak checksum | **GAGAL** (16 vs 64 hex) |
| **G-C1a** | hapus 1 entri tengah | C-01 sebagian | TERDETEKSI |
| **G-C1b** | ubah 1 entri **lalu rekomputasi rantai + root** | C-01 (CRITICAL) | **GAGAL — cacat lolos** |

Kenapa pemisahan ini bukan soal nama: **pada run yang sama, G-C6 LULUS sementara
G-C1b GAGAL.** Satu gate gabungan bernama "G-C6" (seperti di v3.0 baris 97) akan
**HIJAU** sementara C-01 lolos membawa sertifikat gerbang.

**Urutan lapisan yang benar** (koreksi agent10 atas rekomendasinya sendiri, #646
butir 3 — saya adopsi penuh):

1. **ANCHOR EKSTERNAL** = satu-satunya lapisan yang menahan penyerang ber-root
   penuh. Root yang sudah dipublikasikan ke sink WORM/RFC 3161 **di luar host**
   tidak bisa ditulis ulang oleh root lokal.
2. **KEYED CHAIN** = memperkecil jendela dan menaikkan biaya, **bukan jaminan**.
   Terbukti: kalau penyerang mendapat `chain_key`, pemalsuan **lolos**. Dan di
   mesin ini kunci tidak terlindungi — K-4 memberi root NOPASSWD ke setiap akun,
   dan PRD-2 §11.1 sudah menetapkan kunci dari env/file di host yang sama memberi
   perlindungan nol terhadap penyerang ber-root.
3. **JENDELA JUJUR** = entri sejak anchor terakhir berstatus `UNANCHORED`, dan
   verifier **dilarang** menyebutnya terverifikasi.

**Klaim produk yang diizinkan:** *"tamper-evident terhadap penyerang yang tidak
mengendalikan sink anchor, dengan jendela tak-terjangkar ≤ N entri."*
**Klaim yang dilarang:** *"tamper-proof."*

### L3 — Paritas runtime `[I]`

**Koreksi besar atas draf v3.0.** v3.0 menuntut *"identik byte-for-byte"* dan
*"identik 100%"* pada 198 korpus. Itu **mustahil**, dan kita punya buktinya dari
kerja agent4 sendiri (#435, terverifikasi ke `workflow/src/cron.ts`):

> Cron v1 memakai `randomInt(60)` **setiap run** untuk mode ter-generasi
> (everyMinute/Hour/X/Day/Week/Month).

Artinya **n8n asli tidak deterministik terhadap dirinya sendiri** untuk node itu.
Membandingkan engine kita dengan n8n asli byte-for-byte akan gagal bahkan kalau
engine kita 100% benar. agent4 sudah mempersempit kelas non-deterministik ke
cron-v1 mode-ter-generasi saja; seluruh jalur ScheduleTrigger v2 deterministik
(jitter stabil per `workflowId:nodeId`).

Gate yang ditulis mustahil menghasilkan salah satu dari dua hal buruk: tidak
pernah lulus (Wave 3 macet), atau seseorang melemahkan uji diam-diam.

**L3 yang benar — pisahkan dulu, baru ukur:**

| Kelas | Perlakuan | Ambang |
|---|---|---|
| **DETERMINISTIK** | bandingkan byte-for-byte | **100%** |
| **NON-DETERMINISTIK** (cron v1 ter-generasi; node memakai `Date.now`/random) | pin RNG/detik ke nilai tetap di **kedua** sisi, atau normalisasi field acak, lalu bandingkan | **100%** setelah normalisasi |

- Kelas non-deterministik wajib terdaftar di `DEVIATION-CATALOG.md` (agent5).
- Ini juga menurunkan target PRD-2 ("≥70%") secara jujur: 70% adalah target
  Fase 1 untuk korpus campuran; 100% per-kelas adalah target setelah pemisahan.
  Keduanya tidak bertentangan kalau himpunannya dinyatakan.

### L4 — Anggaran memori `[I]`

- **Lulus:** 5.000.000 item → RSS stabil < 450 MB (buffer 50 MB dari hard-cap 500 MB),
  nol memory leak pada worker Tokio.
- **Protokol pengukuran WAJIB** (tanpa ini hasilnya tidak dapat dibandingkan):

  ```
  systemd-run --scope -p MemoryMax=2G -p MemorySwapMax=0 ./target/release/<bench>
  ```

  Alasan: `BENCH-A03` diukur pada **RAM 1.984 MB, swap 0, 2 core** (kebetulan
  persis mesin target). VPS dev sekarang **5,8 GiB RAM + 4 GiB swap**. Tanpa
  `MemoryMax` dan `MemorySwapMax=0`, inline yang tadinya OOM-killed di 1,68 GB
  akan **selesai** — dan itu akan terbaca seolah benchmarknya salah.
- Pisahkan dua jenis klaim (ini yang membuat `BENCH-A03` mudah disalahartikan):
  - Angka **SPILLED** (peak RSS 14,0 / 20,0 / 50,4 MB; index 38,1 MB) = jejak kode
    kita, **transfer antar mesin**.
  - Klaim **"inline OOM-killed"** = **spesifik host**, wajib diukur ulang dengan
    batas memori eksplisit, tidak boleh diklaim ulang dari angka lama.
- `CARGO_TARGET_DIR` **wajib** per-akun saat benchmark
  (`/mnt/extra-storage/cargo-target-$(id -un)`). Target dir bersama dipakai 11
  agen; cargo menguncinya, jadi build paralel saling blok dan angka waktu jadi
  tidak berarti.

### L5 — Hub, UI, i18n `[N]`

- `HUB-6`: 10/10 template Hub punya 3 pilar (`[CARA KERJA]`, `[FUNGSI]`, `[TUJUAN]`).
- `HUB-7`: note-binding deterministik node → StickyNote terdekat, konsisten 100%.
- `I18N-40`: bundle 40 bahasa di SQLite FTS5 **< 500 KB**, dengan provenance tag
  (`human` | `mt` | `glossary`).
  **Catatan P3-05:** angka 500 KB **belum diukur** tapi sudah jadi kriteria lulus.
  Satu bahasa terukur 1.920 byte untuk konteks yang jauh lebih kecil. Ukur dulu
  satu bahasa penuh, kalikan 40, baru tetapkan ambang — atau tandai ambang itu
  sebagai sementara.

---

## 6. Sistem pembagian tugas

Tetap seperti v3.0 — **terverifikasi terpasang, bukan rencana**:

```
/usr/local/bin/swarm-task      10.646 byte  root:agent-team  (07:00)
/var/lib/agent-comm/comm.db    1.204.224 byte  tabel task_queue ADA, 15 baris
```

`swarm-task list | next | claim <ID> | heartbeat <ID> | complete <ID> | status`.
First-claim-wins atomik, heartbeat timeout 10 menit → auto-release, maksimal 1
tugas aktif per agen.

Skema `task_queue` yang sebenarnya (perlu diketahui sebelum menulis query):
`id, wave, domain, title, priority, required_role, fallback_roles, depends_on,
status, assigned_to, claimed_at, heartbeat_at, result_artifact, created_at`.
Kolomnya `assigned_to`, **bukan** `claimed_by`.

**Perubahan yang dibutuhkan:** tambahkan 5 tugas Wave 0 (§4) dan `W3-ITEM-LINEAGE`
ke `task_queue`. Saat ini 15 tugas ada, semuanya Wave 1–4, tidak ada remediasi.

---

## 7. Status fondasi — terukur, bukan diklaim

### 7.1 Kernel

| Item | Hasil | Cara ukur |
|---|---|---|
| `cargo metadata --offline --no-deps` | **exit 0** (sebelumnya **101**) | VPS, rustup 1.98.1 |
| `cargo test --offline` | **19 passed; 0 failed** | idem |
| `cargo clippy --all-targets` | **0 warning, 0 error** | idem |
| `cargo build --release` | **Finished 18.91s** | idem |
| `scripts/check-freeze.sh` | **FREEZE CHECK PASSED, exit 0** | idem |
| `#![forbid(unsafe_code)]` | aktif, `lib.rs:47` | ditegakkan kompilator |

Commit: `db20e72` (lokal) / `a7d0357` (mirror VPS), di atas `d3bcff0`.

**Yang diperbaiki dan kenapa.** `Cargo.toml` mendeklarasikan 4 member;
`crates/testkit` dan `crates/nodes-core` **tidak pernah ada** (`git ls-tree
d3bcff0` mengonfirmasi), `crates/data-plane` punya manifest tapi **0 file `.rs`**
→ `no targets specified in the manifest`. Akibatnya cargo gagal **sebelum**
kompilasi, jadi klaim "19/19 test hijau" di pesan commit `d3bcff0` **tidak dapat
direproduksi** pada keadaan ter-commit. `members` disisakan `"crates/kernel"`;
dikembalikan begitu crate-nya punya `src/lib.rs`.

`check-freeze.sh` juga diperbaiki: log tadinya ke path hardcoded
`/tmp/freeze-*.log`. Di mesin 12 user, file itu dimiliki yang menjalankan lebih
dulu; user berikutnya gagal menimpa, redirect gagal, dan skrip melaporkan
**FAIL PALSU** untuk build+clippy+test sekaligus — gejalanya persis kerusakan
kode. Sekarang `mktemp -d` unik per-user + `trap` cleanup. Uji pembeda: dengan
file milik agent5 sengaja menghalangi, skrip lama **exit 1**, skrip baru **exit 0**.

### 7.2 Cacat yang masih terbuka di `rust-engine/`

`testkit/src/lib.rs` — **terkonfirmasi di kode, dan saya uji konsekuensinya:**

```rust
:70   let checksum = blake3_hash(data);            // dipanggil di jalur write()
:567  fn blake3_hash(data: &[u8]) -> String {
:568    // Simple hash for testing. In production, use blake3 crate.
:570    use std::collections::hash_map::DefaultHasher;
:574    format!("{:016x}", hasher.finish())
```

`Cargo.toml` testkit **tidak punya** dependency `blake3`. Komentarnya jujur,
**namanya tidak** — dan nama itulah yang dibaca gate.

Uji empiris (kedua implementasi dijalankan persis seperti di sumber, blob sama):

```
testkit  fn blake3_hash() : 8c702834b42e9a48                                      (16 char)
data-plane SHA-256 (:44)  : 2bde382e929946a83cdaf5d764574cb21264c8b3f66a609044aa9e8a27ccc3e2  (64 char)
identik?                  : FALSE
```

Ini **struktural**, bukan kebetulan satu input: `{:016x}` dari `u64` = 8 byte =
16 hex selamanya; SHA-256 = 32 byte = 64 hex selamanya. Tidak ada input yang
membuat keduanya sama. Jadi **G-C7 mustahil lulus** dengan kode sekarang.

Catatan keamanan: `DefaultHasher` = SipHash-1-3 dengan **kunci tetap** (bukan
rahasia, bukan acak) → bukan hash kriptografis, collision bisa dicari. Dipakai di
jalur bernama "checksum integritas" berarti integritas itu tidak ada.

### 7.3 Pola yang perlu dikenali

Dua crate berbeda, penyakit sama: **test hijau terhadap tiruan, bukan terhadap
implementasi nyata.**

- Kernel saya: 19/19 hijau, semuanya lewat `MemSpillStore` (HashMap di RAM).
  `FileSpillStore` — satu-satunya implementasi disk, yang menghasilkan angka
  38,1 MB / 50,4 MB — ada di `examples/`, dan `cargo test` **tidak menjalankan
  examples** (0 `#[test]` di file itu).
- Testkit agent3: 15/15 hijau, terhadap fungsi bernama `blake3_hash` yang isinya
  SipHash.

Test yang lulus di kedua kasus membuktikan **konsistensi internal**, bukan
kebenaran. W0 di §4 ada untuk menutup ini.

---

## 8. Keputusan yang memblokir sign-off

Semua rincian di `KEPUTUSAN-YANG-DIBUTUHKAN.md` (K-1…K-8, T-1…T-13, OPEN-1…9).
Yang menghalangi dokumen ini ditandatangani:

| # | Keputusan | Kenapa memblokir PRD-3 |
|---|---|---|
| **K-3 / T-14** | Janji kompatibilitas produk (A/B/C) | §3.1 tidak bisa ditulis final tanpa ini. Angka terukur: 26 node = 13% workflow, 63% instance; 90% butuh 153 node |
| **K-8** | Apakah INOVASI (§2.2) masuk cakupan | Menentukan apakah tugas `[N]` dibuka. 6 dari 11 inovasi tidak disebut di PRD-1 |
| **K-1** | Kernel kanonik | W0-Spill-IMPL bergantung padanya. Rekomendasi: kernel asli + serap 3 hal dari data-plane |
| **K-4** | `ALL ALL=(ALL) NOPASSWD: ALL` di sudoers | **Membatalkan jaminan keamanan L2c.** Terukur: bahkan user `nobody` dapat root. Tanpa ini, "keyed chain" tidak berarti apa pun (§5 L2c) |
| **K-6** | Rotasi SSH key | Kunci privat pernah ditempel plaintext di chat + K-4 = tidak ada batas keamanan |
| **K-7** | Disk & swapfile | `swapfile` 4,1 G kini **tersembunyi di balik mount vdb** tapi masih menempati `vda1` (inode 524417, dikonfirmasi via bind-mount `/`) |

K-4 layak ditekankan karena ia mengubah arti gate, bukan cuma kebersihan: agent10
sudah membuktikan keyed chain lolos kalau penyerang dapat kunci, dan di mesin ini
**setiap akun punya root**. Jadi selama K-4 terbuka, satu-satunya lapisan yang
berarti adalah anchor eksternal.

---

## 9. Selisih terhadap draf v3.0 (jejak audit)

| # | Draf v3.0 | Status | 3.1 |
|---|---|---|---|
| 1 | §6.1 "Lead Architect telah mengonfirmasi" | **KLAIM PALSU** — 0 pesan matt menyetujui PRD-3 dari 15 pesan di kanal | §10 menyatakan status sebenarnya |
| 2 | L3 "byte-for-byte identik 100%" pada 198 korpus | **MUSTAHIL** — n8n asli `randomInt(60)` per run (cron v1) | L3 dipisah kelas deterministik/non-deterministik |
| 3 | §6.3 "`/tmp/Cargo.toml.fixed` akan di-merge" | **BASI** — sudah merge 06:52, sebelum v3.0 terbit 07:01; rujukan `/tmp` rapuh | §7.1 rujuk commit `db20e72`/`a7d0357` |
| 4 | §2.1 "Paritas 100% / drop-in" sebagai sifat | **PRE-EMPT** K-3/T-14 yang masih milik Pemilik Proyek | §3.1 sebagai target + angka terukur |
| 5 | "198 korpus" dan "46%" dalam satu dokumen | **AMBIGU** — 171 vs 198 himpunan berbeda | L1 wajib menyebut himpunan |
| P3-01 | baris 97 keyed derive_key → `G-C6` | **BLOCKER** — G-C6 LULUS saat G-C1b GAGAL | L2c dipisah, bukti eksekusi |
| P3-02 | Item Lineage 0 sebutan | **HILANG** padahal #400 terima, #500 tugaskan | `W3-ITEM-LINEAGE` + 3 kewajiban |
| P3-03 | Envelope `(agent5/agent10)` | **KEPEMILIKAN GANDA** | §4: crypto=agent10, gate=agent5 |
| P3-04 | `SEC-08` di spec agent5 masih teks lama | milik agent5, ditandai | tetap ditandai |
| P3-05 | `I18N-40 <500KB` belum diukur | **AMBANG TANPA DASAR** | L5: ukur dulu, atau tandai sementara |

**Yang v3.0 sudah benar dan saya pertahankan tanpa ubah:** StickyNote 46%
(79/171, terverifikasi di `ANALISIS-KORPUS-NODE.md:72,193`), `swarm-task`
(terpasang nyata), jumlah 198 berkas, struktur 4-Wave, pemisahan BLAKE3
(Envelope) vs SHA-256 (spill) — **bukan** kontradiksi, saya hampir salah menuduh
ini dan koreksinya ada di #634.

---

## 10. Protokol penandatanganan

**Status konfirmasi sebenarnya** (menggantikan §6.1 v3.0 yang tidak akurat):

| Pihak | Status |
|---|---|
| matt (Lead Architect) | **BELUM** — dokumen ini adalah syarat yang saya ajukan; saya tanda tangani bila §9 butir 1–5 + P3-01/02/03 dibereskan |
| agent1 | mendukung urutan (#645), belum sign-off formal |
| agent2 | mendukung 5 masalah wajib diperbaiki (#638), belum sign-off formal |
| agent10 | menandai P3-01 **BLOCKER**; menolak mengesahkan gate yang ia usulkan sendiri (#628) — **saya dukung penolakan itu** |
| agent3, agent4, agent5, agent6, agent7, agent8, agent9 | belum tercatat |
| fern (Supervisor) | penerbit v3.0 |
| **Pemilik Proyek** | **BELUM — dan ini yang menentukan** |

**Urutan yang saya usulkan:**

1. Pemilik Proyek menjawab §8 (minimal K-3, K-8, K-4).
2. Wave 0 (§4) dikerjakan — lima tugas remediasi, semuanya sudah punya bukti cacat.
3. Draf ini direvisi sesuai jawaban, lalu tiap lead menandatangani eksplisit.
4. Wave 1 dibuka.

**Prinsip yang saya pegang dan saya usulkan jadi aturan tim:** *recommender ≠
approver.* agent10 menolak mengesahkan gate yang ia usulkan sendiri (#628) dan
saya dukung. Saya terapkan simetris: saya tidak akan mengesahkan gate yang
menguji artefak saya sendiri. Untuk G-C7 khususnya — gate itu menguji testkit
(agent3) vs data-plane (agent2), jadi saya bukan pengusul maupun pemilik; saya
bisa mengesahkannya, dan verdict saya saat ini: **G-C7 GAGAL**.

Pemilik Proyek sudah mengidentifikasi pelajaran ini sendiri: 88 keputusan pernah
disetujui tanpa review adversarial, oleh pihak yang sama yang mengusulkannya.
Dokumen ini ada karena pola itu hampir terulang pada §6.1 v3.0.
