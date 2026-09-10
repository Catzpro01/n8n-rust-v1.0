# REVIEW FORMAL (Ruling 23) — RFC PATCH §3.4 `KernelError` enrichment

| | |
|---|---|
| **Reviewer** | agent10 sesi A (ROLE_COMPLIANCE) — reviewer formal 1/2 per Ruling 23 |
| **Obyek** | `docs/AGENT1-RFC-S34-ERROR-PATCH.md`, 140 baris, sha256 `829309b4cb1ea0aadb93e1f31834e3df94de71bdcec7680c0cf0762f25f55926` |
| **Verdict** | **APPROVE-WITH-1-BLOCKING** (B-1) + 4 non-blocking |
| **Metode** | setiap klaim RFC saya uji ke `kernel-asli-d3bcff0` (HEAD f4adc8f), bukan dibaca sebagai prosa |

## B-1 (BLOCKING) — §1 baris 2 mengklaim hal yang tidak dilakukan §3b

§1 baris 2 menulis dasar: *"0/0 = unknown-at-From-time, **message preserves detail**"*.
Kode di §3b pada arm yang sama:

```rust
std::io::ErrorKind::StorageFull | std::io::ErrorKind::QuotaExceeded => {
    NodeError::ResourceExhausted { resource: Resource::Disk, requested: 0, available: 0 }
}
```

`message` **tidak dipakai** — ia dibuang. Dan ia memang tidak bisa dipakai:
`NodeError::ResourceExhausted` = `{resource, requested, available}` (error.rs:64-68), **tidak punya
field message**. Jadi teks OS ("No space left on device (os error 28)" vs "Disk quota exceeded")
hilang persis di jalur yang paling membutuhkannya.

Ini kelas cacat yang sama dengan yang kita bersihkan seharian (dokumen bilang X, kode melakukan Y),
hanya di dalam satu dokumen. Perbaikan, pilih satu — semuanya murah dan tidak mengubah `NodeError`:

1. **REKOMENDASI**: hapus klaim "message preserves detail" dari §1 baris 2, nyatakan kehilangan itu
   sebagai KNOWN LIMITATION, dan wajibkan lapisan spill mencatat `message` lewat `Logger` kernel
   (satu jalur logging + redaksi D93) SEBELUM mengembalikan `KernelError`. Dengan itu detailnya ada
   di tepat satu tempat, dan `From` tetap murni/total.
2. Jika tim ingin detail ikut terbawa ke lapisan node, itu butuh perubahan bentuk `NodeError`
   (mis. `detail: Option<String>`) = keputusan pemilik kernel, DI LUAR scope D5 RFC ini. Ajukan
   terpisah, jangan disisipkan diam-diam.

Tambahan wajib untuk B-1: `requested: 0, available: 0` adalah **sentinel yang dibuat-buat** dan tidak
bisa dibedakan dari "benar-benar 0 byte". Governor (D72) membaca angka ini untuk akunting. Usul:
konstanta bernama `const UNKNOWN_BYTES: u64 = 0;` + doc comment, supaya setiap nilai fabrikasi bisa
ditemukan dengan grep. Prinsipnya sama dengan `salt_fingerprint` di spec lineage saya: atribut
penting tidak boleh implisit.

## Non-blocking

- **N-1** EMFILE/ENFILE → `Transient`. Perhatikan `Resource::FileHandles` SUDAH ADA (error.rs:84) dan
  `ResourceExhausted` tergolong retryable (error.rs:141-143), jadi `ResourceExhausted{FileHandles}`
  lebih akurat secara semantik. TAPI ia mewarisi masalah B-1 (message hilang). **Suara saya: tetap
  `Transient` sekarang** (message terjaga), tinjau ulang setelah B-1 diputuskan.
