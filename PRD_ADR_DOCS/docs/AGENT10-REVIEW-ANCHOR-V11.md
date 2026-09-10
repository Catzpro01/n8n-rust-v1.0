# REVIEW INDEPENDEN — W0-ANCHOR-IMPL v1.1 (patch A4 / D-A5)

| | |
|---|---|
| **Reviewer** | agent10 (ROLE_COMPLIANCE) — pemilik spec `AGENT10-W0-ANCHOR-SPEC.md` v1.1 |
| **Implementor** | agent1 |
| **Tanggal review** | 2026-09-09 (UTC) |
| **Obyek review** | `/opt/agent-workspace/w0-anchor-v11/` (`src/main.rs`, `tests/anc_gates.sh`, `V11.patch`) |
| **Verdict** | **APPROVED** — A4 dan D-A5 tertutup. 21/21 gate PASS direproduksi sendiri, 0 FAIL. |
| **Sisa open item** | 1 (administratif, di luar kode): penyerahan CA fingerprint FreeTSA ke Pemilik Proyek secara out-of-band. |

Dokumen ini bukti untuk syarat merge kanonik @matt butir (c) *review independen*, dan untuk Pilar 5 SOP
ZERO-COLLISION (implementor ≠ reviewer). Reviewer **bukan** penulis kode; reviewer adalah penulis spec.

---

## 1. Metode

Review dilakukan dengan **membangun dari sumber dan menjalankan gate sendiri**, bukan dengan memeriksa
klaim implementor. Urutan:

1. Rekam sha256 seluruh sumber yang direview (bagian 2) → identitas obyek tidak bisa berubah tanpa terdeteksi.
2. Build release di lingkungan reviewer sendiri (`CARGO_HOME` & `CARGO_TARGET_DIR` terpisah di
   `/mnt/extra-storage`), rekam sha256 biner hasil build.
3. Jalankan `cargo test --release --locked` (unit).
4. Jalankan `tests/anc_gates.sh` (gate end-to-end, memakai TSA nyata FreeTSA + OpenSSL untuk verifikasi TSR).
5. Bandingkan perilaku terukur terhadap pasal spec v1.1 (A4, D-A5) dan terhadap gate yang reviewer usulkan
   sebelumnya (ANC-8, ANC-8b/c/d/e, ANC-9, ANC-9b).

## 2. Identitas obyek review (bukti tak-terbantahkan)

| Artefak | sha256 | Ukuran |
|---|---|---|
| `src/main.rs` | `a01e1ee2d2be1c045db100ea8b9de8d02e818b35f21ce5527af183573a3873b7` | 1224 baris |
| `tests/anc_gates.sh` | `f448e138a4c1ac1635961e2dceb8f676337d3632af41c99ef239e9ad75a26ebc` | 141 baris |
| `V11.patch` | `672ad8e7d96b3d82f2783b2f10323675a803d65ed931f9f4f30614f28f4b5185` | 305 baris |
| biner `anchor` (build reviewer) | `139b489bd28992ce5f229c95264c59889885b80de2bf106f71dc50f45a5d0869` | 2.711.960 byte |

Perintah build yang berhasil (lihat §6 tentang kegagalan build dengan paralelisme default):

```bash
export CARGO_HOME=/mnt/extra-storage/agent10-cargo-home
export CARGO_TARGET_DIR=/mnt/extra-storage/agent10-cargo-target-v11
cd /opt/agent-workspace/w0-anchor-v11
cargo build --release --locked -j 1
```

## 3. Hasil unit test

```
test result: ok. 7 passed; 0 failed; 0 ignored
```

## 4. Hasil gate — 21 PASS / 0 FAIL

| Gate | Yang dibuktikan | Hasil |
|---|---|---|
| ANC-1 | create→anchor nyata FreeTSA, baris ANCHORED tercatat | PASS |
| ANC-2 | verify ulang (offline, tanpa create) → verdict sama | PASS |
| ANC-3 | tamper TSR → `MISMATCH` | PASS |
| ANC-3b | tamper head di DB → `MISMATCH` | PASS |
| ANC-4 / 4a | TSR palsu lokal lolos terhadap CA palsu (kontrol positif) tetapi **ditolak** terhadap pin FreeTSA | PASS |
| ANC-5 / 5b | verifier **menolak jalan** tanpa `--pin` (exit 2 + POLICY REFUSAL) | PASS |
| ANC-6 | replay TSR lama untuk window baru → `ANCHOR-MISMATCH` di baris ke-2 | PASS |
| V8 | window dengan gap (`entry_from`/`entry_to` tidak menyambung) ditolak saat create | PASS |
| ANC-7 / 7b / 7c | TSA mati → baris `FAILED`, alarm, **exit 0** (non-blocking, eksekusi lanjut) | PASS |
| **ANC-8** | `verify` dengan pin ≠ baris terakhir `pin_change_log` → **exit 2** | PASS |
| **ANC-8b** | alarm `PIN-MISMATCH` tercetak | PASS |
| **ANC-8c** | `--expected-fp` salah → exit 2 (ditolak) | PASS |
| **ANC-8d** | `--expected-fp` benar → exit 0 (diterima) | PASS |
| **ANC-8e** | baris seed `bootstrap/gates` benar-benar tercatat → **gate anti-vacuous** | PASS |
| **ANC-9** | file pin ditukar tanpa baris log baru → `verify` menolak (exit 2) | PASS |
| **ANC-9b** | jumlah baris `pin_change_log` tidak berubah setelah percobaan itu | PASS |

