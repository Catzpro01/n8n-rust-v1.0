# ADR-v2: Content-Addressed WASM Module Cache — Zero-Sacrifice Cold-Start

- Penulis: agent6 (ROLE_WASM) — lane W1-WCB (siklus-1 sayembara, ARSIP SUKSES Wave-1)
- Tanggal: 2026-09-09 16:15 UTC
- Sayembara: ADR v2 "ZERO-SACRIFICE PERFORMANCE & HIGH-VALUE INNOVATION" (#1490 fern)
- Pilar: 1 (Performa ekstrem tanpa pengorbanan); §7 menautkan sinyal ke Pilar 2.
- Status: SUBMISSION (untuk evaluasi Lead Architect @matt + tim verifier)

---

## 1. Masalah

WCB (WASM Community Node Bridge, Wave-1) telah diterapkan: node komunitas n8n berjalan
sebagai guest WASM di host `wasmtime` di bawah engine pooling 32 MiB + fuel + gate MV-1..MV-7
(impor deny-by-default, egress deny, modul diidentifikasi oleh `module_sha256` = MV-3, memori
linear ≤ 512 halaman). Kontrak node sudah punya `ReplayRecord { seq, attempt }` (MV-7) sehingga
run yang sama dapat diulang secara deterministik.

Biaya yang belum pernah ditangani: **setiap eksekusi node meng-kompilasi ulang modul dari byte
mentah**, padahal `module_sha256` modul yang sama (identitas sudah diverifikasi MV-3) muncul
berulang di: run berurutan, retry, `ReplayRecord`, dan banyak node memakai modul komunitas yang
sama. Kompilasi Cranelift itu mahal dan mahalnya **tumbuh bersama ukuran modul**; instantiate
justru murah. Work yang diulang = `O(#run × compile(modul))`, padahal `compile` adalah fungsi
deterministik dari byte yang sama.

Pengukuran awal (harness: `/mnt/extra-storage/agent6-work/adr2-exp/`, wasmtime =48.0.1, engine
pooling 32 MiB identik dengan host WCB, median beberapa iterasi):

```
small (guest WCB e2e asli, 79 B wasm):
  compile (Module::new)  median 1.03 ms
  deserialize artefak    median 0.055 ms      -> 18.8x
big  (proksi sintetis jujur: 3000-fn, ~50 KB wasm, menyerupai modul node komunitas):
  compile (Module::new)  median 630 ms
  deserialize artefak    median 1.3 ms        -> 490x
  sanity: run() lewat artefak ter-deserialize = 11968 (modul berfungsi penuh)
```

Catatan kejujuran (Pelajaran 21/48a): angka "big" adalah **proksi sintetis** (di-generate),
bukan modul komunitas sungguhan; arah & orde besarnya yang menjadi klaim, dan akan diukur ulang
terhadap modul nyata saat implementasi.

## 2. Solusi

Cache artefak ter-kompilasi wasmtime yang **content-addressed dan terikat versi engine** di host
runtime WASM (lane agent6, pasca-merge; TIDAK menyentuh kernel/data-plane/rosetta):

1. **Key = `module_sha256`** — hash sudah dihitung & diverifikasi gate MV-3 pada ingest
   manifest; tidak ada key kedua, tidak ada hashing ulang, dan identitas byte terjamin
   (byte sama ⇒ perilaku sama ⇒ zero-regression secara konstruksi).
2. **Alur**:
   - *miss*: `Module::new` (compile) → `Engine::precompile_module` → tulis artefak ke
     `state/cache/wasm/<versi-engine>/<sha256>.bin` (mirror disiplin SHA256SUMS; chmod 0644).
   - *hit*: `Module::deserialize` artefak (lihat keamanan di §3). Tanpa compile.
   - **Fail-closed**: artefak korup / versi engine berubah → deserialize gagal → fallback
     `Module::new` + tulis ulang. Perilaku eksekusi TIDAK pernah berubah karena cache gagal.
3. **Terikat versi engine**: cache di bawah subpath `v48/…` (pin `=48.0.1`). Bump pin → miss
   alami; tidak ada artefak lintas versi yang bisa salah dipakai.
4. **Bounded (guard <500MB)**: LRU dengan cap total (usul default: 32 artefak ATAU 64 MiB,
   governor-settable). Artefak ≈ 18× ukuran wasm (terukur); cap membuat delta statis ≤ cap.
   Memori per-instance TIDAK berubah (pooling 32 MiB/512 halaman tetap).
5. **Tidak ada reuse instance antar-run**: instance dibuat segar per run (state bersih demi
   determinisme / `ReplayRecord`); yang dieliminasi hanya kompilasi, bukan instantiate.
   Kebijakan konstanta (cap, key) = milik konsumen (disiplin RULING 41/47b yang sudah berlaku).

## 3. Bukti "Zero-Regression"

- **Semantik identik**: cache mengubah hanya *asal* modul; artefak adalah hasil
  `precompile_module` atas byte yang sama yang akan di-*compile* (`Module::new`). Untuk
  wasmtime, deserialize modul dari artefak yang dibuat engine yang sama ≡ modul hasil compile —
  perilaku guest tidak bergantung pada jalur asal. Sanity run lewat artefak = 11968 (di atas).
- **Tidak ada fitur dipangkas / akurasi n8n tak berubah**: byte modul sama; gate MV-1..MV-7,
  fuel, memori 32 MiB, egress deny-by-default semuanya tetap di jalur eksekusi. Cache hanya
  menyediakan `Module`, bukan melewati gate.
- **Baseline saat ini tetap hijau**: nodes-wasm 31/31 + clippy 0 (R42, SHA256SUMS 7/7 pada
  /mnt/extra-storage/agent6-work/W1-WCB-IMPL). Saat implementasi cache, seluruh suite dijalankan
  ulang — wajib tetap hijau (bukti 3-baris R42).
- **Memori**: cap LRU (≤64 MiB) + pooling allocator (32 MiB/instance) ⇒ RSS proses tetap di
  bawah hard-cap 500 MB; delta statis terukur = ukuran cache ≤ cap.

## 4. Proyeksi Metrik Terukur

1. **p95 cold-start node** (jalur cache warm, modul khas puluhan-KB): `compile+instantiate`
   (~630 ms untuk proksi 50 KB) → `deserialize+instantiate` (~1.3 ms + instantiate). Reduksi
   **>95%** pada run kedua+ untuk modul yang sama. Untuk modul kecil (79 B): 1.03 ms → 0.055 ms
   (18×) meski absolut kecil.
2. **Throughput workflow** ber-node-wasm dengan pengulangan (loop/retry/ReplayRecord):
   membaik proporsional dengan `#run per modul unik` (work kompilasi dihapus untuk run 2..n).
3. **Memori**: delta statis ≤ cap LRU (usul 64 MiB) — angka konkret diukur saat implementasi;
   delta per-instance = 0 (pooling tidak diubah).
4. **Pengukuran pasca-implementasi** (R42): bench deterministik — modul tetap, engine pooling
   32 MiB, cold-vs-warm, 20 iterasi median — dilaporkan 3-baris + mutan; komitmen mengikuti
   standar verifikasi sha-baru (baca diff + jalankan penuh, RULING 49c).

## 5. Rencana Verifikasi Mutan

- **M1 — key salah**: ganti key `module_sha256` dengan panjang-byte → dua modul beda isi
  berukuran sama akan bertukar artefak → test identitas (perilaku guest benar per modul) GAGAL.
- **M2 — cache dimatikan** (selalu compile): bench warm-vs-cold harus tetap menunjukkan
  percepatan; tanpa cache, test regresi warm GAGAL.
- **M3 — cap LRU dibuang** (unbounded): test invariant "ukuran cache ≤ cap" GAGAL.
- **M4 — error deserialize ditelan salah** (return tanpa fallback compile): artefak korup →
  eksekusi harus tetap berhasil via fallback `Module::new`; test fail-closed GAGAL bila
  fallback dihapus.
- **M5 — path versi engine dihapus**: bump pin versi → test "miss alami (tidak pakai artefak
  versi beda)" GAGAL bila cache dipakai lintas versi.
