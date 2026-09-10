# W0-CONTEXT-CONTRACT — Laporan (agent10, sesi B)

| | |
|---|---|
| **Task** | `W0-CONTEXT-CONTRACT` (Wave 0, P1, ROLE_SECURITY_QA — adopsi agent10 Plt.) |
| **Basis** | `docs/W0-CONTEXT-CONTRACT-TEST.md` (matt, #912 — 24 uji CT-01..CT-08) + `kernel-asli-d3bcff0/crates/kernel/src/context.rs` (a7d0357) |
| **Artefak** | `/opt/agent-workspace/w0-context-contract/` = `context_contract.rs` (suites) + `context-3.1.patch` (keputusan §3.1) |
| **Shadow** | `/home/agent10/w0-context-contract/kernel-shadow/` (salinan kanonik + patch; TIDAK menyentuh kanonik — SOP #864) |

---

## 1. Hasil terukur (shadow workspace, rustc 1.98.1)

```
cargo test -p kernel            -> 19/19 lama HIJAU (tanpa regresi) + 34/35 baru (1 ignored, alasan §2)
cargo clippy --all-targets      -> 0 warning, 0 error
```

| Grup | Uji | Status |
|---|---|---|
| CT-01 PriorOutputs | a (case-sensitive name), b (None vs Err(NodeNotFound)), c (by_id≡by_name), d (IndexOutOfBounds bukan panic), e (Spilled tanpa materialisasi — `ram_footprint` metadata-only), f (available_nodes = hanya selesai) | 6/6 ✅ |
| CT-02 ExpressionEngine | a (validate TIDAK eksekusi), b (sintaks≠evaluasi), c (eval_batch≡eval per expr), d (kosong→Ok), e (error bawa teks asli), f (kompilasi semua field scope D32 + shape env) | 6/6 ✅ |
| CT-03 EnvAccess | a (whitelist deny secret), b (bentuk API hanya get — review-gate utk `keys()`/`iter()`) | 2/2 ✅ |
| CT-04 CredentialProvider | a (kind tak dideklarasi → Err), b (Ok), c (set akses TIDAK lebih dari deklarasi — D92), d (**ALARM §3.1**: Logger terima `&LogFields`, bukan `&Value`) | 4/4 ✅ |
| CT-05 HttpClient | d (RequestBody::Blob tanpa BlobStore::get — never fully in RAM), e (BTreeMap urutan deterministik) | 2/2 ✅ |
| CT-06 BlobStore | a byte-identik, b put/size konsisten, c unknown→Err/None, d delete→Err/None, e delete idempoten, f (perilaku mock DIRECAM — keputusan K-8/W2-CASD-DEDUP) | 6/6 ✅ |
| CT-07 CancellationToken | a NoCancel false, b reason default None, c reason, d pembatalan teramati di iterasi (batas wajar) | 4/4 ✅ |
| CT-08 Logger | a Noop tak panic, b LogLevel serde lowercase (wire-format; `"Info"` GAGAL deserialize — alarm pola lama), c **redaksi by construction** (0 plaintext di field log), d jalur log tanpa escape | 4/4 ✅ |

## 2. Keputusan §3.1 (diminta #912 @fern + agent10): **OPSI (a) — tegas lewat tipe**

**Pertanyaan:** klaim doc `:394-395`/`:514-515` ("logger cannot leak them", "a node cannot opt out") lebih kuat dari API (`get()`/`as_value()` → `&Value` mentah; `Logger::log(.., &Value)`).

**Keputusan gatekeeper (agent10, Plt. ROLE_SECURITY_QA — siap dibantah fern/matt 30 menit):**
1. `Logger::log(level, message, fields: &LogFields)` — `LogFields` = `BTreeMap<String,Value>` + `put()` (data publik) + `put_credential()` (menulis `{"redacted": true, "type": "CredentialValue"}` — nilai TIDAK PERNAH masuk, by construction).
2. `CredentialValue::as_value()` **DIHAPUS** → diganti `#[doc(hidden)] reveal()` (trusted integration only, D92). `get()` dipertahankan (kebutuhan agen9 membangun header) + doc diperbarui.
3. Precedent dalam file: `ExpressionScope` Debug manual `:351-355` — rahasia dikeluarkan **berdasarkan konstruksi tipe** (D94). Ini patokan yang sama.
4. Biaya terukur: **NOL konsumen** — satu-satunya `impl Logger` = `NoopLogger` (grep `impl Logger` di kanonik: 0 selain Noop); `CredentialProvider` belum diimplementasikan siapa pun. Jadi (a) murah sekarang, mahal nanti — keputusan tepat waktu.
5. **Patch siap-kanonik:** `context-3.1.patch` (77 baris, hanya context.rs; berlaku `patch -p1` dari root kernel-asli). **Kepada @agent1** (owner crates/kernel/ per SOP Pilar 2): mohon aplikasi + re-run; **@matt**: merge via pintu tunggal.

**CT-05a/CT-05b DITAHAN (1 ignored):** plafon memori butuh `KernelError::ResourceExhausted { resource, limit, actual }` (+ `Timeout { elapsed_ms }`) yang belum ada di error.rs kanonik. Rekomendasi: tambah varian (agent1/matt) lalu aktifkan; `ResponseBody::TooLarge` = penanda arah benar. **CT-06f** tidak diberi assert tetap (keputusan K-8 belum ada — perilaku mock non-CASD direkam). **CT-08b bonus gate**: deserialisasi `"Info"` (capitalized) GAGAL → alarm bila `rename_all="lowercase"` hilang.

## 3. Tindak lanjut yang diminta (bukan scope saya)

1. @agent1: aplikasi `context-3.1.patch`; tambah varian `ResourceExhausted`/`Timeout` (setelah keputusan → CT-05a/05b aktif).
2. @matt: merge patch + test ke kanonik (syarat #865: 19+34 hijau ✅, clippy 0 ✅, review @agent5/QA ⏳).
3. **Cakupan nyata**: llvm-cov belum diukur di sesi ini (plan §4: "ukur yang nyata, ganti angka proxy 40%") — menunggu toolchain cov di VPS; catat sebagai open item (bukan klaim).

**Status: DONE (34 uji dlm shadow + 19 regresi hijau + clippy 0; keputusan §3.1 tertulis; patch ter-publish).** — agent10 (ROLE_COMPLIANCE / Plt. ROLE_SECURITY_QA, sesi B)
