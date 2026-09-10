# AGENT6-WCB-SPEC — WASM Community Node Bridge (WCB)

> **Pemilik:** agent6 (ROLE_WASM) · **Tanggal:** 2026-09-09
> **Status:** DRAFT v0.4 — dokumen DESAIN/ARSITEKTUR. **Zero implementation code** (mandat #386 & #498; agent1 #645: kode Wave-1 menunggu sign-off pemilik). Implementasi hanya setelah PRD-3 disahkan.
> **v0.3 (revisi):** integrasi **satu kerangka manifest_sig Ed25519** utk Hub-template & WCB-plugin (disepakati agent7 #624, kontrak agent10 #602/#612, TOCTOU #619/#622, pengesahan #621). SEC-WCB-04 diperluas: tanda tangan naik ke "didukung & dianjurkan", kanon yang ditandatangani, rotasi dual-key (SEC-TRUST-01), verifikasi dua-tahap. Namespace registri `n8n://plugin/{id}/manifest` (D-W6 diputuskan #624).
> **v0.4 (revisi):** serap review agent3 #663 (original proposer) + jawaban engine agent1 #664: (C.1) seed invokasi bertingkat `BLAKE3(execution_id ‖ plugin_id ‖ invocation_depth)`; (C.2) `output_strategy` manifest (`spill|stream|auto`, default auto host-heuristic); (C.3) propagasi error → node-FAILED + E-WCB-TRAP/FUEL/TIMEOUT (slot agent1 #664) → routing onError (default STOP, cabang-error bila continue) — TOLAK silent-failure; (B.1) semantik warm/cold pool; (B.2) roadmap SDK Rust→AssemblyScript→Go; (B.3) registri lokal MVP → marketplace pasca-MVP.
> **Basis mandat:** Matriks 11 agen #498 (agent6 = WCB, wasmtime pooling allocator, 32MB isolation) · Adjudikasi WCB #431 (MAX 32MB/instance, pooling wajib) · Proposal #375 (agent3) · Sayembara #298/#362.
> **Rekonsiliasi:** SEC-WCB-01..04 kanon = bingkai agent6 (ditetapkan agent5 #488).

---

## 1. Ringkasan keputusan kunci (jejak keputusan)

| Keputusan | Nilai | Sumber |
|---|---|---|
| Runtime | **wasmtime** (Bytecode Alliance) | #467, feasibility agent6 |
| Batas memori/instance | **32 MiB linear memory guest** (512×64KiB) | #431; definisi di #470 (agent1 dukung) |
| Strategi instance | **Pooling allocator**, Engine+modul shared, Store reuse | #431 |
| ABI MVP | **wcb:* minimal TANPA WASI** (permukaan kecil, audit mudah) | Q3 #470 (agent1 dukung); agent6 |
| Egress jaringan MVP | **Allowlist statis per manifest** + host function `wcb:net/request` | Q2 #492 (agent6), arahan agent1/agent5 |
| Gate keamanan | SEC-WCB-01..04 (kanon di bawah) | #488 (agent5) |
| Boundary data | Kernel ItemList (inline/spilled) → postcard → host-streaming | #467, #470 |
| Posisi crate | `crates/wcb/` di LUAR kernel; aturan dependensi §3.3 PRD-2 tidak berubah | #467 |
| Trust/rantai pasok | **Satu kerangka manifest_sig Ed25519** (Hub-template & WCB-plugin); trust-anchor di registri agent7; rotasi dual-key (SEC-TRUST-01); TOCTOU dua-tahap (validate + first-load re-verify) | #611/#624 (agent7), #602/#612 (agent10), #621 fern, #619/#622 (agent1) |

Sifat WCB: **ekstensibilitas node komunitas** — TIDAK menggantikan expression engine (tetap QuickJS, PRD-2 §4.2) dan TIDAK menggantikan node inti.

---

## 2. Posisi arsitektur

```
kernel (trait: ItemList/SpillStore/Node)   ← 0 dep internal (§3.3)
   ▲
crates/wcb  (WASM bridge; 1 titik FFI, terpisah spt rquickjs)
   ├── wasmtime engine (pooling, 32MiB/instance)
   ├── host function wcb:* (entropy/secret/net/log)
   └── boundary ItemList ↔ guest (postcard + host-streaming)
executor → crates/* nodes  &  crates/wcb (plugin node)
```

- WCB hanya boleh bergantung pada **trait** kernel, bukan impl konkret → desain tidak terblokir total oleh K-1; implementasi final menunggu K-1 + PRD-3.
- Plugin node diregistrasi lewat `NodeRegistry` (trait di kernel) — pola sama dengan node inti, sehingga perbedaan internal (WASM vs native) tidak bocor ke workflow/executor.

### 2.1 Boundary tanggung jawab (dikunci review #577/#579)

```
Klien AI / HTTP
   │  (exec_endpoint = binding HTTP agent9 + admission §5.2 agent1 + orkestrasi engine agent1)
   ▼
admission-API (agent1): rate / antre / RAM-governor §5.2   ── gerbang lapis-1
   ▼
admission-sandbox (agent6): fuel / cap-instance / 32MiB / allowlist-import  ── gerbang lapis-2
   ▼
substrate eksekusi WCB (agent6): hosting instance, SEC-WCB-01..04,
   PRG-meter, host-streaming read_at/read_range, kontrak manifest
   ▼
konsumen: engine agent1 (per-invokasi) · kompresor agent1+3 · nodes komunitas
```

- **Dua gerbang admission SERI, lapis beda:** API-layer (agent1) tidak menggantikan sandbox-layer (agent6), dan sebaliknya. Satu-satunya jalur eksekusi = melewati keduanya.
- agent6 = **substrate**, bukan pemilik endpoint; endpoint = agent9 (HTTP) + agent1 (admission/orkestrasi).
- MCP (agent7) hanya discovery/katalog — eksekusi TIDAK lewat MCP (F-8/MCP-11, D-5 terkunci).

---

## 3. Siklus hidup plugin (desain)

1. **Install/publikasi:** artefak = `.wasm` + `manifest.json` (Satu file paket atau dua berkas terpisah — keputusan D-W3).
2. **Validate** (SEC-WCB-01): statis — import allowlist, hash modul vs manifest, ukuran berkas.
3. **Compile-once:** modul di-compile sekali per Engine (cache hasil kompilasi bila ada).
4. **Pool (semantik eksplisit #663-B.1):** `pool_size > 0` = **warm pool** (instance pre-initialized, siap pakai, utk node frekuensi-tinggi); `pool_size = 0` = **cold per-eksekusi** (compile-once, instance dibuat per-invokasi). Default 0 (#431). Nilai diturunkan dari budget (§9).
5. **Execute:** executor memanggil trait node → WCB menyiapkan instance → host-streaming input → seed entropy + injeksi secret (SEC-WCB-03) → fuel/timeout aktif → eksekusi → streaming output (sesuai `output_strategy`, §6) → ukur meter (PRG) → kembalikan ItemList ke jalur spill normal. **Error plugin** (trap/fuel-exhausted/timeout) → propagasi §6.3.
6. **Unload/evict:** instance dikembalikan ke pool atau di-terminate bila melanggar (SEC-WCB-02 / PRG).

---

## 4. Antarmuka host-guest (wcb:* — design-level)

Prinsip: **guest tidak punya ambient authority**. Semua kemampuan lewat fungsi host yang didaftarkan eksplisit di manifest.

Draf namespace (WIT-sketch, bukan final):

```
wcb:node/input    → streaming potongan input (lihat §6), bukan materialisasi penuh
wcb:node/output   → tulis potongan output
wcb:entropy/seed  → host suntik seed determinisme (dari execution_id)
wcb:secret/acquire→ ambil credential tertentu SEKALI per eksekusi (diinjeksi host)
wcb:net/request   → egress HANYA ke host terdaftar di manifest (allowlist)
wcb:log           → log(level, message) ke execution log
wcb:limits        → query sisa fuel / batas (untuk node sadar-resource, opsional)
```

- **Tanpa WASI pada MVP:** tidak ada `wasi_snapshot_preview1`, `fd_*`, `sock_*`, `proc_*`, clocks, random — ditolak di SEC-WCB-01.
- **Component model vs module model** = keputusan D-W2 (wasmtime mendukung keduanya; module-model-coret lebih sederhana utk MVP, component-model untuk ABI ber-tipe lintas bahasa pasca-MVP).

---

## 5. Model memori & pooling — akuntansi jujur

**Aturan #431: MAX 32 MiB per instance. Definisi (#470): batas LINEAR MEMORY guest.**

- Batas dipaksakan runtime (pooling allocator): linear memory maksimum 32 MiB (512 pages @64KiB).
- **Akuntansi jujur (ERR-029, #431):**
  - 32MiB = alokasi *addressable* guest; TIDAK sama dengan RSS total.
  - Pooling me-reservasi ruang *virtual* per slot; halaman *fisik* ter-commit saat guest memakainya.
  - Klaim yang boleh ditulis: *"≤32MiB addressable per instance; RSS idle ≈ halaman ter-commit + metadata instance"*.
  - RSS baseline engine + delta per instance + per-beban **wajib diukur** (gate agent8, satuan MB), bukan ditebak.
- `instance_limits.count` & `pool_size` diturunkan dari budget (lihat §9 PRG).

---

## 6. Integrasi kernel — boundary data (garis merah)

**Prinsip: WCB tidak boleh menghidupkan kembali masalah "single blob JSON" n8n.**

- Input dari `ItemList`:
  - Inline (kecil): serialize (postcard) → lift ke guest (copy 1×).
  - **Spilled (besar): HOST-STREAMING** via `read_at`/`read_range` + cursor kernel (#467; invarian akses-terindeks agent1 #470). Guest melihat *iterator* per potongan, bukan seluruh list.
- Output: guest menulis potongan → host-streaming → `ItemList::from_vec`/`spill_from_iter` + governor → spill store/CASD (dedup Layer-1 tetap berlaku).
- **`output_strategy`** (#663-C.2, #664): field manifest `output_strategy: "spill" | "stream" | "auto"` — default **"auto"** (host heuristic berdasar estimasi ukuran output). Aturan: output yang tak muat plafon guest harus **spill-atau-stream**, tidak pernah materialisasi penuh di guest (SEC-WCB-02 + PRG).
- **Biaya yang jujur:** isolasi memori ⇒ 1× serialize + 1× copy per arah; batas arsitektural WASM, bukan bug. Zero-copy penuh dilarang dicapai dengan mengorbankan isolasi.

### 6.3 Propagasi error plugin (#663-C.3; kontrak engine agent1 #664)

- WASM trap / fuel-exhausted / timeout → **node-FAILED** + error terstruktur **E-WCB-TRAP / E-WCB-FUEL / E-WCB-TIMEOUT** (slot kode dimiliki agent1, aktif saat spec ini landing).
- Routing mengikuti kebijakan `onError` workflow (paritas n8n): default **STOP**; bila node mengatur `continueOnFail` → **cabang error** dieksekusi.
- Error **WAJIB terlihat**: tercatat di execution receipt + status eksekusi + log. **TIDAK** boleh menjadi ItemList kosong senyap (silent failure ditolak).

---

## 7. Determinisme

- Seed berbasis `execution_id` → `wcb:entropy/seed`. Guest memakai untuk `Date.now`/random internalnya.
- **Invocasi bertingkat (plugin memanggil plugin)** (#663-C.1, #664): seed per-level = `BLAKE3(execution_id ‖ plugin_id ‖ invocation_depth)` — deterministik (pohon reproduktif) sekaligus unik per level (dua plugin di kedalaman sama tak berbagi aliran random).
- WASI clocks/random tidak diekspos (ditutup juga oleh SEC-WCB-01).
- Verifikasi determinisme WCB = **harness `qa/` (agent5)**; SEC-WCB menyediakan kontrol (agent5 #488, pembagian bersih).

---

## 8. SEC-WCB-01..04 (KANON — ditetapkan agent5 #488)

- **SEC-WCB-01 — Import allowlist & validasi statis modul.** Hanya namespace `wcb:*` + fungsi yang terdaftar di manifest; import `wasi_snapshot_preview1`/`fd_*`/`sock_*`/`proc_*`/wasi clock-random → DITOLAK saat VALIDATE. Hash modul ≠ manifest → ditolak.
  GATE (falsifiable): modul mengimpor fd_write/sock_open/proc_spawn/wasi-clock-random → ditolak pada VALIDATE; modul di-tamper → ditolak.
- **SEC-WCB-02 — Enforce sumber daya runtime.** Linear memory ≤32MiB (pooling); fuel + timeout (async) untuk infinite loop; cap instance konkuren (kelebihan → QUEUED / admission control, bukan ditolak diam-diam).
  GATE: modul alokasi >32MiB → di-kill/dibatasi; infinite loop → fuel/timeout mematikan; konkuren > cap → antre.
- **SEC-WCB-03 — Tanpa ambient authority.** Credential disuntik host sekali per eksekusi (`wcb:secret/acquire`); guest tidak bisa membaca credential di luar injeksi. Egress jaringan hanya via `wcb:net/request` + allowlist host per manifest.
  GATE: guest mencoba baca credential tanpa injeksi → GAGAL; egress ke host non-allowlist → DIBLOKIR.
- **SEC-WCB-04 — Rantai pasok & audit (kerangka manifest_sig terpadu).**
  `manifest.json` memuat: sha256 modul, versi pin, batas sumber daya, daftar import, host allowlist, **`manifest_sig` + `kid`** (Ed25519, didukung & dianjurkan). Jejak instal & eksekusi masuk execution receipt + event log tamper-evident (domain agent10).
  - **Satu kerangka dengan Hub-template** (#611/#624): kontrak agent10 (#602/#612) — field registri `{hub_id, kid, pubkey_ed25519, valid_from, valid_until, revoked}`, rotasi **dual-key overlap (SEC-TRUST-01)**, kid wajib dalam tanda tangan, fetch registri ulang sebelum verifikasi. Untuk plugin WCB: `hub_id` → `plugin_id`; penerbit = pengembang plugin.
  - **Kanon yang ditandatangani (WCB):** `{schema_version, plugin_id, module_sha256, content_version}`. Detail lain (imports/limits/allowlist) terikat karena hidup di manifest yang content_version-nya ditandatangani.
  - **Verifikasi dua-tahap (TOCTOU, #619/#622/#612):** (1) di VALIDATE/install-gate; (2) **ulang saat first-load** modul dari cache ke instance — perilaku **verify-or-refuse** (kontrak engine; gagal = hard-refuse, bukan retry-admission). Biaya: 1 verify Ed25519 + 1 lookup registri (di bawah batas transien).
  - **Registri:** satu trust-anchor (agent7, 1-owner-per-URI); namespace `n8n://plugin/{id}/manifest` (D-W6 diputuskan #624).
  GATE: manifest/modul di-tamper → hash mismatch → ditolak; sig invalid/kedaluwarsa/revoked → ditolak; receipt memuat hash modul + batas.

---

## 9. PRG — Plugin RAM Governor & Meter (tempat entri sayembara #493)

Sayembara agent6 #493 (PRG) melekat di sini sebagai lapisan kebijakan runtime:

- Probe per-instance **O(1)** (data_size/halaman ter-commit × 64KiB + glue host) dibaca tiap node-complete + interval epoch/fuel.
- Sinyal diagregasi ke **Pressure/Critical Governor** yang sudah ada (PRD-2 §6.4, `ItemList::ram_footprint()`) — PRG hanya menambah satu sumber sinyal ("budget ekstensi").
- **Admission control:** instance baru ditolak bila budget_pool penuh. **Eviction deterministik:** instance melanggar plafon → terminate + respawn bersih dari manifest; output antara sudah di spill → nol data hilang.
- Rumus: `budget_pool = baseline_engine + Σ(32MiB × instance_aktif) + margin` (terpisah dari 500MB engine; #431). Angka konkret ditetapkan di fase benchmark agent8.
- GATE (PRG, untuk agent8): 7-run; (a) RSS ≤ baseline + n×32MiB + 5%; (b) instance ke-(n+1) saat penuh → ditolak (7/7); (c) instance bocor > plafon → evict dalam X epoch, engine sehat (7/7).

---

## 10. Risiko & biaya yang jujur

| Item | Nilai/status |
|---|---|
| Dep wasmtime | crate besar; binary produk membesar. Preseden: rquickjs sudah jadi 1 titik FFI. Ukur di spike. |
| Compile time | wasmtime menambah waktu build; imbas ke toolchain shared /mnt/extra-storage/rust (base-only dulu; tambah target wasm32 saat implementasi). |
| Boundary copy | 1× serialize + 1× copy/arah — disepakati batas arsitektural (#470). |
| Toolchain | RESOLVED: shared rustc/cargo 1.98.1 + env global (/etc/environment; fix PATH/izin/registry #614). Target wasm32 + wasmtime saat implementasi (spike). |
| Target compile | MVP wcb-minimal ⇒ `wasm32-unknown-unknown`; cargo-component/wit-bindgen hanya bila component-model dipilih (D-W2). |
| K-1 (kernel kanonik) | Bridge ke trait; detail kontrak final mengikuti K-1 (Wave 0 remediasi PRD-3 v3.1 #651). |
| Disk | RESOLVED: vdb 30GB ter-mount (#504); CARGO_TARGET_DIR per-akun (#651 L4). |

---

## 11. Pertanyaan terbuka untuk PRD-3 (keputusan D-W*)

- **D-W1** — Contoh tipe node komunitas pertama yang didukung (mis. transform/set/parse) ditetapkan bersamaan MVP node list (T-13).
- **D-W2** — ABI: module-model core (MVP sederhana) vs component-model/WIT (pasca-MVP, multi-bahasa ber-tipe)? Usul: module dulu, component cadangan.
- **D-W3** — Paket: 1 file `.wasm`+manifest vs 2 berkas; **registri**: MVP = registri lokal (file system/manual install), pasca-MVP = marketplace (HTTP + trust-anchor Ed25519 agent7 sudah siap) (#663-B.3).
- **D-W4** — WCB masuk mono-repo `crates/wcb/` saat implementasi; apakah perlu repo terpisah utk SDK/template penulis plugin.
- **D-W5** — Roadmap bahasa SDK (#663-B.2): **v1 = Rust** (wasm32-unknown-unknown, minimal tooling) → **v2 = + AssemblyScript** (barier rendah) → **v3+ = Go** (butuh dukungan GC WASM stabil). Utk toolchain wasm32.
- **D-W6** (diputuskan #624) — Namespace registri plugin: `n8n://plugin/{id}/manifest` di registri agent7 (satu trust-anchor, bukan registri terpisah).
- **D-W7** (usulan #663-B.1) — Nilai `pool_size` default & policy warm/cold; usul: default 0 (cold), warm pool = keputusan per-plugin/perf (agent8 data).

---

## 12. Rujukan

#298 #362 #373 #375 #386 #428 #431 #455 #459 #467 #470 #471 #485 #488 #491 #492 #493 #498 #504 #577 #579 #602 #611 #612 #619 #621 #622 #624 #645 #651 #663 #664 · PRD-2-RUST.md (§3.2/§3.3/§4.2/§6.4/§7.1) · PRD-3 v3.1/v3.2 (matt) · AGENT5_QA_SECURITY_SPEC.md · SINTESIS-SPILLSTORE.md · AGENT4 handoff (#459).

— agent6 (ROLE_WASM)
