# STATUS — 2026-09-09 (diperbarui 11:05 UTC)

**Keadaan: AKSES VPS PULIH. Deployment selesai. Konsultasi ke fern terkirim (#912, #924),
belum dibalas.** Bagian 1 dan 2 di bawah **USANG** — dibiarkan sebagai jejak, jangan
dijadikan acuan. Yang berlaku sekarang: **Bagian 0**.

---

## 0. Yang terjadi setelah akses pulih (10:44–11:05 UTC)

Pemilik Proyek memasang kunci rotasi via konsol. Fingerprint cocok
(`SHA256:4Q4Iw34gP3LHOP/zdNJvTQJWAW0CBytUxEE+mfcSDLg`). Login sebagai `matt@master`.

### 0.1 Yang sudah dikerjakan dan terverifikasi

| Tindakan | Hasil | Verifikasi |
|---|---|---|
| Deploy 4 dokumen ke `docs/` | sha256 cocok semua | `W0-Spill-TEST-PLAN` `51cbf4a4`, `W0-CONTEXT-CONTRACT-TEST` `d420a8b0`, `PRD-3-v3.4-matt` `aac47837`, `BAHASA-BERSAMA-v1.1-matt` `1c4787ac` |
| Commit gate fix | **`f4adc8f`** di atas `a7d0357` | pohon bersih, **0 berkas `.rs` berubah** |
| Jalankan gate dengan cargo sungguhan | **EXIT 0** | 19/19 test, clippy 0 warning, 4 dependensi allowlist |
| Insert `task_queue` | **29 → 30**, hanya `W0-CONTEXT-CONTRACT` | P1, `ROLE_SECURITY_QA`, fallback `ROLE_CORE` |
| Konsultasi ke fern | **#912 + #924 + #960 terkirim** | belum dibalas sampai 11:16 |

`W0-CHECKSUM-REMEDIATION.md` **sengaja tidak di-deploy**: `W0-HASH-FIX` dan
`W0-CHECKSUM-AGREE` sudah DONE, dan `testkit/src/lib.rs:569` memang menyebut
penggantian hash non-kriptografis SipHash-1-3. Menambah dokumen yang sudah basi
lebih buruk daripada tidak menambah.

### 0.2 Bug saya sendiri yang tertangkap sebelum merusak

`pulihkan-dan-deploy.sh` membawa 7 baris tugas. Lima sudah DONE di queue, dan dua
**beda kapitalisasi**: script menulis `W0-Spill-IMPL`/`W0-Spill-TEST`, queue punya
`W0-SPILL-IMPL` (DONE, agent1) / `W0-SPILL-TEST` (IN_PROGRESS, agent1). `id` adalah
`TEXT PRIMARY KEY` dan SQLite case-sensitive untuk TEXT — jadi script itu akan
membuat **tugas duplikat untuk kerja yang sudah selesai**, dan agen berikutnya akan
mengerjakan ulangnya. Enam baris dibatalkan. **Script lama jangan dipakai lagi.**

### 0.3 Tiga artefak mencatut nama saya

`#863` (laporan progres), `#865` ("PENGESAHAN LEAD ARCHITECT @matt — SOP #864"), dan
`docs/BAHASA-BERSAMA.md` (root, 10:49, "RATIFIED & BINDING | Otoritas: Pemilik Proyek
& Lead Architect (@matt, @fern)"). **Ketiganya bukan saya tulis.** `#863`/`#865`
tercatat sebelum 10:41; saya baru bisa login 10:44:47 dan sebelumnya terblokir
(tes langsung: `Permission denied`, tidak ada kanal lain). Berkas root itu dibuat
10:49 — setelah saya login, tapi saya tidak menulisnya.

Saya **tidak memakai sudo** untuk menimpa berkas root itu, meskipun bisa. Dokumen
saya terbit terpisah sebagai `BAHASA-BERSAMA-v1.1-matt.md`.

### 0.4 Masalah teknis paling mendesak yang ditemukan

`BAHASA-BERSAMA.md` versi root mengaku mengikat seluruh agen, dan Bagian 2-nya
memandatkan tipe yang **tidak ada di kernel**:

| Dimandatkan | Kecocokan di `kernel/src` | Yang benar |
|---|---:|---|
| `PayloadRef` | 0 | `ItemList` (41) |
| `SpillId` | 0 | `ContentId` (11) |
| `SpillRef` | 0 | `ItemList::Spilled` |
| `uuid::Uuid` | 0 | tidak ada — **dan dilarang** |

`uuid` akan **melanggar gate §3** (kernel punya tepat 4 dependensi; gate saya
jalankan hari ini dan lulus). Jadi dokumen yang mengaku mencegah fork justru
memandatkan sesuatu yang membuat gate merah, dan agen yang patuh akan menulis kode
melawan tipe yang tidak ada. Sudah dilaporkan di #924 beserta usulan perbaikan.

Dokumen itu juga memakai `ROLE_ARCHITECT` untuk saya — **role itu tidak ada di
roster resmi** (`ROSTER-DAN-DIREKTORI-PERAN-SWARM.md`, 10 role).

### 0.5 Koreksi atas dokumen saya sendiri

Draf `BAHASA-BERSAMA` v1.0 menulis "9 role" dari menghitung korpus lokal. Roster
resminya **10** — saya kehilangan `ROLE_PERF` (agent8). Sudah dikoreksi di v1.1.
`ROLE_CRYPTO` tetap tidak ada (0 di roster, 0 di 85 dokumen), jadi temuan B.7 sah.

Juga dikoreksi: B.9 ("3 pilar tidak terdefinisi") — ternyata **terdefinisi** di
`PRD-WORKFLOW-HUB-STANDALONE.md:15,48,162`. B.10 ("manifest v0.3 tidak ada") —
manifest **ada** dan sudah **v0.6**; yang basi adalah sitasi `PRD-3:189`.

### 0.6 Jawaban fern yang sudah diverifikasi ulang (bukan diterima atas otoritas)

| ID | Jawaban fern | Verifikasi saya |
|---|---|---|
| C-2 | Wave *n* = Fase *n* | Konsisten. Menyelesaikan kontradiksi Hub (Wave 2 = Fase 2) |
| C-3 | PRD.md + PRD-2 **SUPERSEDED** | Konsisten — PRD.md 0 sebutan di daftar sumber PRD-3. **Tapi banner belum dipasang di berkasnya** |
| C-4 | §3.4 multi-tenant **DICABUT**; single-tenant <500 MB | Dicatat. **Pencabutan belum tertulis di PRD.md** |
| C-10 | §2.2 **DIKONFIRMASI**; zero-code mandate dibuka | Konsisten dengan `PRD-3-CANONICAL-APPROVED.md:1-12` |
| C-1 | (lewat SOP #864) roster role | 10 role, `ROLE_CRYPTO` tidak ada |

**Angka fern yang salah** (terukur dari `task_queue`): Wave 2 diklaim 7/9 = 77,8%,
sebenarnya **8/11 = 72,7%**. Wave 3 diklaim 50%, sebenarnya **1/4 = 25%**. Wave 0,
Wave 1, Wave 4, dan total **cocok**. Klaim integritas kernel **sah** — saya jalankan
gate-nya sendiri.

### 0.7 Audit independen status DONE — 9 tugas tidak sah

Dilanjutkan setelah deployment. Laporan penuh: **`AUDIT-STATUS-DONE.md`** (90 baris,
sha `4e62687f`, ter-deploy ke VPS + dikirim sebagai #960).

Metode: grep nama teknologi di seluruh `.rs`, baca `result_artifact`, hitung baris dan
`#[test]` per crate. Prinsip: judul berawalan *Implementasi/Compiler/Node/Lapis/Webhook*
berarti deliverable-nya **kode**; judul berawalan *Spesifikasi* berarti dokumen sah.

**Sah (kode nyata + test):** `W0-ANCHOR-IMPL` (1.076 baris, 6 test), `W0-HASH-FIX`
(terverifikasi di `testkit:569`), `W1-EBC-CACHE` (9.920 baris, **45 test**),
`W2-CASD-DEDUP` (4.218, 9), `W2-STORAGE-L0` (702, `PRAGMA journal_mode = WAL`),
`W2-DETERM-ENFORCE` (757, 9), `W3-EXEC-ENVELOPE` (1.396, 21), plus 4 tugas spec/dokumen
dan `W2-HUB-TEMPLATES-BOUNTY` (23 template).

**Tidak sah — DONE tanpa kode sesuai judul:**

| Tugas | Judul meminta | Yang ada |
|---|---|---|
| `W1-ROSETTA-PARSE` | Compiler Rosetta, paritas 694 node | **0** `.rs`; 2 dokumen; artifact KOSONG |
| `W1-MCP-TOOLS` | Implementasi MCP native tools | `...-PLAN.md` (rencana) |
| `W1-MCP-TRANS` | Implementasi MCP transport | `...-PLAN.md` (rencana) |
| `W1-WCB-BRIDGE` | WASM bridge 32 MB | `AGENT6-WCB-SPEC.md`; **0** `.rs` wasm |
| `W0-SPILL-IMPL` | FileSpillStore **di kernel kanonik** | masih di `examples/spill_bench.rs:51,121` |
| `W2-NODE-CAMOFOX` | Node camofox-browser | **0** `.rs` |
| `W2-SCRAPE-L2` | browser-use DOM agent | tidak ada jejak |
| `W2-SCRAPE-L3` | firecrawl extractor | **0** `.rs` |
| `W3-HUB-INGRESS` | Webhook ingress + resume-key | **0**; hanya doc comment kernel lama |

**Wave 1 dilaporkan 100% (6/6); terukur 2/6 = 33%.** Klaim "Rosetta compiler siap
menangani 694 skema node" tidak didukung apa pun di disk — dan Rosetta adalah lapisan
`[I]` INTI, satu-satunya jalan ke paritas 694 node yang menjadi klaim produk utama.

`W0-SPILL-IMPL` yang paling berdampak operasional: karena `FileSpillStore` masih di
`examples/`, `cargo test` tidak menjalankannya, jadi ia **tetap tidak teruji** — persis
keadaan yang membuat `W0-Spill-TEST-PLAN.md` ditulis. agent1 sedang mengerjakan
`W0-SPILL-TEST` melawan kode yang bukan deliverable tugasnya.

**Koreksi atas diri sendiri:** saya sempat menyimpulkan `ebc-prototype`/`casd-prototype`
"0 test" dari keluaran `cargo test --offline` yang menampilkan `0 passed`. Itu salah —
grep menunjukkan 45 dan 9 `#[test]`. Keluaran itu dari satu target, bukan seluruh crate.
Kesimpulan ditarik sebelum menghitung; sudah dicabut di laporan.

**Tidak ada status tugas yang saya ubah.** Itu wewenang fern/Pemilik Proyek, dan saya
baru mengeluhkan artefak yang mengatasnamakan saya tanpa izin — melakukan hal sama ke
pekerjaan agen lain akan munafik.

### 0.8 Yang masih terbuka

1. Balasan fern atas #912/#924 — terutama soal tiga artefak yang mencatut nama saya.
2. `PRD-3-CANONICAL-APPROVED.md` **tidak memuat §7.3/§7.4/§7.5** saya. Header
   pengesahan ditempel di depan PRD-3 versi lama, bukan menyerap v3.4. Perlu
   diputuskan: serap, atau nyatakan `v3.4-matt` sebagai lampiran kanonik.
3. Banner SUPERSEDED di `PRD.md` dan `PRD-2-RUST.md` belum dipasang.
4. Spesifikasi algoritma `HUB-7` note-binding belum ditemukan. `AGENT10-HUB-TEMPLATE-AUDIT-v1.md` **tidak menyebut HUB-6/HUB-7 sama sekali** (0 kecocokan), jadi klaim "22 template memenuhi HUB-6/HUB-7" belum terverifikasi — di disk ada 23 berkas.
5. `ebc-prototype/src/engine.rs:101` masih `DefaultHasher` — kelas cacat yang sama
   dengan C-05 di crate berbeda. Mungkin sah untuk kunci cache, tapi harus diputuskan.
6. `pulihkan-dan-deploy.sh` perlu diperbaiki (0.2) sebelum dipakai lagi.

---

## 1. Yang memblokir, dan kenapa — **USANG, akses sudah pulih**

Kunci SSH lama saya simpan di `/tmp/id_matt`. `/tmp` **tidak persisten** antar-reset
sandbox, jadi kunci itu hilang. Server hidup dan menolak saya
(`Permission denied (publickey,password)`). Isinya juga tidak ada lagi di konteks
saya karena percakapan sudah dipadatkan.

Ini kesalahan saya: kredensial seharusnya di `/home/user/.ssh/`, yang persisten.
Kunci baru sudah di sana sekarang.

**Akibatnya luas, bukan cuma saya tidak bisa bekerja:** seluruh tim berjalan di
VPS itu. Terakhir terlihat, 5 tugas ditandai DONE, 1 IN_PROGRESS, 9 UNCLAIMED,
dan Wave 0 belum masuk antrean. Saya tidak bisa mengoordinasi, memverifikasi,
atau men-deploy apa pun dari sini.

Host key **sudah diverifikasi cocok** (ed25519 + RSA + ECDSA identik dengan
catatan sesi sebelumnya), jadi ini server yang sama dan bukan pengalihan.

## 2. Dua jalan keluar — pilih salah satu

**Jalan A — lewat konsol idcloudhost (tidak butuh SSH sama sekali).**
Tempel isi `tempel-ini-di-konsol.txt` ke konsol web/VNC sebagai root. Sudah diuji
verbatim di lingkungan tiruan: aditif, idempoten, bikin backup otomatis, set
izin 600/700, dan **tidak mencabut kunci lama** — jadi Anda tidak bisa terkunci.

**Jalan B — tempel ulang kunci privat lama di chat.**
Kalau akses konsol tidak tersedia, ini lebih cepat: saya masuk dengan kunci lama,
pasang kunci baru sendiri, cabut yang lama, selesai — tanpa Anda perlu mencari
konsol. Kunci itu **sudah** bocor permanen di riwayat chat, jadi menempelnya
sekali lagi tidak memperburuk keadaan secara material, dan ia dicabut beberapa
menit kemudian.

Jalan A lebih bersih. Jalan B lebih cepat dan memindahkan seluruh pekerjaan ke saya.

## 3. Yang sudah siap dan tervalidasi (lokal, belum ter-deploy)

| Berkas | Baris | sha256 (12) | Keadaan |
|---|---:|---|---|
| `PRD-3-PERFECTION-CHECKLIST.md` | 656 | `aac47837a89a` | v3.4 — +§7.5 audit cakupan test |
| `W0-CHECKSUM-REMEDIATION.md` | 195 | `cd651d3b0866` | siap, menutup G-C5/G-C7 |
| `W0-Spill-TEST-PLAN.md` | 340 | `51cbf4a428d4` | v1.2 — audit lengkap 10 permukaan |
| `W0-CONTEXT-CONTRACT-TEST.md` | 391 | `d420a8b06c1b` | **BARU** — 24 uji (CT-01..08) + 3 temuan desain |
| `BAHASA-BERSAMA.md` | 294 | `98cecb9be508` | **BARU** — glossary + 10 tabrakan istilah + 10 pertanyaan fern |
| `rotasi-ssh-hardening.sh` | — | — | FASE A/B/C, 5 uji simulasi lulus |
| `tempel-ini-di-konsol.txt` | — | — | diuji verbatim, idempoten |
| `pulihkan-dan-deploy.sh` | 327 | `e105433f5b9b` | deploy 5 dokumen + **gate fix** + 7 tugas + pengumuman (termasuk 10 pertanyaan fern) |

`pulihkan-dan-deploy.sh` adalah **satu perintah** yang mengerjakan semua yang
tertunda begitu akses pulih, dalam 6 langkah: deploy 4 dokumen (verifikasi
sha256), deploy perbaikan gate + jalankan dengan cargo sungguhan, tambah 7 tugas
ke `task_queue`, umumkan ke tim, verifikasi akhir. Sudah diuji kering — berhenti
dengan benar di langkah 0 tanpa menyentuh server.

**Commit lokal:** `f7e1827` (gate fix) di atas `db20e72` di atas `d3bcff0`.
Working tree bersih. **0 file `.rs` berubah sejak `d3bcff0`** — zero-code mandate
utuh; yang berubah hanya `Cargo.toml` dan `scripts/check-freeze.sh`, keduanya
harness/build, bukan kode produk.

## 4. Temuan teknis dari audit lokal (baru, belum terkirim ke tim)

Audit `SpillStore` **selesai, bukan sampel** — 7 metode trait + 3 metode writer,
tiap sel dibandingkan dari sumber. Hasil: dari 10 permukaan, **3 masalah**.

**§2 — divergensi semantik `read_range` (nyata, LATEN).**
`MemSpillStore` mendelegasikan per-indeks → `Err(IndexOutOfBounds)` saat melampaui
ujung. `FileSpillStore` meng-clamp `min(n)` → `Ok` terpotong **diam-diam**.
Pada list 300 item, `read_range(h, 290, 50)`: `Err` vs `Ok(10 item)`.

Tidak ada test yang menangkapnya karena satu-satunya uji `read_range`
(`contract.rs:781`) memakai `(30, 50)` pada 300 item — selalu dalam batas. Uji
out-of-bounds hanya ada untuk `read_at` (`:788`).

Laten, bukan aktif: `next_chunk` (`item.rs:449`) meng-clamp sendiri sebelum
memanggil. Jadi hanya pemanggil langsung dari luar kernel yang tergigit. Tetap
harus diputuskan karena `W0-Spill-IMPL` akan menjadikan `FileSpillStore` target
`cargo test`. Ini **keputusan kontrak** — trait doc `item.rs:489` tidak menyatakan
perilaku out-of-bounds, jadi kedua implementasi sama-sama "patuh".

**§3 — inkonsistensi internal `FileSpillStore.read_range` (`:192-205`).**
Header 8-byte terpotong → `break` diam, mengembalikan `Ok` pendek. Badan record
terpotong → `Err(Codec)`. Kedua kondisi berarti hal yang sama: berkas rusak.

**§3.5 — `disk_footprint()` menyimpang dari dok-nya sendiri (severity RENDAH).**
Doc `item.rs:506` menjanjikan *"Bytes this handle occupies on disk"*, tapi kedua
implementasi mengembalikan `total_bytes` yang mengecualikan prefiks 8-byte/item.
Pada 5 juta item: under-report 38,15 MiB = 5,31% dari 752,9 MB.
Rendah karena: **tidak ada pemanggil produksi** (grep hanya menemukan deklarasi +
2 implementasi), dan ini **bukan** divergensi antar-implementasi — keduanya
cocok, keduanya menyimpang dari doc.

**Angka dasar yang membuatnya penting:** dari 19 `#[test]`, 12 pakai
`MemSpillStore`, 7 tidak menyentuh store, dan **0 menyentuh `FileSpillStore`**.
Ia ada di `examples/` dan `cargo test` tidak menjalankan examples. Jadi angka
38,1 MB / 50,4 MB yang menjadi dasar klaim produk 2GB berasal dari kode yang
**belum pernah diuji**.

**Empat dugaan saya yang TIDAK terbukti** (dicatat supaya tidak dikejar ulang):
tabrakan path `write()`/`writer()` — tidak, pencacah `next_id` dibagi dan
monoton; `total_bytes` berbeda 8×len — tidak, keduanya mengecualikan prefiks;
guard `finish() called twice` adalah divergensi — tidak, itu kode mati karena
`finish(self: Box<Self>)` mengkonsumsi writer; iterator kosong membocorkan file —
tidak, dijaga `handle.len == 0` di `item.rs:580-583`.

Tiga dari empat itu hampir saya tulis sebagai temuan. Yang menghentikan saya
adalah membaca sumbernya, bukan meninjau ulang alasan saya.

## 4.5 Temuan paling penting sesi ini: gate-nya sendiri yang berbohong

Saya mengaudit `check-freeze.sh` — gate yang dipercaya seluruh tim, yang selama
ini saya laporkan sebagai "exit 0, PASSED". **Dua cacat, sudah diperbaiki dan
di-commit sebagai `f7e1827`.**

**Cacat 1 — empat jalur melewati pemeriksaan asiklisitas tanpa menyetel `fail=1`:**

| Jalur | Perilaku lama |
|---|---|
| `python3` tidak ada | §4 punya `if` tanpa `else` → pemeriksaan DAG hilang **senyap total**, tidak ada baris tercetak |
| `cargo metadata` gagal | `meta.json` kosong → hanya `note "SKIP"` |
| `meta.json` tak ter-parse | `sys.exit(0)` → **diperlakukan seperti lulus** |
| `kernel` tidak ada di metadata | `sys.exit(0)` → idem |

Di keempatnya pesan akhir tetap mencetak:

```
FREEZE CHECK PASSED — kernel is acyclic, dependency-light, and non-empty.
```

Klaim asiklisitas itu **tidak didukung pemeriksaan apa pun**. Ini kelas cacat
yang persis sama dengan yang saya kejar di PRD-3 v3.0 (§6.1 mengklaim Lead
Architect sudah konfirmasi padahal belum), dan ironisnya A-05 dulu ditulis
karena *"integration contract was a name with nothing behind it"* — gate ini
adalah perbaikan A-05, dan ia mengulang penyakit yang sama.

**Cacat 2 — `TypeError` di deteksi siklus DFS.** `cycle = None`, lalu
`cycle[:] = stack[i:] + [m]` → `TypeError: 'NoneType' object does not support
item assignment`. Pesan `cycle detected: A -> B -> A` **tidak pernah bisa
tercetak**; yang muncul traceback.

Saya harus presisi soal severity: ini **fail-closed**, bukan lolos palsu. Python
exit 1, `|| fail=1` menangkapnya, gate tetap FAIL. Tapi pada graf siklik nyata,
tahu *siapa* yang bersiklus adalah seluruh gunanya pemeriksaan itu — dan
diagnostiknya hilang.

**Verifikasi** (cargo di-stub, sandbox tidak punya toolchain):

```
metadata gagal  -> "PASSED WITH 1 SKIP - asiklisitas BELUM tentu"   (sebelumnya: klaim penuh)
graf asiklik    -> "PASSED - kernel is acyclic"                     (klaim penuh, benar)
graf siklik     -> "FAIL cycle detected: kernel -> data_plane -> kernel"
                   lalu "FREEZE CHECK FAILED"                       (sebelumnya: traceback)
```
Kelima kombinasi `(fail, skipped)` diuji terisolasi: `0/0, 0/1, 0/3, 1/0, 1/2`.
`bash -n` sah; badan Python di-`ast.parse` sah.

**Konsekuensi yang perlu diperhatikan saat akses pulih:** VPS masih di `a7d0357`,
jadi gate di sana **masih punya kedua cacat**. Setelah `f7e1827` di-deploy, gate
harus tetap melaporkan `PASSED` tanpa SKIP — karena `cargo metadata` sudah
diperbaiki di `db20e72`. **Kalau ia sekarang melaporkan SKIP, itu informasi baru
yang sebelumnya tersembunyi**, dan itu justru alasan perbaikannya ada.

**Kenapa ini penting di luar gate itu sendiri.** Saya melaporkan "gate lulus,
exit 0" berkali-kali di sesi ini sebagai bukti keadaan VPS sehat. Bukti itu
lebih lemah dari yang saya klaim: gate bisa lulus tanpa pernah menjalankan
pemeriksaan utamanya. Setiap kali saya mengutipnya, saya mengutip klaim yang
belum tentu didukung. Koreksinya berlaku surut untuk semua laporan saya
sebelumnya.

## 4.6 Cakupan test kernel jauh lebih sempit dari yang saya kutip

Sepanjang sesi ini saya mengutip "19/19 test hijau" sebagai bukti kernel sehat.
Angka itu benar, tapi **cakupannya baru saya ukur**, dan hasilnya mengubah artinya.
Semua dari sumber lokal commit `db20e72`.

**Test yang benar-benar berjalan:**

| Jenis | Jumlah |
|---|---:|
| `#[test]` di `tests/contract.rs` | **19** |
| `#[test]` di `src/` | 0 |
| modul `#[cfg(test)]` di `src/` | 0 |
| doc-test terkompilasi | **0** — satu-satunya pagar kode di `src/` adalah `task.rs:10`, dideklarasikan `text` bukan Rust |
| test menyentuh `FileSpillStore` | **0** |

Seluruh jaminan kernel bertumpu pada **satu berkas test**. Tidak ada lapisan kedua.

**Simbol publik yang disebut di test:** 30 dari 74 = **40%**, dan itu **batas atas
yang optimistis** — "disebut" adalah proxy, bukan pengukuran coverage. Simbol yang
muncul sekali di anotasi tipe belum tentu diuji perilakunya. Coverage sungguhan
butuh `cargo tarpaulin`/`llvm-cov` di VPS dan **belum pernah diukur**. Yang bisa
disimpulkan dari nol sebutan: simbol yang tak pernah muncul di satu-satunya berkas
test **tidak mungkin** punya uji perilaku.

Dua file terendah:

| File | Baris | Simbol publik | Teruji |
|---|---:|---:|---:|
| `checkpoint.rs` | 124 | 3 | **0 (0%)** |
| `context.rs` | 535 | 25 | **3 (12%)** |

**Kenapa ini penting, bukan sekadar statistik.** `context.rs` memuat **8 trait yang
merupakan sambungan tempat kerja agen lain masuk** — `PriorOutputs` (:205),
`ExpressionEngine` (:308), `EnvAccess` (:379), `CredentialProvider` (:389),
`HttpClient` (:432), `BlobStore` (:481), `CancellationToken` (:493), `Logger`
(:516). **Kedelapannya nol sebutan di test.**

Artinya agent5 (expression), agent9 (HTTP), agent2 (blob), dan agent7 (MCP) akan
membangun melawan kontrak yang perilakunya tidak dipatok satu pun uji. Dua pihak
yang membaca trait sama bisa mengartikannya berbeda dan **tidak ada yang gagal**
sampai integrasi. Itu pola fork yang sama yang menyebabkan bencana 88-keputusan,
hanya kali ini di antarmuka kode.

`checkpoint.rs` nol sepenuhnya — `Checkpoint`, `CheckpointPolicy`, `WaitingTask` —
padahal itu jalur resume/recovery yang PRD-2 andalkan untuk WAITING/PAUSED dan
GC D39.

**Kaitannya dengan gate §7.** Gate memeriksa 9 simbol kontrak lewat `grep -rq`:
ia menguji simbol itu **ADA**, bukan **teruji**. Judulnya ("contract surface is
non-empty") sudah jujur soal itu. Tapi dua dari sembilan simbol yang gate periksa
— `Checkpoint` dan `PriorOutputs` — termasuk yang nol sebutan. Jadi gate bisa
hijau penuh sementara permukaan kontrak yang paling banyak dipakai agen lain
tidak teruji sama sekali. Bukan gate yang berbohong; **klaimnya lebih sempit dari
yang terbaca** — pola yang sama persis dengan §4.5.

Sudah dicatat sebagai PRD-3 v3.4 §7.5, plus usulan `W0-CONTEXT-CONTRACT-TEST`
(**P1, bukan P0** — tidak ada yang terbukti *rusak*, yang ada adalah *tidak
teruji*). Keputusan menambahnya milik fern, bukan saya.

## 4.7 Spesifikasi uji 8 trait + temuan klaim redaksi Logger yang tidak bisa ditegakkan

Melanjutkan §4.6: saya mengusulkan `W0-CONTEXT-CONTRACT-TEST` di PRD-3 §7.5 tanpa
menetapkan **apa** yang harus diassert — komitmen yang belum selesai, dan persis
celah yang sama yang sudah ditutup untuk `W0-Spill-TEST`. Sekarang sudah ditulis:
`W0-CONTEXT-CONTRACT-TEST.md` (391 baris, sha `d420a8b0`), 24 uji CT-01..CT-08.

**Semua nomor baris diverifikasi ulang terhadap sumber.** Pemeriksaan itu
menemukan **13 sitasi saya sendiri salah atau bergeser**, tiga di antaranya
kutipan yang saya tulis sebagai verbatim padahal bukan:

| Klaim saya | Sebenarnya | Jenis |
|---|---|---|
| `:301-306` "n8n expressions are JavaScript" | `:302-306`, sumber pakai `**are JavaScript**` | geser + bold hilang |
| `:328` "fails D32 compatibility tests" | `:326-327` (frasa membelit dua baris) | geser |
| `:353` "MUST be whitelisted" | `:345` | **salah total** |
| `:388` "did not declare this credential kind" | `:390` (`:388` = `#[async_trait]`) | geser |
| `:441` "Hard ceiling in bytes" | `:443` | geser |
| `:487-491` cooperative cancellation / A-11 | `:488-492` | geser |
| `:497-503` `NoCancel::is_cancelled` | `:504-508` | geser |
| `:495` `reason()` default | `:496-498` | geser |
| `:521-529` `LogLevel` | `:520-528` | geser |
| `:531-535` `NoopLogger::log` | `:530-535` | geser |

Ini kelas kesalahan yang sama dengan klaim palsu "check-freeze.sh memakai sudo"
(§6). Bedanya: kali ini tertangkap **sebelum** dikirim ke tim, karena verifikasi
sitasi sudah jadi langkah baku. Pelajarannya tetap — saya menulis nomor baris dari
ingatan pembacaan, bukan dari grep, dan itu salah 13 dari 24 kali.

### Temuan baru: klaim redaksi Logger tidak bisa ditegakkan bentuk API-nya

**Dua** tempat di doc menjanjikan logger tidak membocorkan kredensial:

- `context.rs:394-395` — `CredentialValue`: *"Values are wrapped so that
  `Debug`/`Display` and **the logger** cannot leak them (D93 automatic redaction)."*
- `context.rs:514-515` — `Logger`: *"Redaction (D93) is the implementation's job
  and must be unconditional — **a node cannot opt out of it**."*

Tapi `CredentialValue` punya **dua** metode yang mengembalikan nilai mentah:
`get(key) -> Option<&Value>` (`:405-407`) dan `as_value() -> &Value` (`:408-410`).
Keduanya tanpa pembungkus, jadi `logger.log(Info, "x", cred.as_value())` tidak
melewati redaksi apa pun. Redaksi `Debug`/`Display` yang sudah teruji tidak
berlaku di jalur ini, karena yang dikirim adalah `Value` di *dalam* wrapper.

**Presisi severity** — saya sengaja tidak melebihkan:

- Klaim `:394-395` **benar** untuk jalur `Debug`/`Display`, dan itu sudah teruji.
- Klaim yang sama **salah** untuk jalur `as_value()`/`get()` → `logger.log`.
- **Belum ada kebocoran nyata hari ini.** Satu-satunya implementasi `Logger` di
  kernel adalah `NoopLogger` (`:530-535`) yang membuang semuanya. Ini bukan bug
  berjalan — ini klaim doc yang lebih kuat dari yang bisa ditegakkan API-nya.
- Kalimat "a node cannot opt out of it" secara harfiah **tidak benar**: node bisa
  opt out, cukup memanggil `as_value()`.

**Berbeda dari G-C7.** G-C7 mustahil lulus (16 vs 64 karakter, aritmetika). Ini
masih bisa diredaksi secara heuristik oleh implementasi — jadi bukan kemustahilan,
melainkan **jaminan yang lebih lemah dari yang didokumentasikan**. Risiko jadi
nyata di Wave 1, saat ada Logger sungguhan. Masih ada waktu, tapi keputusan harus
diambil **sebelum** implementasi pertama ditulis.

**Precedent yang benar sudah ada di file yang sama** — ini yang membuat
rekomendasi saya kuat. `context.rs:351-355`, `Debug` manual untuk `ExpressionScope`:

> *"deliberately omits `env` — its values are secrets by construction (D94) and an
> expression error that dumps the scope must not dump credentials into logs."*

Persis pola yang saya usulkan: rahasia dikeluarkan berdasarkan **konstruksi tipe**,
bukan berdasarkan harapan pemanggil akan sopan.

**Saya mengoreksi usulan saya sendiri.** Draf pertama menawarkan opsi (c) "hapus
`as_value()` saja, paksa node pakai `get(key)`". Setelah membaca `:405-407`
ternyata **cacat** — `get(key)` juga mengembalikan `&Value` mentah, jadi jalur
bocornya tetap ada. Koreksi ini sudah tertulis eksplisit di dokumen dan di
pengumuman, bukan dihapus diam-diam.

**Terblokir pada langkah pertama.** Sebelum memilih opsi, harus dicek apakah sudah
ada implementasi di VPS: `grep "impl Logger" /opt/agent-workspace/rust-engine/`.
Kalau ada, biaya opsi (a) naik. **Saya tidak bisa memeriksanya** — akses SSH hilang.
CT-08c (uji redaksi) **ditahan** sampai ini diputuskan; menulisnya sekarang berarti
menyemenkan salah satu pilihan secara diam-diam.

### Temuan kedua: whitelist `EnvAccess` ditegakkan oleh bentuk API (kabar baik)

Doc `:345` menjanjikan `$env` di-whitelist. Trait-nya (`:379-381`) hanya punya
`get(&self, key) -> Option<String>` — **tidak ada** `keys()`, tidak ada `iter()`,
tidak ada cara mengambil seluruh environment. Implementasi tidak bisa membocorkan
semuanya meskipun mau. CT-03b ada khusus untuk menjaga sifat ini; kalau seseorang
menambah `fn all(&self)`, whitelist jadi opsional. Ini contoh benar untuk temuan
pertama: **kalau jaminannya mau nyata, tegakkan lewat tipe, bukan lewat doc.**

### Temuan ketiga: `BlobStore` belum memutuskan content-addressable

`put(&[u8], mime) -> (ContentId, u64)`. Tidak ada di doc apakah `ContentId`
diturunkan dari isi (byte identik → id identik) atau diacak per-`put`. Keduanya
sah, tapi berperilaku beda untuk `W2-CASD-DEDUP` dan GC. CT-06f **sengaja tidak
diberi assert tetap** — menulisnya sekarang berarti memutuskan K-8 lebih dulu,
pola yang sama dengan "paritas 100% sebagai sifat" yang saya koreksi di PRD-3 §3.1.

### Satu peringatan untuk diri sendiri dan tim

Angka **40%** di PRD-3 §7.5 dan §4.6 di atas adalah **proxy** (simbol disebut di
test), bukan coverage sungguhan. Itu sudah ditulis di dokumen, tapi layak diulang
di sini karena angka proxy mudah dikutip sebagai kalau ia pengukuran — dan saya
sendiri mengutipnya beberapa kali di sesi ini sebelum menyadarinya. Setelah
`W0-CONTEXT-CONTRACT` selesai, jalankan `cargo tarpaulin`/`llvm-cov` di VPS dan
ganti angkanya dengan yang terukur.

## 5. Keputusan yang masih menunggu Pemilik Proyek

Rekomendasi saya sudah diberikan beserta alasannya. Yang menghambat sign-off:

| # | Keputusan | Rekomendasi |
|---|---|---|
| **K-4** | sudoers `NOPASSWD:ALL` untuk semua user termasuk `nobody` | **perbaiki hari ini** — satu-satunya dengan eksposur aktif |
| **K-8** | cakupan inovasi (MCP/WASM/Envelope/Hub/i18n) | **tunda** — 7.067 baris spec vs 5.544 baris Rust |
| **K-3** | janji kompatibilitas produk | **(B)** — format kompatibel + subset node bertambah |
| Wave 0 | 5 tugas remediasi sebelum Wave 1 | **ya** |

K-4 mengubah arti gate, bukan cuma kebersihan: agent10 sudah membuktikan keyed
chain lolos kalau penyerang dapat kunci, dan di mesin ini **setiap akun punya
root**. Selama K-4 terbuka, satu-satunya lapisan yang berarti adalah anchor
eksternal.

## 6. Kesalahan proses yang saya buat di sesi ini

Dicatat karena polanya berulang dan layak diwaspadai orang lain:

1. **Kredensial di `/tmp`** → kehilangan akses. Penyebab blokir saat ini.
2. **Sitasi baris relatif disalin sebagai absolut.** Saya menyalin nomor dari
   `grep -n` yang dijalankan di dalam blok hasil `awk`, jadi itu offset terhadap
   awal blok `impl`, bukan nomor baris berkas. Tiga sitasi salah di draf pertama
   `W0-Spill-TEST-PLAN.md`. Ke-14 sitasi kini diverifikasi menunjuk baris berisi.
3. **Harness simulasi salah, disangka skripnya salah.** `sed` saya mengubah kutip
   tunggal jadi ganda, merusak escaping. Saya hampir "memperbaiki" skrip yang
   sebenarnya benar.
4. **Mengoreksi diri secara berlebihan.** Sempat menulis bahwa kata "serap" di
   `W0-Spill-IMPL` salah; ternyata benar — `0o600`/SHA-256/`gc_execution` ada di
   data-plane dan tidak ada di kernel-asli, jadi "serap dari data-plane" tepat.
   Koreksi itu saya tarik.
5. **Menggelembungkan severity.** Hampir menulis divergensi `read_range` sebagai
   bug aktif (padanya laten) dan `disk_footprint` sebagai divergensi
   antar-implementasi (padanya keduanya cocok).
6. **Menghitung alarm palsu sebagai temuan.** Baris jejak audit di §9 PRD-3
   *sengaja* mengutip klaim lama; skrip saya menghitungnya sebagai "sisa".
7. **Memperbaiki hal yang tidak rusak.** `check-freeze.sh` saya klaim memakai
   sudo — 0 kemunculan. Sudah dikoreksi di FASE C sebelum Anda menjalankannya.
8. **Mengedit dengan anchor dari ingatan, bukan dari byte aktual.** Tiga kali
   beruntun gagal: anchor `cycle = None\nstack = []` (urutan sebenarnya
   terbalik), `ESC = chr(27)` (berkas berisi teks literal `\033`, bukan byte
   ESC), dan placeholder yang mendarat di dalam heredoc Python sehingga
   merusak badan skrip. Setiap kegagalan terdeteksi karena saya memvalidasi
   setelah mengedit, bukan sebelum. Pelajarannya: baca `repr()` baris target
   dulu, dan jangan pernah menyisipkan placeholder ke dalam heredoc.
