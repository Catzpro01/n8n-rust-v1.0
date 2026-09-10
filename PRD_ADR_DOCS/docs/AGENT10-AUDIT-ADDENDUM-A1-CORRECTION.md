# ADDENDUM A1 — KOREKSI ATAS AUDIT §3.3: keyed chain PERLU tapi TIDAK CUKUP
Penulis: agent10 (ROLE_COMPLIANCE, sesi A). Status: **KOREKSI DIRI SENDIRI**.
Berlaku atas: docs/AGENT10-COMPLIANCE-CRYPTO-AUDIT.md §3.3, §4 butir 2 (bagian A.1), §6 tabel, D-L2.
Alasan dibuat sebagai addendum: dokumen induk v1.2 sedang dimiliki sesi lain; saya tidak menyunting
artefak yang bukan saya terbitkan sesi ini (lihat #628 butir 5). Addendum ini yang dipakai.

## A1.1 Ringkasan koreksi
Klaim yang saya buat dan sekarang saya cabut sebagian:
> "keyed chain + anchor menutup C-01 ... penyerang tanpa chain_key bisa memalsukan? **false**."
Kalimat itu benar, tetapi tidak lengkap dengan cara yang menyesatkan: ia hanya menguji penyerang
TANPA kunci. Sisi lain tidak saya uji saat menulis. Setelah diuji:

> [G-C1b-keyed] **JIKA** penyerang MENDAPAT `chain_key`, pemalsuan **LOLOS** (`true`).

Jadi `keyed chain` bukan jaminan. Ia memindahkan masalah ke **perlindungan kunci**.

## A1.2 Mengapa di deployment ini perlindungan kuncinya tidak ada
1. ERR-012 / K-4: `agent-team ALL=(ALL) NOPASSWD:ALL` untuk 11 akun (agent1..10 + fern) — root penuh di host mana pun.
2. PRD-2 §11.1 (ditulis sendiri oleh tim): kunci yang dibaca dari `env` atau file di host yang sama
   "memberi perlindungan nol terhadap penyerang ber-root".
3. `chain_key` engine berasal dari secret-manager, tapi engine berjalan di host yang sama. Penyerang
   ber-root bisa membaca memori proses, env, atau vault-cache tanpa pernah "membobol" apa pun.

Kesimpulan: di mesin ini, satu-satunya lapisan yang menahan penyerang ber-root adalah lapisan yang
root lokal TIDAK bisa tulis — yaitu **anchor eksternal** (sink WORM / RFC 3161 di luar host).

## A1.3 Urutan lapisan yang benar (menggantikan penekanan di §4 butir 2)
| Lapisan | Menahan apa | Batas jaminannya |
|---|---|---|
| **A.2 Anchor eksternal** (WORM/RFC3161, di luar host) | Penyerang ber-root penuh di host | Root yang sudah dipublikasikan TIDAK bisa ditulis ulang. **Ini lapisan yang MENJAMIN.** |
| A.1 Keyed chain | Penyerang dengan akses tulis DB saja | **Mengurangi** jendela & menaikkan biaya. BUKAN jaminan bila kunci bocor. |
| A.3 Jendela jujur | Klaim berlebihan oleh verifier sendiri | Entri sejak anchor terakhir = `UNANCHORED`; verifier DILARANG menyebutnya terverifikasi. |
A.4 (append-only + root di Envelope) = memperkuat bukti, bukan lapisan deteksi.

## A1.4 Konsekuensi untuk klaim produk (WAJIB dipakai di dokumen apa pun)
- SALAH: "tamper-proof", "riwayat tidak bisa dipalsukan".
- BENAR: "tamper-**evident** terhadap penyerang yang tidak mengendalikan sink anchor, dengan jendela
  tak-terjangkar <= N entri; entri dalam jendela itu berlabel `UNANCHORED` dan tidak diklaim terverifikasi."
Selisih dua kalimat ini adalah selisih antara klaim yang bertahan di audit SOC2 dan klaim yang
dipatahkan auditor di pertanyaan pertama tentang root lokal.

## A1.5 Konsekuensi untuk spec Envelope
1. `chain_key` wajib didistribusikan lewat **secret-manager**, TIDAK pernah lewat env/file (sudah ada di EXEC-ENVELOPE-SPEC §6.2 — dipertahankan, kini dengan alasan yang lebih kuat).
2. Rotasi `chain_key` = peristiwa **pemutusan rantai**: verifier harus tahu `chain_key_id` per segmen
   (sudah di draft #555 §3.2). Rotasi TANPA pencatatan segmen membuat rantai lama tak terverifikasi.
3. Sink anchor harus di luar kontrol root host mana pun. Selama sink-nya ada di vda1/vdb yang sama,
   A.2 TIDAK ADA dan seluruh klaim C-01 kembali terbuka.

## A1.6 Bukti eksekusi (reproducible)
Host: master. Toolchain shared rustc 1.98.1; blake3 1.8.7. `CARGO_HOME`/`CARGO_TARGET_DIR` di vdb.
Workdir: `/mnt/extra-storage/a10-work` (`cargo run`). Kode: chain(u32-BE len||prev||payload),
`hash()` vs `keyed_hash()`; tiga skenario (hapus / ubah+rekomputasi / ubah+rekomputasi dengan kunci).
Hasil angka:
- root asli (unkeyed, 3 entri)          = `3b8624d5…8734`
- hapus-tanpa-rekomputasi               = TERDETEKSI (mismatch)
- ubah+rekomputasi (unkeyed)            = LOLOS verifikasi internal (G-C1b GAGAL)
- root keyed asli                        = `66f99a35…2a9c`
- ubah+rekomputasi **dengan chain_key**  = LOLOS (`true`)
- G-C6 KAT BLAKE3('') & SHA-256('')      = LULUS
- `blake3::Hasher` = 1.920 B; `sha2::Sha256` = 112 B; 2 hasher = 94% budget, 3 = 141% (MELANGGAR)

## A1.7 Koreksi lain sesi ini (untuk pola, bukan untuk alasan)
| Klaim awal saya | Terukur | Sumber |
|---|---|---|
| `size_of::<blake3::Hasher>()` = 372 B | **1.920 B** (5,2x) | #604 |
| `derive_key(CTX)` 1-arg | 2-arg `(context, key_material)` | #518 butir 3, #555 |
| keyed chain menutup C-01 | perlu **tetapi tidak cukup**; anchor yang menjamin | addendum ini |
| C-10 (duplikat ERR-012) | ditarik, #553 | #553 |
| G-C1a "terdeteksi" (uji pertama memakai `|| true`) | diuji ulang dengan benar: terdeteksi | #646 |
Pola yang konsisten: **analisis statik saya tahan uji; penekanan saya tidak.** Mitigasinya bukan
berhati-hati lebih keras, tapi menandai setiap klaim jaminan `[PERLU-UJI-RUNTIME]` lalu benar-benar
menjalankan ujinya, termasuk kasus yang akan membatalkan rekomendasi saya sendiri.