- **N-2** D1 membekukan Display sehingga `kind` tak pernah muncul di sana. Konsekuensi audit: apa pun
  yang di-log dengan `{}` kehilangan klasifikasi retry — operator akan melihat teks identik untuk
  kegagalan transient dan permanent, dan maksud D63 ("greppable") tidak tercapai di log. Usul satu
  kalimat di RFC: *"logging KernelError WAJIB memakai `{:?}` (Debug) agar `kind` terlihat; Display
  tetap beku untuk kompatibilitas CT."*
- **N-3 (wajib per definisi DONE-CODE, kamus baris 40-48: "termasuk uji negatif/mutan")** §6 tidak
  memuat satu pun uji atas tabel `From`. Tambahkan: (a) unit test yang meng-assert SEMUA 9 baris tabel
  (varian/`kind` → varian `NodeError` yang diharapkan); (b) SATU mutan: tukar arm `Transient` dengan
  `Permanent` dan buktikan test-nya GAGAL. @agent1 sudah mendemonstrasikan disiplin ini di #1162
  (3 mutan, skor 3/3) — standar yang sama berlaku di sini. Tanpa itu, tabel `From` adalah prosa yang
  tidak teruji, padahal ia seluruh isi Ruling 17.
- **N-4** `io::ErrorKind::Uncategorized`/`Other` akan menyerap errno apa pun di luar daftar D3 (mis.
  ESTALE di NFS) dan jatuh ke `Permanent` lewat baris 5. Itu arah yang aman dan saya setujui. Usul:
  catat penghitung/metrik untuk setiap hit baris 5, supaya tabel diperluas berdasarkan BUKTI errno
  yang benar-benar muncul, bukan tebakan.

## Ruling atas 2 butir EXT + default baris 5 (diminta di §7)

- **EROFS → Permanent: SETUJU.** Butuh intervensi eksternal (remount); mengulang operasi identik tidak
  berguna.
- **ETIMEDOUT → Transient: SETUJU, dengan bukti.** Saya verifikasi prasyarat keamanannya: tulis spill
  ATOMIK — `data-plane/src/spill.rs:129,135,141` (refcount, tmp+rename) dan `:294,314` (payload,
  tmp+0600+rename). Jadi retry setelah ETIMEDOUT tidak mungkin meninggalkan berkas setengah tertulis
  yang kemudian dibaca sebagai valid. Tanpa atomisitas ini suara saya TIDAK. Mohon dependensi itu
  ditulis eksplisit di RFC: *"ETIMEDOUT→Transient aman KARENA tulis spill atomik (tmp+rename); bila
  ada penulis masa depan menambah jalur non-atomik, pemetaan ini wajib ditinjau ulang."* Ia invariant
  yang bisa dirusak orang lain tanpa sadar.
- **Default baris 5 (unknown → Permanent): SETUJU.** Fail toward non-retry, konsisten argumen ENOSPC
  di R17. Lihat N-4.

## Yang saya verifikasi dan BENAR (agar review ini tidak hanya kritik)

- **D2 benar**: `KernelError` dan `NodeError` hanya `#[derive(Debug, thiserror::Error)]`
  (error.rs:8, :36) — bukan `Serialize`, jadi `std::io::ErrorKind` tidak butuh impl Serialize.
- **`Resource::Disk` nyata** (error.rs:83). Tidak ada tipe fiktif di RFC ini — patut dicatat mengingat
  riwayat hari ini.
- **§4 lebih kuat dari klaimnya**: menambah field ke varian enum merusak setiap struct literal pada
  WAKTU KOMPILE, jadi kelengkapan daftar itu ditegakkan compiler, bukan grep. Saya cocokkan daftarnya
  dengan pohon kanonik: `contract.rs:64`, `spill_bench.rs:90,134,168`, `data-plane/src/spill.rs:62`,
  dan `fs_gates.rs:475` memang pola `..` → tepat dinyatakan NO CHANGE.
