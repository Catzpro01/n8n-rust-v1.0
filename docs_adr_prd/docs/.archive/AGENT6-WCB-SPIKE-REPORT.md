# AGENT6-WCB-SPIKE-REPORT — Verifikasi L2a: Sandboxing WASM (SEC-WCB-01..04, 32 MiB)

**Penulis:** agent6 (ROLE_WASM) · **Tanggal:** 2026-09-09 08:10 UTC · **Status:** SPIKE PASS 7/7 (dev profile)
**Konteks:** pembukaan zero-code #723 (PRD-3 disahkan), K-8 inovasi WCB = mandat, PRD-3-CANONICAL-APPROVED §5 L2a. Spike ini menjawab klausul **"WASM instance ≤32 MB linear memory — belum terverifikasi (butuh agent6/agent8)"** (PRD-3 v3.2 §3.2/§5 L2a, sha e51bcc99).
**Kode:** crate `wcb-spike/` di VPS `/home/agent6/wcb-spike` (di luar kernel, sementara; integrasi task resmi menyusul). Rekaman run: `/home/agent6/wcb-spike/spike-run.log`.

---

## 1. Konfigurasi engine (persis desain AGENT6-WCB-SPEC §3/#431)

| Parameter | Nilai | Arti |
|---|---|---|
| Runtime | **wasmtime 48.0.1** (cranelift, dev) | versi spec; dep minimal: runtime+cranelift+pooling-allocator+wat |
| Allocator | **Pooling** (`InstanceAllocationStrategy::Pooling`) | Engine+modul shared, slot reuse |
| Cap memori | `max_memory_size = 32 MiB` (= 512 × 64 KiB pages) | batas keras #431 per instance |
| Pool | 16 core instances / 16 memories | kapasitas slot |
| Fuel | `consume_fuel(true)` + `set_fuel` per store | meter E-WCB-FUEL |

## 2. Hasil gate (7/7 PASS)

| Gate | Uji | Hasil |
|---|---|---|
| SEC-WCB-01a | import `wasi_snapshot_preview1::fd_write` | **DITOLAK** (di luar wcb:*) |
| SEC-WCB-01b | import `env::random_get` | **DITOLAK** |
| SEC-WCB-01c | import `wcb:net/request` + `wcb:entropy/seed` | **DITERIMA** |
| SEC-WCB-01d | modul tanpa import | **LOLOS** (0 import) |
| SEC-WCB-02a | `memory.grow(511)` (→512 pg) sukses; `memory.grow(600)` | grow sukses (size=512), beyond cap → **-1 (ditolak)** |
| SEC-WCB-02b | modul `memory 32768` (min 2 GiB) | **GAGAL instantiate** ("does not fit in pooling allocator requirements") |
| E-WCB-FUEL | loop tak hingga, budget fuel 30.000 | **trap out-of-fuel** teramati (mapping #664/#684) |

## 3. BENCH-REPRO (dev profile; angka final = agent8 dgn release)

```
engine        : wasmtime 48.0.1 | pooling | max_memory_size=32 MiB | pool=16
profile       : dev (opt-level default), 2 vCPU
host-ram      : 5.8 GiB total / 5.2 GiB available
rss-baseline  :   6.6 MiB   (proses start)
rss-engine    :  17.3 MiB   (engine pooling dibuat, 0 instance)
rss-compile   :  28.0 MiB   (+ modul touch dikompilasi sekali)
rss-after-run :  28.2 MiB   (3× cold + 8× warm full-32MiB-touch, instance di-drop)
peak-rss HWM  :  60.1 MiB
vsz-reservation: 64.7 GiB  (reservasi virtual pool — BUKAN RSS ter-commit)
cold-instance : 50.0 ms avg (engine+compile+instantiate+touch 32 MiB), n=3
warm-instance : 33.0 ms avg (modul reuse: instantiate+touch 32 MiB), n=8
```

**Pembacaan:** slot pool di-reserve virtual (VSZ besar), RSS hanya naik saat halaman guest di-commit/di-touch → RSS per instance live ≤ 32 MiB halaman ter-commit (sesuai desain host-streaming spill #467/#680). Engine dasar ~17–28 MiB RSS, jauh di bawah baseline 150 MiB & budget 500 MB.

## 4. Makna utk L2a (jujur, tidak over-claim)

- **Terverifikasi pada level mekanisme**: wasmtime 48 + pooling menegakkan cap 32 MiB & tolak modul min-memori besar; SEC-WCB-01 allowlist berjalan sebagai audit statis; fuel metering memetakan E-WCB-FUEL.
- **BELUM selesai sebagai GATE**: (1) run ini dev-profile → angka release & repeatabilitas multi-run harus dikonfirmasi **agent8** (ROLE_PERF, VACANT) dgn disiplin BENCH (#474); (2) verifikasi SEC-WCB-03 (egress allowlist `wcb:net/request`) & SEC-WCB-04 (manifest_sig Ed25519) butuh fixture manifest + host-function lanjutan — di luar cakupan spike murni ini.
- Usulan kepada pemilik: tandai L2a **"SPIKE-PASS (agent6), menunggu angka release agent8"** — bukan CLOSED penuh.

## 5. Akses artefak

- Kode: VPS `/home/agent6/wcb-spike` (Cargo.toml + src/main.rs), log run `spike-run.log`.
- Crate dependency: wasmtime =48.0.1, wasmparser =0.258.0, wat =1.258.0; CARGO_TARGET_DIR pribadi agent6 (tidak mengganggu build agen lain).
- Reviewer design-keamanan: agent3 (#683/#762); gate approver: agent5/agent10/agent8 (recommender ≠ approver #488).

*Ditulis oleh agent6 (ROLE_WASM) — bukti eksekusi L2a, pasca #723/#726.*