### Pemetaan ke pasal spec

- **A4 — seed bootstrap `pin_change_log`.** Penggunaan pin pertama per TSA kini menulis baris seed
  (`bootstrap/<authorized-by>`) sehingga "baris terakhir" selalu terdefinisi. Ditutup oleh ANC-8e.
  Penting: tanpa ANC-8e, ANC-8/ANC-9 bisa lulus **secara vacuous** (tabel kosong membuat pemeriksaan
  tak pernah diuji). Gate anti-vacuous itu ada dan PASS.
- **D-A5 — `--expected-fp` + enforce baris terakhir.** Verifier menolak bila fingerprint yang dipakai
  tidak sama dengan `--expected-fp` yang diberikan operator, dan tidak sama dengan baris terakhir log.
  Ditutup oleh ANC-8/8b/8c/8d + ANC-9/9b.
- Rotasi pin tetap memerlukan `--authorized-by` **dan** `--pin-change-reason` → jejak audit rotasi
  tidak bisa kosong.

## 5. Yang TIDAK diubah oleh patch ini (batas klaim)

Review ini **hanya** menyatakan A4/D-A5 tertutup dan tidak ada regresi pada ANC-1..ANC-7/V8.
Review ini **tidak** menyatakan:

- Anchor membuat hash chain menjadi "cukup" tanpa keyed-chain — sebaliknya, keduanya perlu
  (koreksi saya sendiri yang sudah dipublikasikan; anchor adalah yang **menjamin**, keyed chain
  adalah yang **perlu**).
- FreeTSA cocok untuk produksi. Ini TSA publik tanpa SLA; pinning CA + quorum M-of-M sudah ada,
  tetapi keputusan TSA produksi ada pada Pemilik Proyek.
- Multi-tenancy (dicabut oleh dekrit) atau aspek determinisme eksekusi (itu `W2-DETERM-ENFORCE`).

## 6. Catatan reproduksi untuk agen lain (BUKAN cacat kode)

`cargo build --release` dengan paralelisme default **gagal** di `libsqlite3-sys`:

```
error occurred in cc-rs: command did not execute successfully (exit status: 1)
```

Penyebab terukur: kontensi resource pada host — 2 vCPU, RAM 5,9 GB, dan **disk root `/dev/vda1` 98 %
terpakai (sisa 563 MB)** sementara `/mnt/extra-storage` punya 21 GB bebas. `sqlite3.c` adalah satu
unit kompilasi C yang besar; kegagalan muncul saat beberapa build berjalan bersamaan.

Dengan `-j 1` build sukses dan seluruh gate lulus. Karena itu:

- Jangan melaporkan kegagalan build ini sebagai bug implementor.
- Set `CARGO_TARGET_DIR` ke `/mnt/extra-storage/...` (sudah saya lakukan).
- Keadaan disk root sudah dilaporkan ke @agent9/@matt sebagai risiko infra.

## 7. Open item yang tersisa (butuh tindakan manusia, bukan kode)

| ID | Butir | Pemilik | Status |
|---|---|---|---|
| D-A5-admin | Serahkan CA fingerprint FreeTSA ke Pemilik Proyek **out-of-band** (jangan lewat channel agent): `A6:37:9E:7C:EC:C0:5F:AA:3C:BF:07:60:13:D7:45:E3:27:BB:BA:A3:8C:0B:9A:F2:24:69:D4:70:1D:18:AA:BC`, sha256(`freetsa-ca.pem`) = `2151b61137ffa86bf664691ba67e7da0b19f98c758e3d228d5d8ebf27e044438` | Pemilik Proyek | **TERBUKA** |
| TSA-sekunder | TSA kedua untuk quorum nyata: `timestamp.comodoca.com` memberi HTTP 200/Granted tetapi verifikasi chain gagal (intermediate tidak lengkap); `tsa.certum.pl` timeout | agent1 + Pemilik Proyek | TERBUKA |

## 8. Verdict

**APPROVED untuk merge kanonik** oleh Single Gatekeeper (@matt) bersama syarat (a) tests hijau dan
(b) clippy 0 warning — keduanya kewenangan matt untuk diverifikasi ulang pada pohon kanonik
`kernel-asli-d3bcff0`. Reviewer tidak melakukan merge; merge hanya lewat matt (SOP Pilar 2).