- Setiap mutan disertai satu test yang menjadi merah — pola mutan yang sudah dipakai armada
  (LIN-3, G-2, R41 rev-3, WCB-F1).

## 6. Lingkup, kepemilikan, dependensi

- Lingkup: host runtime WASM nodes-wasm (lane agent6), diimplementasikan pasca-merge ke
  rust-engine (menunggu lokasi kanonik + approval dep wasmtime/wasmparser + RFC v0.4 `n8n_type`
  — bukan blocker ADR, ini proposal arsitektur).
- Nol dependensi baru: wasmtime (sudah di dev-dep; approval kanonik menyusul), sha2 =0.10.8
  (sudah di-pin). Cache = IO state + komponen kecil; tidak menyentuh tipe kontrak kernel.
- Kebijakan konstanta di konsumen (disiplin yang sama dengan RULING 41/47b `MAX_JSON_DEPTH`).

## 7. Sinyal Pilar 2 (parsial, tanpa klaim implementasi)

Contoh fitur bernilai tinggi yang fern sebut — *Time-Travel Execution Replay* dan *Differential
Tracing* — mengeksekusi modul yang sama berkali-kali; cache ini membuat replay/banding run
tersebut murah (work kompilasi dibayar sekali per modul unik). `ReplayRecord {seq,attempt}`
(MV-7, sudah ada di kontrak WCB) memberi dasar deterministik untuk replay. Ini sinyal sinergi,
bukan lingkup ADR ini.

---

Lampiran terukur (tempel mesin, R42):

```
small : wasm_bytes=79 artifact_bytes=13784 artifact/wasm=174.5x
small : compile median_ms=1.0310   deserialize median_ms=0.0550   speedup=18.75x
big   : wasm_bytes=49895 artifact_bytes=916240 artifact/wasm=18.4x
big   : compile median_ms=629.89   deserialize median_ms=1.28   speedup=490.57x
sanity: run() via deserialized artifact returned 11968 (0=OK fold)
```

Harness reproduksi: `/mnt/extra-storage/agent6-work/adr2-exp/` (terbaca grup 38a; Cargo.toml
pin wasmtime =48.0.1, wat =1.258.0). SHA256 berkas ini: lihat kanal posting submissi.
