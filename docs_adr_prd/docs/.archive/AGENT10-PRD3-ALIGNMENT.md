# AGENT10 — PERNYATAAN KESELARASAN & VERIFIKASI INDEPENDEN: PRD-3 CANONICAL (07:53)

**Penulis:** agent10 (ROLE_COMPLIANCE, sesi A) · **Tanggal:** 2026-09-09 07:5x UTC
**Objek:** `PRD-3-CANONICAL-APPROVED.md` (fern, 07:53, 29.249 B, sha256 `1b1949cf…`) — identik byte-per-byte dengan `PRD-3-PERFECTION-CHECKLIST.md` (sha sama)
**Isi:** pengesahan pemilik (20 baris wrapper) + `PRD-3-PERFECTION-CHECKLIST-v3.2-matt.md` apa adanya (sha `e51bcc99…`)
**Metode:** verifikasi independen dengan eksekusi, bukan pembacaan. Mengikuti checklist agent3 #533 / agent5 §9.1 (alat benar? bukti lengkap? konteks temporal? penjelasan alternatif?).

---

## 1. STATUS KESELARASAN

**ALIGNED untuk isi teknis v3.2 — dengan 6 cacat kepatuhan (C-1…C-6) yang harus diperbaiki sebelum dokumen ini berfungsi sebagai rekaman audit.**

Cacat itu **tidak** membatalkan keputusan pemilik. Yang saya katakan lebih sempit dan lebih praktis: **berkas kanonik saat ini menyatakan dua hal yang berlawanan tentang statusnya sendiri**, satu hash di dalam keputusan yang disahkan rusak secara byte, versi sebelumnya hilang tertimpa, keputusan K-4 belum dijawab padahal kode dibuka, dan Wave 0 menunjuk dua pohon kode yang berbeda. Semuanya murah diperbaiki sekarang, mahal bila dibiarkan sampai kode ditulis.

---

## 2. ATESTASI POSITIF — klaim §7.1 matt TEREPRODUKSI (ini mendukung keputusan K-1 pemilik)

Keputusan K-1 pemilik bersandar pada "kernel-asli-d3bcff0 (commit a7d0357, 19/19 test hijau)". Saya jalankan ulang sendiri. Lokasi: `/opt/agent-workspace/kernel-asli-d3bcff0`.

| Klaim §7.1 / pengesahan | Terukur oleh saya | Verdict |
|---|---|---|
| `cargo metadata --no-deps` exit 0 (sebelumnya 101) | `exit=0`, stderr kosong | **COCOK** |
| `cargo test` 19 passed; 0 failed | `19 passed; 0 failed; 0 ignored` (1,73 s) | **COCOK** |
| `cargo clippy --all-targets` 0 warning 0 error | bersih, `Finished dev profile in 9.69s` | **COCOK** |
| `members` disisakan `crates/kernel` | `members = [ "crates/kernel", ]` | **COCOK** |
| `#![forbid(unsafe_code)]` di `lib.rs:47` | `47:#![forbid(unsafe_code)]` (lib.rs 104 baris) | **COCOK** |
| `crates/data-plane` punya manifest tapi 0 file `.rs` | `Cargo.toml` ada, **0** `.rs` | **COCOK** |
| commit `d3bcff0` dan `a7d0357` ada; HEAD = `a7d0357` | keduanya ada; `HEAD a7d0357`; **0 file berubah** (pohon bersih) | **COCOK** |
| commit `db20e72` (lokal matt) | **tidak ada** di repo VPS | **KONSISTEN** dengan label "(lokal)" |
| §7.2 `testkit/src/lib.rs:567` = `DefaultHasher`, tanpa dep `blake3` | baris 567/570/572 tepat; `grep -c blake3 Cargo.toml` = **0** | **COCOK** |

**Kesimpulan §2:** dasar teknis keputusan K-1 **sah dan dapat direproduksi**. Ini penting justru karena matt sendiri menemukan klaim palsu di §6.1 v3.0: klaim teknis di v3.2 lolos verifikasi pihak ketiga.

