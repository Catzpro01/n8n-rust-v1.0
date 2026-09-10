# W0-CONTEXT LANDING — LogFields/D93 + tests/context_contract.rs ke kernel kanonik

- Status: STAGED (pohon kerja, BELUM commit — menunggu review 2/2 + otorisasi fern, R41 #1345 langkah 6)
- Perintah: RULING 41 §6 (matt #1345)
- Jalur: sama dengan §3.4 (terapkan → uji → dokumen → review → otorisasi → commit)
- Penulis pendaratan: agent1 (kernel owner). 2026-09-09 ~14:10 WIB.

## 1. Provenance (dari mana artefak ini datang)

Satu-satunya salinan pra-pendaratan ada di direktori preservasi (BUKAN repo,
tanpa Cargo.toml, tak bisa diuji) — diselamatkan dari kernel-shadow yang
dihapus per Ruling 40 (laporan #1337):

    /mnt/extra-storage/agent1-work/preserved-w0-context/
        context_contract.rs        25196 B  sha256 fee1d62b586de993…
        context-LogFields.patch     3557 B  (unified diff vs context.rs kanonik)
        check-freeze.sh            10011 B  (perkakas guard freeze, bukan kode kernel)
        README.txt                  1224 B

Plan: `docs/W0-CONTEXT-CONTRACT-TEST.md` (sudah di kanonik) +
`docs/AGENT10-W0-CONTEXT-CONTRACT-REPORT.md` (laporan agent10).

Risiko single-copy yang disebut matt (#1345 §6) kini terpotong: salinan ada
dua (preservasi + pohon kerja kanonik). Commit menutupnya penuh setelah
langkah 6.

## 2. Perubahan 1 — src/context.rs: LogFields / put_credential / D93 (+42/-6)

Patch `context-LogFields.patch` terap BERSIH (4/4 hunks, tanpa fuzz):

    crates/kernel/src/context.rs | 48 ++++++++++++++++------
    1 file changed, 42 insertions(+), 6 deletions(-)

Isi (keputusan W0-CONTEXT §3.1 opsi (a)):

- `LogFields(BTreeMap<String, Value>)`: `put()` untuk data publik,
  `put_credential()` HANYA menulis penanda `{redacted: true, type:
  "CredentialValue"}` — nilai kredensial TIDAK PERNAH masuk jalur log.
- `CredentialValue::as_value()` DIHAPUS dari jangkauan log; `Logger::log`
  menerima `&LogFields`, bukan `&Value` (redaksi D93 by-construction:
  node tak bisa opt-out).
- `CredentialValue::reveal()` ditandai `#[doc(hidden)]` — hanya untuk
  integrasi TRUSTED (D92, mis. membangun header Authorization), bukan log.

## 3. Perubahan 2 — tests/context_contract.rs: 35 uji CT-01..CT-08 (685 baris)

Disalin utuh dari preservasi, lalu SATU uji diaktifkan (lihat §4). Tanpa
tokio (mini-executor `block_on` pola contract.rs — aturan freeze kernel).
Mock = executable spec: MockCreds (D92 least-privilege), MockHttp
(max_response_bytes, BTreeMap ordering), MockBlobStore (never-fully-in-RAM),
CapturingLogger, token cancel.

Peta sebar (nama uji `c_tXXy_…`): CT-01 availability, CT-02 eval-sidecars,
CT-03 credentials-store, CT-04 credential-value + redaction-by-construction
(CT-04d = alarm API: bila `&Value` dikembalikan ke `Logger::log`, uji gagal
kompilasi), CT-05 HttpClient, CT-06 BlobStore, CT-07 cancellation, CT-08
logger/noop + serde wire-format.

## 4. Perubahan 3 — CT-05a DIAKTIFKAN dengan badan uji NYATA (langkah 3)

Keadaan asal: `#[ignore]` + badan KOSONG (`{}`).

KOREKSI PENTING (kesalahan saya di #1337): ignore yang ada adalah SATU,
bukan dua — `grep -c 'ignore'` saya menghitung baris komentar doc (`//! …
#[ignore] …`) sebagai temuan kedua. matt menulis "2 ignore" mengikuti angka
saya; yang benar: 1 atribut ignore (baris 488 pra-edit). Koreksi ini
tercatat di kanal bersama laporan ini.

Un-ignore vakum (badan kosong → lulus trivial) adalah klaim kosong, jadi
badan diisi gerbang nyata — mock `CappedHttp` menegakkan
`max_response_bytes`:

- Kontrol positif: 512 B di bawah plafon 1024 → `Ok`.
- Oversized (1_000_000 vs plafon 1024) → `Err(ResourceExhausted{Memory,
  requested: 1_000_000, available: 1024})` — angka dieksplisitkan, bukan
  hanya varian.
- Pesan Display menyebut "memory".
- Rantai §3.4 penuh: `NodeError::from(err)` → `!is_retryable()` (Ruling 32:
  Memory = keputusan governor, jalur re-queue, bukan retry node).

Impor ditambah: `NodeError, Resource`. Komentar doc `DITAHAN` → `AKTIF`.

## 5. Bukti uji (langkah 4) — dijalankan ~14:1x WIB

    cargo test -p kernel --test context_contract
      test result: ok. 35 passed; 0 failed; 0 ignored
    cargo test -p kernel   (penuh: lib + context_contract + contract)
      1 passed + 35 passed + 19 passed; 0 failed; 0 ignored
    cargo clippy -p kernel --all-targets → 0 warning, 0 error

`contract.rs` (19) tetap hijau: patch context.rs tidak merusak kontrak lama.
`c_t05a_oversized_response_is_resource_error` diverifikasi lulus mandiri
(`--test context_contract c_t05a` → 1 passed).

Target-dir: `/mnt/extra-storage/cargo-target-canonverify` (cache bersama,
tak menyentuh pohon).

## 6. Status pohon + permintaan review (langkah 6)

    M  crates/kernel/src/context.rs
    ?? crates/kernel/tests/context_contract.rs
    HEAD masih 047c0fa (TIDAK ADA commit — freeze dipatuhi)

Minta review (2/2, Ruling 23): @agent10 + @agent3. Setelah APPROVE 2/2,
minta otorisasi @fern, lalu commit bentuk baku 89fdb3b.

Catatan untuk reviewer: (a) diff context.rs kecil (48 baris) — baca penuh;
(b) context_contract.rs 685 baris — fokus pada CT-05a baru (§4) +
CT-04d/CT-08c (redaksi); sisanya mock-spec dari sesi pagi, tak diubah
kecuali impor + header; (c) satu-satunya logika produksi baru adalah
LogFields/put_credential — murni, tanpa I/O, tanpa policy di luar redaksi.

## 7. Sisa yang BUKAN bagian pendaratan ini

- `json_depth_within` ke kernel (R41 §5): dikerjakan @agent6 (penulis),
  mendarat terpisah lewat jalur yang sama; saya gatekeeper review-nya.
- Logging B-1 penuh (matt #1317 §5: ikat kedalaman struktur rekursif):
  LogFields ini fondasinya; Logger produksi + MAX depth menyusul perintah.

— agent1
