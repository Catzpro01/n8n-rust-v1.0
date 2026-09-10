# RFC — W3-OPENAPI-EXEC: Jalur Eksekusi Node OpenAPI Ter-generate (v0.3)

**Pengusul:** agent9 (ROLE_INTEGRATION) · **Tanggal:** 2026-09-09 · **Status:** v0.3 — **RATIFIED-AS-DIRECTION matt Ruling 46 (#1408)**; perubahan wajib (a)+(b) diterapkan sejak v0.2; v0.3 menggabungkan **review-dini agent10 #1413 §4** (kepatuhan Ruling 12/32/33). **Eksekusi kode HANYA post-merge W3-OPENAPI-IMPL** (gerbang tersisa: agent4 EMIT + agent10 verifikasi).
**Acuan:** AGENT9-INTEGRATION-SPEC v0.5.3; kernel `context.rs` (D28/D72/D92), `error.rs` §3.4 (merged 89fdb3b); Ruling 12 (taksonomi 7 varian, larang fork-synonym), Ruling 32 (retryability = sifat penyebab), Ruling 46; R-1/R-2 #933; adapter agent6 v0.3; review agent10 #1413 §4.

## 0. Masalah

`nodes-openapi` v0.1 menghasilkan `NodeDescriptor` (form/validasi) tetapi belum punya `execute()`. RFC ini mendefinisikan kontrak eksekusi node hasil-generate — murni di atas layanan kernel, nol dependensi jaringan baru.

## 1. Prinsip kunci

1. **SEMUA I/O via `NodeContext`**: request via `ctx.http` (trait `HttpClient`, D28: pooled/streaming/**size-gated**); kredensial via `ctx.credentials` (D92); TIDAK ADA reqwest/hyper/tokio di crate node (matt #1116 b4).
2. **Determinisme output**: `HttpResponse.duration_ms` (wall-clock) **DILARANG masuk Item DAN digest** (R-2 + agent10 #1413: bila durasi ikut digest, 2 run seed sama ≠ digest sama — determinisme rusak; lihat INTEG-07). Node = fungsi murni dari `(params, input, response)`; replay dibaca dari record-replay engine.
3. **Retry hanya utk idempotent** (SideEffect kernel): backoff eksponensial + **jitter deterministik `f(input_digest, attempt)`** — BUKAN RNG.
4. **SSRF resolve-then-check** sebelum kirim (R-1): tolak IP private/link-local/metadata; skema hanya http/https (Q-2 #1031).

## 2. Keputusan (Ruling 46 + review agent10 #1413)

### (a) Pemilihan `servers` — TANPA pemilihan diam-diam + validasi EMIT-time [agent10 Q1]

| Kondisi | Perilaku |
|---|---|
| `servers` kosong/absen | **TOLAK** — error terlihat yang menyebut dokumen + path |
| `servers` tepat 1 | pakai, dan **REKAM URL-nya di `NodeDescriptor`** |
| `servers` > 1 | **JANGAN pilih** — field konfigurasi/credential diisi pengguna, SEMUA kandidat terdaftar sebagai opsi |

**Tambahan v0.3 (agent10 Q1): validasi di EMIT, bukan runtime** — setiap `servers[i].url` HARUS absolute http(s); URL relatif → TOLAK saat EMIT (OpenAPI 3.1 mengizinkan relative, tapi resolve-then-check yang bergantung basis dokumen = non-deterministik; tolak di EMIT = fail-closed di waktu yang tepat).

### (b) `max_response_bytes` — 10 MiB default GOVERNOR-SETTABLE + varian error eksak [agent10 Q2]

1. **Bukan konstanta**: default 10 MiB per-eksekusi, disetel governor, dihitung terhadap hard-cap fern (RSS <500MB).
2. **Cek INKREMENTAL selama pembacaan** (bukan pasca-buffer) — prinsip R39b.
3. **> ambang spill → `FileSpillStore`** (data-plane), BUKAN ditolak.
4. **Varian error (v0.3, resolusi Q2→Q3)**: pelanggaran size-gate di atas governor-max → **`Permanent`** (penyebab tidak berubah oleh retry: responsnya memang sebesar itu; `message` membawa limit + byte terbaca + ambang spill). **`ResourceExhausted` TIDAK dipakai** (agent10: D5 — field requested/available bermakna memori/kapasitas sistem, bukan kebijakan ukuran respons). usulan ini milik ranah kontrak agent1 (kernel owner) — ditandai utk konfirmasi.

### (c) Pemetaan error → taksonomi §3.4 — NOL varian baru [agent10 Q3; Ruling 12]

`NodeError` = **TEPAT 7 varian** (Ruling 12; larang fork-synonym): `Cancelled, Expression, Internal, Permanent, ResourceExhausted, Timeout, Transient`. v0.1 pernah menyebut `RateLimited{retry_after}`/`NotFound` — **DUA-DUANYA DIHAPUS sejak v0.2** (tidak ada di taksonomi; menambah varian = amendemen §3.4 = jalur RFC+kuorum, bukan keputusan satu-satu).

| Penyebab | Pemetaan (varian yang ADA) |
|---|---|
| HTTP 429 | `Transient { retry_after: Some(Retry-After) }` — field `retry_after` SUDAH ADA di varian |
| HTTP 5xx / 408 | `Transient` + backoff |
| HTTP 4xx selain 408/429 (401/403/404/422/…) | `Permanent { code, message }` — status HTTP dibawa di `message` (pola `Internal{message}` R17 baris-6) |
| timeout / koneksi putus | `Transient`, HANYA metode idempotent (non-idempotent → semantik InDoubt kernel) |
| SSRF ditolak | `Permanent`, JANGAN pernah retry |
| size-gate > governor-max | `Permanent` (§2b-4) |

### (d) Nama task — `W3-OPENAPI-EXEC-IMPL` [disetujui Ruling 46]

ID `task_queue` TEXT case-sensitive: verifikasi kapitalisasi PERSIS pra-INSERT + tempel keluaran query baris-ada post-INSERT (INTEG-12).

## 3. Alur eksekusi (v0.3)

```
EMIT-time (codegen, sudah bisa berlaku saat regenerate):
  - validasi servers §2a: 0 → tolak; semua entri harus absolute http(s)
  - servers-1 → URL terekam di NodeDescriptor; servers->1 → field konfigurasi + opsi

execute(ctx):
 1. Routing dari manifest node + params user (divalidasi ParameterSchema kernel)
 2. Susun HttpRequest:
      url    = aturan servers §2a + path-template {var} (di-URL-encode)
      query  = BTreeMap (kunci terurut; list di-sort kanonik)
      headers= kredensial via CredentialSpec kind
      body   = RequestBody::Json | RequestBody::Form
      max_response_bytes = governor (default 10 MiB)
      timeout_ms         = default 10_000, override per-node
 3. SSRF check: resolve host → tolak private/link-local/metadata (fail-closed, Permanent)
 4. ctx.http.send(req) → HttpResponse   [D28: cek inkremental §2b; >spill → Blob]
 5. Mapping hasil §2c (duration_ms TIDAK masuk Item/digest)
```

## 4. Batas v0.1 eksekusi (jujur)

- `RequestBody::Bytes`/multipart: TIDAK didukung (konsisten CG-E-205).
- Streaming SSE = bukan scope (layer terpisah).
- Polling/pagination otomatis: TIDAK (user memakai loop workflow).

## 5. Gate falsifiable

| Gate | Kriteria | Cara uji |
|---|---|---|
| INTEG-07 e2e deterministik | 2 run seed sama → Item & **digest** identik; assert `duration_ms` tidak muncul di output **DAN tidak masuk digest** (agent10 #1413) | harness mock HttpClient |
| INTEG-08 size-gate | respons > governor-limit → `Permanent` eksak / spill-ke-Blob, BUKAN OOM; cek inkremental terbukti (stream dipotong sebelum habis) | mock respons besar |
| INTEG-08b servers | 3 fixture: servers-kosong → tolak; servers-1 → URL terekam; servers-2 → error konfigurasi + opsi; + URL relatif → TOLAK di EMIT (v0.3) | unit test |
| INTEG-09 leak runtime | request/log/metrics discan pola nilai kredensial = 0; **test MENYENTUH JALUR PRODUKSI, bukan hanya mock** (agent10 #1413, pola #1290 §2) | harness + grep run output |
| INTEG-10 SSRF corpus | 20 kasus corpus R-1 → semua ditolak Permanent, 0 retry | unit test reuse fixture |
| INTEG-11 retry deterministik | backoff+jitter = f(input_digest, attempt); 2 run seed sama = urutan retry identik; 429+Retry-After dihormati (cap backoff); 4xx≠408/429 tak pernah retry | unit test |
| INTEG-12 task-queue INSERT | kapitalisasi terverifikasi + keluaran query baris-ada ditempel | bukti SQL |

## 6. Status ratifikasi & changelog

- **matt Ruling 46 (#1408): DITERIMA SEBAGAI ARAH = ratifikasi dual-mode.** (a)+(b) wajib → diterapkan v0.2; (c) kepatuhan taksonomi; (d) disetujui.
- **agent10 #1413 §4 (review-dini, domain compliance)**: Q1 → validasi EMIT-time absolute-URL masuk v0.3; Q2 → `ResourceExhausted` ditolak, usul `Permanent` (menunggu konfirmasi agent1); Q3 → dikonfirmasi v0.2+ sudah nol varian baru (RateLimited/NotFound dihapus); INTEG-07 digest-assert & INTEG-09 production-path masuk v0.3.
- v0.1→v0.2: (a) servers 0/1/>1, (b) governor+inkremental+spill, (c) taksonomi eksak, (d) prosedur INSERT, INTEG-08b/12.
- v0.2→v0.3: EMIT-time absolute-URL, size-gate→Permanent, digest-assert INTEG-07, production-path INTEG-09, catatan Ruling 12.
- Kode tetap menunggu gerbang merge M3: agent4 (EMIT) + agent10 (verifikasi).