### 2.1 Koreksi atas kecurigaan saya sendiri (wajib dicatat)
Pada pemeriksaan pertama saya membandingkan klaim §7.1 dengan `/opt/agent-workspace/rust-engine` dan mendapati semuanya tampak salah (15 member, `testkit`/`nodes-core` ada, `data-plane` punya 4 `.rs`, tidak ada `.git`). **Saya hampir melaporkan itu sebagai cacat. Itu salah saya.** Pohon itu adalah **STUB** — `crates/kernel/src/lib.rs` menulis sendiri: *"Status: STUB — kode asli Fase 0 (2.644 baris) belum diunggah ke server."* Pohon yang matt ukur adalah `kernel-asli-d3bcff0`. Setelah dibandingkan dengan pohon yang benar, semua klaim cocok.
Pelajaran yang sama untuk keempat kalinya sesi ini: **verifikasi objek yang tepat sebelum menyimpulkan.** Catatan tambahan untuk tim: keberadaan dua pohon bernama mirip (`rust-engine` = stub, `kernel-asli-d3bcff0` = kanonik) adalah jebakan yang akan menelan orang lain juga. Usul: stub itu diberi nama `rust-engine-STUB-jangan-dipakai/` atau dihapus setelah K-1 dieksekusi.

### 2.2 Tiga selisih sitasi di §1 (kecil, tapi dokumen ini menuju tanda tangan)
| Sitasi §1.2 | Diklaim | Terukur | mtime berkas | Penilaian |
|---|---|---|---|---|
| `PRD-1-N8N-ANALYSIS.md` | 1.016 baris | 1.016 | — | cocok |
| `PRD-2-RUST.md` | 1.290 baris | 1.290 | — | cocok |
| `ANALISIS-KORPUS-NODE.md` | **v1.2**, 572 baris | header berkas: **Versi 1.0** (ada catatan "Koreksi v1.1" di dalam), 572 baris | 05:55 | **label versi tidak cocok** |
| `SINTESIS-SPILLSTORE.md` | 300 baris | 299 | 06:13 | selisih 1 (kemungkinan newline akhir) |
| `AGENT5_QA_SECURITY_SPEC.md` | 669 baris | **726** | **06:54** | **basi saat ditulis** (v3.2 ditulis 07:23) — 57 baris §9.1 ditambahkan agent5 #551 sebelum v3.2 |
| `AGENT10-COMPLIANCE-CRYPTO-AUDIT.md` v1.2 | 63.406 byte | 63.406 byte, 936 baris | 06:56 | cocok |
| `AGENT10-PRD3-COMPLIANCE-REVIEW.md` | 257 baris, sha `6f9f8354…` | 257 baris, sha identik | 07:08 | cocok |

Bukan tuduhan — penjelasan alternatifnya wajar (berkas bergerak selama sintesis). Tapi untuk dokumen sign-off, sitasi sebaiknya menyebut **sha256**, bukan jumlah baris: baris berubah diam-diam, hash tidak.

---

## 3. ENAM CACAT KEPATUHAN (C-1…C-4 pada dokumen pengesahan; C-5…C-6 pada sasaran Wave 0)

### C-1 (HIGH) — Berkas kanonik menyatakan dua status yang berlawanan
Wrapper pengesahan hanya **menambah 20 baris di depan**; tidak satu pun pernyataan di dalam v3.2 diperbarui. Hasilnya satu berkas berisi:

| Baris | Pernyataan |
|---|---|
| 4 | `**Status:** ✅ APPROVED & SIGNED-OFF — ZERO-CODE MANDATE RESMI DIBUKA` |
| 26 | `**Status:** ⚠️ DRAFT — BELUM DISETUJUI, dan belum boleh ditandatangani` |
| 27 | `**Aturan:** Zero-Code Mandate aktif.` |
| 47, 51, 113 | INOVASI menunggu konfirmasi pemilik; *"tugas `[INOVASI]` tidak dibuka"* |
| 543 | `Pemilik Proyek \| **BELUM — dan ini yang menentukan**` |

Wrapper butir 1 mengesahkan K-8 ("seluruh tugas [INOVASI] dibuka"), sementara baris 113 di berkas yang sama mengatakan sebaliknya.
**Kenapa ini serius:** ini kelas cacat yang sama dengan §6.1 v3.0 yang matt temukan sendiri — hanya arahnya dibalik. Di sana: catatan persetujuan tanpa aksi. Di sini: aksi persetujuan yang sah, disertai catatan yang masih menyangkalnya. Auditor yang membuka berkas kanonik tidak bisa menentukan apa yang disahkan. Dalam sengketa, dokumen yang berkontradiksi internal melemahkan **seluruh** rangkaian bukti, bukan hanya bagian yang salah.
**Perbaikan (murah):** sunting baris 26/27/51/113/543 agar mencerminkan pengesahan, atau tambahkan satu baris di tiap lokasi: *"[DIPERBARUI 07:53: disahkan — lihat wrapper]"*. Jangan menghapus teks asli matt; cukup tandai.

