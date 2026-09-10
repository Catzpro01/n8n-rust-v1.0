# AGENT6 — W2-NODE-SCRAPLING · Slice Integrasi #1 (Bukti Konsep Host-Guest)

- **Tanggal**: 2026-09-09 (sprint W2)
- **Status**: 7/7 PASS (exit=0) — artefak eksperimen di `/home/agent6/w2-slice/` (VPS) dan salinan lokal `/home/user/w2-slice/`.
- **Terkait**: desain `AGENT6-WCB-SCRAPLING-ADAPTER.md` v0.2 (sha `24247788…`), #781, #784 (anti-ban 5-lapis / stabilitas feed).

## 1. Tujuan slice

Membuktikan kontrak integrasi WCB-Scrapling paling kritis **sebelum** menyambung ke trait node (kontrak agent9 / WHAT):
eksekusi ekstraktor **di guest WASM (tanpa socket)** sementara **semua egress via host-fn `wcb:net/request` deny-by-default**,
dengan manifest gate, retry 429, dan replay RecordSet — yaitu lapis-lapis inti dari desain v0.2 & #777.

## 2. Arsitektur bukti

```
guest (no_std, wasm32-unknown-unknown, 0 import selain env::wcb_net_request)
   ├─ alloc / run(html,mode,sel) -> Item JSON / try_fetch(req,out)
   └─ TIDAK punya akses jaringan: satu-satunya jalan keluar = host-fn
host (wasmtime 48.0.1, rust 1.98.1, personal toolchain di /mnt/extra-storage/agent6-rustup)
   ├─ gate manifest: schema_version + module_sha256 (dicek saat load)
   ├─ wcb:net/request: allowlist (kosong => EGRESS_DENIED, nol network)
   ├─ fetch host-side + retry 429/Retry-After + backoff/jitter 1–3s
   ├─ RecordSet W2-DETERM: simpan payload; replay membaca RecordSet (bukan fetch-ulang)
   └─ ekstraksi deterministik -> Item (input_digest di-pin di ingress)
```

## 3. Hasil (7/7 PASS)

| Gate | Isi | Hasil |
|---|---|---|
| G1 | manifest `schema_version=0.1.0` + `module_sha256` (host sha == python sha) | PASS |
| G2 | egress deny-by-default: allowlist `[]` → `EGRESS_DENIED`, tidak ada koneksi | PASS |
| G3 | fetch host-side allowlist + ekstraksi smart guest → title, word_count=40, links_total=4, `input_digest=99f26945…` | PASS |
| G4 | retry 429: `backoff 1038ms (Retry-After=1s +jitter)`, attempt 2 → 200 | PASS |
| G5 | replay RecordSet dengan allowlist KOSONG → digest & Item identik (tanpa net) | PASS |
| G6 | granularity smart/css/xpath ter-plumbing: `smart_wc=40 css_h2_hits=1 xpath_h2_hits=1` | PASS |
| G7 | konkuensi host 2 → wall 1201ms < 0.85×1800ms (paralel efektif) | PASS |

Catatan verifikasi: G1 membuktikan implementasi sha256 host menyamai `sha256sum` Python untuk bytes modul
(cek silang vektor; `input_digest` stabil antara live & replay → determinisme #777-E terpenuhi di slice ini).

## 4. Cara menjalankan ulang (VPS)

```bash
. /home/agent6/.w2env
cd /home/agent6/w2-slice/guest && rustup run stable-x86_64-unknown-linux-gnu cargo build --release --target wasm32-unknown-unknown
cd /home/agent6/w2-slice/host && rustup run stable-x86_64-unknown-linux-gnu cargo build
cd /home/agent6/w2-slice && /mnt/extra-storage/agent6-w2-target/debug/wcb-slice ./guest.wasm ./manifest.json
```

## 5. Batasan slice (jujur) & langkah berikutnya

- Semantik `cssQuery`/`xpathQuery` masih *placeholder* (tag/class hit + cuplikan) — **bukan** paritas SDK-v1; mesin ekstraktor penuh mengikuti keputusan paritas (E) #777 dan kontrak agent9.
- HTTP plain ke server uji lokal (`127.0.0.1`); **TLS/stealth host-side (JA3/JA4)** dan batas modul `<15 MB`/fuel diukur pada eksperimen berikutnya.
- RecordSet saat ini file di `/tmp/w2-record` (persistence path produksi & integrasi storage = boundary agent10/agent4, dijaga).
- Tidak menyentuh repo bersama (zero-code #723); slice berjalan di area kerja sendiri `/home/agent6/w2-slice/`.

Berikutnya (urutan): (1) host-fn `wcb:net/request` versi TLS + allowlist berbasis domain, (2) pengukuran RSS/fuel & ukuran modul terhadap batas desain, (3) trait node sesuai kontrak agent9 (WHAT) — impl di sisi agent6 (HOW), (4) sambungan granularity `batchScrape` (konkuensi 1–5) pada RecordSet.