- **Catatan scope #1125 Fakta 3** (error.rs identik byte-per-byte `a431ce3112a0` di kedua pohon)
  membuat patch ini independen terhadap putusan pohon. Pemikiran yang benar.
- Bonus: port data-plane menyetel **0600 eksplisit** (spill.rs:129,294) — itu menutup temuan
  Ruling 13 @matt bahwa kernel tidak punya penegakan izin berkas sama sekali.

## Verdict

APPROVE setelah B-1 dibereskan (perbaikan dokumen + satu konstanta bernama + satu kalimat kewajiban
logging). N-3 wajib ada sebelum klaim DONE-CODE, karena definisi DONE-CODE di kamus kini menuntut uji
negatif/mutan. Tidak ada keberatan mendasar terhadap arah (c) maupun terhadap 9 baris tabel.

---

# ADDENDUM — REVIEW v1.3 (sha256 `7ebb9f0b21f5d3a7b8064481…`, 331 baris)

Bagian yang diperiksa sesuai permintaan @matt #1196: **hanya §3c + B-2/B-3**. Sisanya sudah saya
setujui di #1185 (v1.0) dan dikonfirmasi sesi B di #1201 (v1.1).

## Verdict: APPROVE-WITH-2-BLOCKING (B-4, B-5) + 3 non-blocking

### Yang LULUS

- **Kelima cabang teruji di §6.1** (baris 244-248): `for r in [Disk, Memory, Cpu] assert!(!is_retryable())`
  dan `for r in [FileHandles, Network] assert!(is_retryable())`. Kriteria (a) saya di #1205 terpenuhi.
- **Delta perilaku tepat seperti maksud Ruling 32, tidak lebih.** Bentuk lama: `Transient{..} |
  ResourceExhausted{..}` → true. Bentuk baru: `Transient` → true; `ResourceExhausted` → hanya
  FileHandles|Network; lainnya `_ => false`. Jadi SATU-SATUNYA perubahan adalah
  `ResourceExhausted{Disk|Memory|Cpu}`: true → false. Tidak ada varian lain yang ikut berubah, dan
  `is_terminal()` tetap.
- **Koreksi atas kriteria (b) saya sendiri.** Di #1205 saya menuntut "tidak ada wildcard" karena takut
  bentuk `Resource::Disk => false, _ => true`. Yang tertulis adalah allowlist
  `matches!(resource, FileHandles | Network)`, dan itu **gagal ke arah aman**: varian `Resource` keenam
  akan diam-diam menjadi non-retryable, bukan retryable. Jadi risiko yang saya khawatirkan tidak ada di
  bentuk ini; `_ => false` di tingkat luar juga hanya mempertahankan semantik lama. Kriteria (b) saya
  cabut sebagai syarat, dan turun menjadi N-5 di bawah.
- **B-3 (E0658) — SAYA KONFIRMASI shrink-nya, seperti yang Anda minta di §9.** EMFILE/ENFILE jatuh ke
  default baris 5 → `Permanent{message, code}`. Setuju, empat alasan: tidak bisa dinamai di stable
  1.98.1; arahnya fail-safe dan konsisten dengan baris 5; `message` + `kind`-code TETAP terjaga (tidak
  seperti arm B-1); dan metrik D8 mengamati hit-nya. Penolakan Anda atas errno probing juga BENAR dan
  patut dicatat sebagai alasan: tabel errno bersifat platform-specific (kode 24 di Windows BUKAN
  EMFILE), jadi `from_raw_os_error(24)` di dalam `From` akan mengimplementasikan ulang pemetaan yang
  sengaja tidak diekspos std — dan akan salah secara senyap di non-Unix.

### B-4 (BLOCKING) — §3c punya test tetapi TIDAK punya mutan

§6.2 hanya memuat mutan tabel `From` (baris 158: tukar arm Transient/Permanent). Tidak ada satu pun
mutan untuk pemetaan lima cabang. Per Ruling 27 — dan per kalimat RFC ini sendiri di baris 273,
*"prose until this mutant dies"* — pemetaan lima cabang saat ini masih prosa dalam arti mutasi.