### C-2 (MED) — Hash commit di keputusan K-1 mengandung byte kontrol
Baris 14 berkas kanonik:
```
- Basis kanonik engine adalah kernel-asli-d3bcff0 (commit <BEL>7d0357 yang 19/19 test hijau) ...
```
Terukur: tepat **1 byte `0x07` (BEL)** di seluruh berkas; hexdump menunjukkan `07 e2 80 94` pada posisi itu. Huruf `a` pada `a7d0357` terganti karakter kontrol — kemungkinan artefak `sed`/editor (saya pernah terkena kelas bug yang sama: karakter CJK terselip ke berkas saya sendiri).
**Akibat:** pengenal yang disahkan tidak bisa disalin/dipakai apa adanya; alat yang membandingkan string akan gagal. Untuk keputusan yang menunjuk **basis kanonik kode**, hash yang rusak adalah cacat substansial, bukan kosmetik.
**Perbaikan:** ganti dengan `` `a7d0357` `` (dan idealnya hash penuh `git rev-parse a7d0357`). Saya sarankan menambahkan pemeriksaan byte kontrol ke `check-freeze.sh`: `LC_ALL=C tr -d '\11\12\15\40-\176' < berkas | wc -c` harus 0 untuk dokumen terkendali.

### C-3 (HIGH) — Dokumen terkendali ditimpa di tempat, versi lama hilang
`PRD-3-PERFECTION-CHECKLIST.md` v3.0.0-FINAL (fern, 07:01, 8.356 B, sha `64b06ee8…`) **ditimpa** oleh berkas 29.249 B dengan **nama yang sama**, tanpa `.bak`.
Saya mencari pemulihan: `find / -name "PRD-3*"` di luar `docs/` → tidak ada; `/home/fern` → kosong; `/tmp` → hanya berkas agent5/agent6. **v3.0 tidak dapat dipulihkan dari disk.** Ia kini hanya bertahan sebagai kutipan di dalam v3.1/v3.2 dan di review saya.
**Kenapa ini masalah kepatuhan, bukan kerapian:** seluruh nilai audit trail terletak pada kemampuan menunjukkan *keadaan sebelumnya*. §9 v3.2 sendiri berjudul "Selisih terhadap draf v3.0 (jejak audit)" — jejak itu sekarang merujuk ke berkas yang sudah tidak ada. Dan agent5 Pasal 7.3 menetapkan "jangan pernah menimpa atau menghapus file milik agent lain; buat file baru dengan nama berbeda".
**Perbaikan:** (a) ke depan, nama kanonik tidak pernah ditimpa — versi baru = berkas baru + pembaruan penunjuk; (b) sekarang, rekonstruksi v3.0 dari kutipan dan simpan sebagai `PRD-3-v3.0-fern-REKONSTRUKSI-PARSIAL.md` dengan pernyataan jujur bahwa ia tidak lengkap; (c) tambahkan `.bak`/snapshot sebelum penimpaan dokumen terkendali.

### C-4 (HIGH) — "SETUJUI SEMUA" tidak menjawab keputusan yang sudah dienumerasi, dan yang belum dijawab justru yang paling menentukan
Wrapper menulis *"Mandat Mutlak: SETUJUI SEMUA"*, dan menjawab **K-8, K-3/T-14, K-1**. Yang **tidak** dijawab, padahal matt §8 menyebutnya memblokir sign-off:

| Keputusan | Status setelah pengesahan | Kenapa ini yang paling mahal |
|---|---|---|
| **K-4** `ALL ALL=(ALL) NOPASSWD: ALL` | **TIDAK DIJAWAB** | matt §8: *"Membatalkan jaminan keamanan L2c … bahkan user `nobody` dapat root."* §8 juga: *"selama K-4 terbuka, satu-satunya lapisan yang berarti adalah anchor eksternal."* Kode implementasi kini **dibuka** di atas kondisi ini |
| **K-6** rotasi SSH key | **TIDAK DIJAWAB** | kunci privat pernah ditempel plaintext di chat + K-4 = tidak ada batas keamanan |
| **K-7** disk & swapfile | **TIDAK DIJAWAB** | `vda1` 88% (sisa 2,3 G) — terukur lagi hari ini |
| **D-A1…D-A6** (anchor, terbit 07:48 — 5 menit sebelum pengesahan) | **TIDAK DIJAWAB** | D-A5 (siapa pemegang fingerprint CA out-of-band) adalah akar kepercayaan anchor; tanpa itu anchor hanya memindahkan masalah |
| **D-L1…D-L5** (lineage) | **TIDAK DIJAWAB** | D-L1 menentukan lineage masuk cakupan atau dinyatakan sadar di luar |

**Posisi saya, dan alasannya:** saya **tidak** membaca "SETUJUI SEMUA" sebagai jawaban atas pertanyaan yang dienumerasi. Bukan karena meragukan wewenang pemilik — wewenangnya mutlak dan saya tidak menantangnya — tetapi karena persetujuan blanket atas daftar yang tidak disebut **tidak menghasilkan rekaman yang bisa diaudit**. Kalau dibaca sebagai jawaban, maka keputusan K-4/D-A5/D-L1 akan diambil **di dalam kode oleh siapa pun yang mengimplementasikan lebih dulu**, tanpa tercatat. Itu persis pola yang pemilik identifikasi sendiri dan dikutip matt di §10: *"88 keputusan pernah disetujui tanpa review adversarial, oleh pihak yang sama yang mengusulkannya."*
**Usul konkret (10 menit):** pemilik/fern menerbitkan lampiran satu halaman: untuk tiap K-4, K-6, K-7, D-A1…D-A6, D-L1…D-L5 → `DISETUJUI-USULAN` / `DITOLAK` / `DITUNDA`. Default eksplisit jauh lebih murah daripada default diam-diam.

---

### C-5 (HIGH) — Wave 0 menunjuk DUA pohon kode yang berbeda tanpa menyebutnya
K-1 mengesahkan `kernel-asli-d3bcff0` sebagai basis kanonik. Terukur:

| Pohon | crates | `testkit/*.rs` | `data-plane/*.rs` | `.git` |
|---|---|---|---|---|
| `kernel-asli-d3bcff0` (**kanonik per K-1**) | `data-plane`, `kernel` | **0** | **0** | **ada**, HEAD `a7d0357`, bersih |
| `rust-engine` | 15 crate (api, auth, cli, executor, expr, …) | **1** (785 baris, `blake3_hash` di :567) | **4** | **tidak ada** |

Akibatnya tabel Wave 0 §4 menunjuk sasaran yang tidak hidup di pohon yang sama:
- `W0-HASH-FIX` (perbaiki `testkit/src/lib.rs:567`) → **testkit tidak ada di pohon kanonik**; ia hanya ada di `rust-engine`, yang `kernel/src/lib.rs`-nya menulis sendiri *"Status: STUB — kode asli Fase 0 (2.644 baris) belum diunggah ke server."*
- `W0-CHECKSUM-AGREE` (samakan testkit ↔ data-plane) → sama; kedua crate itu tidak ada di kanonik.
- `W0-Spill-IMPL` (pindahkan `FileSpillStore` dari `examples/` ke `crates/data-plane/src/`) → **ini cocok dengan kanonik**: `kernel-asli-d3bcff0/crates/kernel/examples/spill_bench.rs` ada, `data-plane` kosong.

Dokumen baru `AGENT2-AGENT3-W0-CHECKSUM-AGREE.md` (07:53, sha `b748e8cf…`) menulis *"Existing implementation di data-plane (agent2)"* untuk SHA-256 — implementasi itu **hanya ada di pohon stub** (4 `.rs`), bukan di kanonik (0 `.rs`). Jadi kalimat "existing" benar untuk satu pohon dan salah untuk pohon yang baru disahkan sebagai kanonik.
**Perbaikan:** setiap tugas Wave 0 wajib menyebut **path pohon** eksplisit. Dan putuskan: apakah `testkit` + 13 crate lain **masuk** ke pohon kanonik (K-1 menyebut "penyerapan FileSpillStore dari data-plane", bukan penyerapan 15 crate), atau `W0-HASH-FIX`/`W0-CHECKSUM-AGREE` dikerjakan di `rust-engine` yang akan dibuang. Tanpa ini, agent3 bisa "memperbaiki" testkit di pohon yang tidak kanonik dan gate tetap merah di pohon yang kanonik — atau sebaliknya.
**Usul tambahan:** ganti nama `rust-engine` → `rust-engine-STUB-jangan-dipakai/` supaya tidak ada yang membangun di atasnya tanpa sadar (saya sendiri hampir salah menyimpulkan karena ini — §2.1).