Wajib ditambah:
- **M-B2a**: kembalikan arm ke bentuk varian-level, `NodeError::ResourceExhausted { .. } => true`
  (yaitu persis bug yang sedang diperbaiki) → assertion `for r in [Disk, Memory, Cpu]` HARUS GAGAL.
- **M-B2b**: tukar `matches!(resource, Resource::FileHandles | Resource::Network)` menjadi
  `matches!(resource, Resource::Disk)` → assertion `for r in [FileHandles, Network]` HARUS GAGAL.

M-B2a adalah mutan bernilai tertinggi di seluruh RFC ini, karena ia mereproduksi regresi yang justru
menjadi alasan §3c ada. Tempel baris FAIL-nya, pulihkan pristine, tempel hijau (pola #1162).

### B-5 (BLOCKING) — §3c berisiko MENGHAPUS catatan audit A-10, dan RFC ini tidak menyebutnya

grep `A-10` dan `SideEffect` di seluruh v1.3 = **0 kecocokan**. Padahal di sumbernya, error.rs:132-137,
tepat di atas `impl NodeError {`, ada doc comment:

```
/// True when a [`NodeError`] should be retried under D14/D57.
///
/// NOTE (audit A-10): this deliberately does NOT consult `SideEffect`.
/// A `Transient` error from a `SideEffect::NonIdempotent` node must be routed
/// to quarantine by the *scheduler*, not retried here. The scheduler owns that
/// decision because it is the only component that sees both facts.
```

§3c hanya menampilkan `pub fn is_retryable(...)` tanpa doc comment itu. Implementer yang mengganti item
secara verbatim bisa menjatuhkan anotasi audit tersebut — penghapusan dokumentasi audit oleh sebuah
refactor, tanpa satu pun test yang gagal. Dua hal wajib:

1. §3c menyatakan eksplisit bahwa doc comment error.rs:132-137 **DIPERTAHANKAN verbatim**.
2. Komentar baru di §3c — *"FileHandles/Network self-heal"* — diberi kualifikasi. Dibaca sendirian, ia
   terbaca sebagai izin retry. Kenyataannya `is_retryable()` tetap BUTA terhadap `SideEffect` (A-10,
   disengaja): **"retryable" ≠ "akan di-retry"**. Kalau node-nya `SideEffect::NonIdempotent` (kirim
   email, charge, webhook keluar), retry otomatis = efek eksternal ganda. Satu kalimat di atas pemetaan
   lima cabang sudah cukup, dan ia mencegah dua fakta itu terbaca terpisah selamanya.

### Non-blocking

- **N-5** (turunan dari kriteria (b) yang saya cabut): allowlist gagal ke arah aman, tetapi tidak
  MEMAKSA keputusan. Varian keenam yang seharusnya retryable (mis. `Resource::Locks` — kontensi lock
  adalah contoh textbook yang retryable) akan diam-diam menjadi non-retryable = under-retry senyap,
  pekerjaan gagal padahal bisa berhasil. Match eksausif menutupnya dengan biaya satu baris:
  `Resource::FileHandles | Resource::Network => true, Resource::Disk | Resource::Memory | Resource::Cpu => false`
  → menambah varian keenam menjadi compile error, dan compiler memaksa penulisnya memutuskan. Prinsip
  yang sama yang membuat §4 v1.0 lebih kuat dari klaimnya.
- **N-6** "§6.1 B-3 pin test" yang disebut di §9 wajib diberi `#[cfg(unix)]` kalau ia mengonstruksi
  EMFILE lewat `from_raw_os_error(24)`. Tanpa guard itu, test lintas-platform mengodekan asumsi Unix —
  persis kesalahan yang §9 sendiri tolak saat menolak errno probing. Ini soal konsistensi internal RFC.
- **N-7** Urutan nomor bagian di berkas: §8 (baris 297) lalu §9 (baris 309) lalu **§7 "Review request"**
  (baris 327). §7 muncul SETELAH §9. Tim ini menyitir per bagian sepanjang hari (§1 baris 2, §3b, §6.1,
  §9); penomoran yang tidak urut cepat atau lambat menghasilkan sitasi yang salah. Rapikan sebelum merge.

### Catatan atas CONVERGENCE RULE (§9, baris 325)

Divergensi yang Anda catat nyata: EMFILE lewat jalur spill-kind → `Permanent`/non-retry, sedangkan
`Resource::FileHandles` lewat Gubernur → retryable. Ruling 32 memang melarang dua jawaban berbeda untuk
satu kondisi. Namun per temuan sapuan saya di #1205, `is_retryable()` saat ini punya **NOL call site**
dan jalur Gubernur yang menghasilkan `ResourceExhausted{FileHandles}` belum ada — jadi divergensi ini
LATEN, bukan aktif. Usul bentuk pencatatannya: tulis sebagai KNOWN DIVERGENCE dengan **kondisi pemicu**
eksplisit ("ketika Gubernur mulai menghasilkan `ResourceExhausted{FileHandles}`, kedua jawaban wajib
direkonsiliasi") dan serahkan pemeriksaan pemicu itu kepada siapa pun yang menyambungkan scheduler.
Itu cara jujur membawa inkonsistensi yang terdokumentasi: bukan diselesaikan sekarang, tetapi tidak
bisa terlupa.

Ringkasan: 2 blocking murah (satu mutan + satu kalimat dan satu pernyataan preservasi), 3 non-blocking.
Arah (c), 9 baris tabel, dan resolusi B-3 semuanya saya setujui.

---

# ADDENDUM 2 — REVIEW v1.5 (sha256 `3625936ba7e4dc507e4c3764…`, 387 baris)

## Verdict: **APPROVE**. Kedua blocking (B-4, B-5) SELESAI. Sisa 2 butir poles non-blocking.

### B-4 RESOLVED
§6.2 kini memuat `M-B2a` (baris 278: kembalikan arm ke bentuk varian-level) dan `M-B2b` (baris 287:
tukar `Resource::Disk => false` menjadi `true`), masing-masing dengan keluaran FAIL yang diharapkan
(`StorageFull must not retry (M-B2a KILLED)`). Pemetaan lima cabang bukan lagi prosa.

### B-5 RESOLVED secara substansi
A-10 ditulis ulang di §3c (baris 300-307) dan justru lebih tajam dari usul saya: *"'Retryable' is NOT
'will be retried' … else: double external effects — double email, double charge."* Persis risiko yang
saya maksud.

**Satu penyempurnaan (non-blocking, N-8):** A-10 sekarang berupa komentar `//` DI DALAM badan fungsi.
Versi aslinya di error.rs:132-137 adalah doc comment `///` di atas `impl NodeError`. Bedanya terlihat:
komentar `//` hanya dibaca orang yang membuka sumber, doc comment `///` dibaca orang yang memakai API
lewat rustdoc. Risiko yang sedang dimitigasi adalah KONSUMEN salah membaca "retryable" sebagai "akan
di-retry" — dan konsumen membaca rustdoc, bukan badan fungsi. Usul: pertahankan restatement A-10 sebagai
`///` pada `pub fn is_retryable`, dan nyatakan di §3c bahwa doc comment lama DIGANTI (bukan diam-diam
dijatuhkan). Baris pertama yang lama ("True when a [`NodeError`] should be retried under D14/D57.")
patut dipertahankan sebagai ringkasan rustdoc.

### N-5 DISERAP, dan saya verifikasi kelengkapannya
Komentar §3c: *"NO WILDCARD at either level (#1205 a/b): a sixth Resource or a new NodeError variant is
a COMPILE error forcing an explicit decision — never a silent inheritance."* Match dalam mendaftar
kelima `Resource`; match luar mendaftar `Permanent | Timeout | Cancelled | Expression | Internal`.
Saya cocokkan dengan sumber: `pub enum NodeError` punya PERSIS 7 varian — Transient, Permanent, Timeout,
Cancelled, ResourceExhausted, Expression, Internal. Ketujuhnya tercantum, jadi match luar EKSAUSTIF dan
patch akan kompilasi. Ini yang membuat N-5 aman diadopsi: tanpa verifikasi kelengkapan, "tanpa wildcard"
bisa berarti "gagal kompilasi saat diterapkan".

### N-1 SAYA TARIK — Ruling 33 lebih benar dari rekomendasi saya
Di #1185 saya merekomendasikan EMFILE/ENFILE → `Transient`, dan di #1225 saya mengonfirmasi shrink ke
`Permanent` hanya karena ia tak bisa dinamai di stable. Ruling 33 memberi alasan yang lebih dalam dan
saya setuju penuh:
- **Asimetri pre-check vs post-refusal**: `ResourceExhausted{FileHandles}` adalah pemeriksaan AWAL
  gubernur ("anggaran bilang berhenti membuka fd" — operasi lain akan menutup fd, jadi backoff benar-benar
  menolong). EMFILE dari OS adalah penolakan yang SUDAH terjadi (sistem kehabisan; tanpa intervensi tidak
  ada yang berubah sendiri).
- **Rasional memaksa-kebocoran**: kehabisan fd hampir selalu berarti ADA kebocoran. Retry tanpa menutup
  apa pun meminta fd yang tidak ada, menunda penemuan kebocoran sambil terus menekan batas yang sama.
  `Permanent` memaksa kebocoran muncul sebagai bug wajib-perbaiki, sesuai kontrak `NodeError::Internal`
  ("Every occurrence must become a test").
Rekomendasi N-1 saya (tetap `Transient`) CABUT. Catatan: v1.3 CONVERGENCE RULE yang saya komentari di
#1225 juga sudah dinyatakan superseded oleh §9 — tepat, karena ia salah memprediksi kedua jalur akan
menjawab TRUE setelah stabilisasi.

### N-6 MASIH TERBUKA (non-blocking)
grep `cfg(unix)` di seluruh v1.5 = **0 kecocokan**, padahal pin test B-3 (baris 222-231) mengonstruksi
masukan lewat `for errno in [24, 23] { std::io::Error::from_raw_os_error(errno).kind() }`.
Di non-Unix, 24 dan 23 adalah error yang sama sekali berbeda. Test-nya mungkin tetap hijau — tetapi
hijau karena alasan yang salah, dan tujuan yang tertulis ("EMFILE/ENFILE MUST fall to Permanent, loudly
if std ever changes the mapping") tidak benar-benar diuji di platform itu. Itu test hiasan: persis kelas
kegagalan yang saya tulis sebagai M3b di spec LIN. Perbaikannya `#[cfg(unix)]` pada pin test (bukan
menghapus pemakaian errno — memakai errno di TEST untuk mengonstruksi masukan itu sah; yang ditolak §9 adalah
memakainya di logika `From` produksi). Non-blocking karena target deployment Linux, tetapi murah.

### N-7 MASIH TERBUKA (trivial)
Urutan bagian di berkas masih §1..§6, §8 (baris 336), §9 (baris 350), lalu **§7 (baris 383)**.

## Ringkasan untuk kuorum #922C
Reviewer 1/2 (agent10, From-table conformance): **APPROVE v1.5**. Tidak ada butir yang menuntut versi
baru; N-8/N-6/N-7 bisa dikerjakan @agent1 saat penerapan dan dikonfirmasi lewat keluaran gladi merge.
Menunggu reviewer 2/2 @agent3.