### C-6 (MED) — `Merkle tree` muncul kembali di dokumen baru, bertentangan dengan spec yang sudah direkonsiliasi
`AGENT2-AGENT3-W0-CHECKSUM-AGREE.md:45`:
> `**Purpose:** Derive chain_key untuk Merkle tree di Execution Envelope`

Posisi resmi yang berlaku: `AGENT10-EXEC-ENVELOPE-SPEC.md` v1.2 §3 — `merkle_root` dan `leaf_i` **DICABUT** (rekonsiliasi #555 + #628 butir 6); `envelope_root` = **chain head `h_{n-1}`**, dan *"istilah 'Merkle root' di #327 butir 3 adalah salah — C-03"*. PRD-3 kanonik §2.2 juga mencatat "Merkle / hash-chain **0×**".
Ini bukan soal selera istilah: C-03 ada karena **Merkle root tidak memberi urutan**, sedangkan Envelope adalah *rolling* chain yang seluruh nilainya terletak pada urutan. Menyebut "Merkle tree" di kontrak turunan mengundang implementasi yang menghitung akar tak-berurut lalu mengklaim sifat rantai.
Butir lain di §2.3 dokumen yang sama: *"When: Execution start (derive chain_key dari execution_id + secret)"*. Spec saya mendistribusikan `chain_key` lewat **secret-manager** dengan `chain_key_id` per segmen (rotasi = peristiwa pemutusan rantai), dan yang bersifat per-eksekusi adalah `salt_exec` (lineage), bukan `chain_key`. Keduanya mungkin bisa didamaikan, tapi **harus** didamaikan secara tertulis — dokumen itu menyebut dirinya "Contract untuk Envelope derivation (agent10)", sedangkan P3-03 yang sudah diputus matt menempatkan **konstruksi crypto Envelope = agent10**.
**Perbaikan:** @agent2 @agent3 mohon §2.3 merujuk `AGENT10-EXEC-ENVELOPE-SPEC.md` §3/§6.2 alih-alih menyatakan ulang kontraknya, dan hapus "Merkle tree". Bila kalian memang butuh sifat yang Merkle berikan (range-proof parsial), itu use-case baru — sampaikan, dan spec saya §3 sudah menyiapkan jalur masuknya kembali.
**Yang saya ENDORSE dari dokumen itu:** batas SHA-256 = integritas spill vs BLAKE3 = identitas/dedup CASD adalah pemisahan yang tepat dan saya dukung; adopsi angka "1920 bytes transient RAM (agent10 measurement)" juga tepat — hanya perlu tambahan bahwa **2 hasher hidup = 94% budget 4 KB dan 3 = 141% (melanggar)**, jadi aturan "satu hasher per jalur" mengikat (§5 L2b kanonik).

---

## 4. KONFLIK ATURAN TUGAS YANG TIMBUL KARENA PEMBUKAAN ZERO-CODE

1. **Wave yang dibuka = 0 dan 1 saja.** Tugas saya (`W3-EXEC-ENVELOPE`, `W3-ITEM-LINEAGE`) ada di **Wave 3** ⇒ **belum dibuka**, jadi saya tidak mengklaimnya. Ini juga menyelesaikan konflik instruksi yang saya angkat di #717 (fern #637 "ambil begitu storage siap" vs matt "[N] tunggu §2.2"): K-8 sudah mengesahkan INOVASI, jadi sisi matt terpenuhi; yang belum adalah urutan Wave.
2. **ROLE_COMPLIANCE kini punya DUA tugas Wave-3** (W3-EXEC-ENVELOPE + W3-ITEM-LINEAGE, keduanya `[N]`, keduanya agent10), sementara aturan tim = **maks 1 tugas aktif per agen**. Perlu keputusan urutan. Usul saya: **W3-EXEC-ENVELOPE lebih dulu** (lineage bergantung padanya — `lineage_root` masuk digest Envelope, lihat `AGENT10-ITEM-LINEAGE-SPEC.md` §5), dan W3-ITEM-LINEAGE menyusul.
3. **Wave 0 tidak punya tugas kode untuk anchor, padahal C-01 CRITICAL dan kode kini diizinkan.** Usul: tambahkan **`W0-ANCHOR-IMPL`** (P0, ROLE_COMPLIANCE) = implementasi penulis + verifier anchor sesuai `AGENT10-W0-ANCHOR-SPEC.md` §5, gate ANC-1…ANC-7. Alasannya: §8 matt sendiri menyatakan selama K-4 terbuka **satu-satunya lapisan yang berarti adalah anchor eksternal**, dan K-4 belum dijawab. Membuka implementasi Wave 1 tanpa anchor berarti membangun riwayat audit yang tidak bisa dipertahankan.
4. **`task_queue` masih belum punya baris Wave 0** (terukur 07:52: `swarm-task list | grep W0` → kosong) — sudah saya angkat di #717, didukung agent2 #719 dan agent3 #720.
5. **Identitas dua sesi paralel agent10 masih belum diputus**, dan urgensinya **naik**: `swarm-task claim` kini terbuka untuk kode. Dua sesi dengan satu identitas bisa mengklaim tugas yang sama atau menulis berkas yang bertentangan — dan kali ini akibatnya kode, bukan pesan. Sampai fern memutuskan (#628 butir 5: satu operator / lane tertulis / akun agent11), saya **tidak mengklaim** tugas apa pun.

---

## 5. YANG SAYA TANDATANGANI DAN YANG TIDAK

| Item | Posisi agent10 |
|---|---|
| Isi teknis v3.2 (arsitektur, Wave, gate L1–L5, remediasi W0) | **SELARAS** — dan klaim §7.1/§7.2 terverifikasi ulang oleh saya (§2) |
| P3-01 / P3-02 / P3-03 (temuan saya) | **TERATASI** di v3.2: L2 dipecah, `W3-ITEM-LINEAGE` ditambahkan, kepemilikan Envelope dipisah (crypto=agent10, gate=agent5) |
| Keputusan K-1, K-3/T-14, K-8 pemilik | **tidak saya tolak**; K-1 bahkan saya kuatkan dengan reproduksi 19/19 |
| **Berkas kanonik sebagai rekaman audit** | **BELUM** — C-1…C-6 harus diperbaiki dulu (§3) |
| Mengesahkan gate apa pun atas nama sendiri | **TETAP MENOLAK** (recommender ≠ approver, #628; didukung matt §10, agent7 #671, agent1 #672) |
| Klaim `W3-EXEC-ENVELOPE` / `W3-ITEM-LINEAGE` sekarang | **TIDAK** — Wave 3 belum dibuka + identitas belum diputus |

Saya tidak menahan tanda tangan untuk menghalangi pekerjaan. Yang saya minta adalah 4 perbaikan murah pada berkas pengesahan dan satu lampiran keputusan — supaya ketika auditor bertanya "apa yang disetujui pemilik pada 07:53?", jawabannya ada di dalam berkasnya, bukan di riwayat chat.

---

## 6. Reproduksi

```
K=/opt/agent-workspace/kernel-asli-d3bcff0
export CARGO_HOME=/mnt/extra-storage/agent10-cargo-home
export CARGO_TARGET_DIR=/mnt/extra-storage/agent10-cargo-target
cd $K && cargo metadata --no-deps --format-version 1 >/dev/null; echo $?      # 0
cargo test 2>&1 | tail -3                                                     # 19 passed; 0 failed
cargo clippy --all-targets 2>&1 | tail -2                                     # bersih
git -C $K -c safe.directory=$K log --oneline -2                               # a7d0357, d3bcff0
git -C $K -c safe.directory=$K status --short | wc -l                         # 0
grep -n "forbid(unsafe_code)" $K/crates/kernel/src/lib.rs                     # 47:
find $K/crates/data-plane -name '*.rs' | wc -l                                # 0
sed -n '565,576p' /opt/agent-workspace/rust-engine/crates/testkit/src/lib.rs  # DefaultHasher (stub tree)
LC_ALL=C tr -d '\11\12\15\40-\176' < docs/PRD-3-CANONICAL-APPROVED.md | wc -c # > 0  (byte BEL, C-2)
sha256sum docs/PRD-3-CANONICAL-APPROVED.md docs/PRD-3-PERFECTION-CHECKLIST.md # identik
```
