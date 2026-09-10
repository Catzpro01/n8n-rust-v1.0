# AGENT6-WCB-SCRAPLING-ADAPTER — Rancangan Runtime Adapter WCB utk Node Scrapling (v0.3)

**Penulis:** agent6 (ROLE_WASM) · **Tanggal:** 2026-09-09 · **Status:** DESAIN v0.3 — serapan review agent9 (#878 FLAG seed-jitter). Catatan pembagian tugas: per dekrit #867/#872, eksekusi W2-NODE-SCRAPLING diserahkan ke Mitra D (@agent7+@agent9); dokumen ini tetap berlaku sbg fondasi WASM Bridge (Mitra C: @agent4+@agent6) & referensi node.
**v0.3 (revisi):** tambah klausa determinisme jitter (FLAG agent9 #878) + klarifikasi status kepemilikan per dekrit #867/#872.
**v0.2 (revisi):** serap review admission agent1 #777 — (B) egress deny-by-default + manifest schema-version pin & module sha256; (C/D) boundary WHAT(trait agent9)/HOW(substrat agent6), paritas fungsional SDK v1 Rust (daftar perilaku agent9), port Python = proposal terpisah; (E) temuan determinisme input_digest & no-refetch-on-replay; (F) lapisan HubCapsuleNode vs plugin WCB di PRD-Hub.
**Mandat:** fern #744 — "agent6: Rancang runtime adapter via WCB (prioritas Scrapling HTTP-stealth mode RAM <15MB agar aman dlm budget 500MB)." · fern #756 (klaim/adopsi ROLE_INTEGRATION) · NODES-CAMOFOX-SCRAPLING-SPEC v1.0 · ARSITEKTUR-SCRAPING-BERLAPIS (Lapis 1).

---


## 0. Revisi v0.2 — serapan review agent1 (#777, admission/engine)

| Poin agent1 | Serapan ke desain |
|---|---|
| (B) Egress deny-by-default; manifest wajib schema-version-pin + module sha256 | §2/§5: allowlist HOST kosong = **nol egress** (semangat ANC-5); manifest = `{schema_version, module_sha256, imports, host_allowlist, rules, output_strategy}` |
| (C) Q1: trait di kontrak agent9 (WHAT), implementasi di substrat agent6 (HOW) | Boundary §1 dipertegas: agent9 definisikan kontrak/node, agent6 impl substrat — tanpa sengketa kepemilikan |
| (D) Q2: paritas fungsional SDK-v1 Rust didukung (daftar perilaku dari agent9); port Python = proposal terpisah | Roadmap §5: v1 = perilaku scrapling terdaftar (smartExtract/cssQuery/xpathQuery/batchScrape) diimplementasi native Rust; **bukan** port runtime Python ke WASM |
| (E) Determinisme: output adapter masuk engine sebagai **Item** dgn `input_digest` ter-pin di ingress; **dilarang fetch-ulang saat replay**; RecordSet W2-DETERM merekam respons | §2/§5: pada replay, `wcb:net/request` membaca **rekaman** (RecordSet), bukan fetch nyata; digest pin di ingress (pembedaan C-02) |
| **(v0.3) Determinisme jitter (FLAG agent9 #878):** seluruh jitter/backoff pada retry WAJIB deterministik — jitter = f(input_digest, attempt_index), TIDAK boleh unseeded RNG; waktu tidur wall-clock (sleep) boleh berbeda antar-run tapi TIDAK boleh memasuki konten Item/digest; Retry-After dipakai sbg batas atas, nilainya tidak direkam ke Item. Menjamin replay RecordSet menghasilkan digest & Item identik (#777-E; selaras W2-DETERM-ENFORCE agent4) |
| (F) PRD-Hub: plugin WCB-guest (lapisan ekstraksi dalam 1-node) vs HubCapsuleNode (orkestrasi sub-workflow) = lapisan berbeda | Catatan §2: tidak konflik; dicatat utk manifest Fase-1 Hub |

## 1. Posisi & boundary (hindari tumpang-tindih)

| Pihak | Pemilik | Batas |
|---|---|---|
| Node n8n-nodes-scrapling (skema/OpenAPI, kontrak node) | agent9 (#744: schema+OpenAPI mapping) | node-level contract, parameter ops, fixture |
| Runtime adapter eksekusi via WCB | **agent6 (dokumen ini)** | cara NODE dijalankan: substrate, memori, gate, egress |
| Katalog Hub & template | agent4 (#744) | registrasi intel-type scrapling, template scraping |
| Guardrail kepatuhan | agent10 (#744) | rate-limit 2–5 s, robots.txt, isolasi PII |

Prinsip Kunci-7 (#573/#575): WCB = **substrate eksekusi/extensibility**; node tetap node (agent9/agent4 mendefinisikan kontrak), WCB tidak menggantikan connector native. Dokumen ini hanya lapisan **bagaimana** node Scrapling dieksekusi dengan aman & hemat di engine Rust.

## 2. Arsitektur adapter (dua lapisan, satu alur)

```
[Node Scrapling di workflow (kontrak agent9)]
        │  invocation via API-eksekusi (bukan MCP — garis F-8/MCP-11)
        ▼
┌──────────────────────────  HOST ADAPTER  (native, agent6 substrate) ──┐
│ • Admission 2-gate (#577/#582): API agent1 → sandbox agent6           │
│ • Manifest {sha256 modul, batas, imports, host-allowlist, rules}      │
│ • Network egress HOST-side: TLS JA3/JA4 stealth (curl_cffi-equivalent)│
│   dipanggil VIA host function wcb:net/request (SEC-WCB-03) — GUEST    │
│   TIDAK punya socket (no WASI, spec §4)                               │
│ • robots.txt check + polite delay 2–5 s + backoff (guardrail agent10) │
│ • Fuel/timeout/32MiB enforcement; PRG meter (#493)                    │
└───────────────────────────────▲───────────────────────────────────────┘
                                │ wcb:* host calls (allowlist per manifest)
        ┌───────────────────────┴───────────────────────────────────────┐
        │  GUEST PLUGIN (wasm32, WASM via wcb crate, ≤32 MiB)           │
        │  • orkestrasi request: urutan URL, Retry-After 429, backoff    │
        │  • self-healing extractor: rules adaptif (semantik tag+bobot)  │
        │  • output kanonik ItemList → host-streaming read_at/range      │
        └────────────────────────────────────────────────────────────────┘
```

**Keputusan arsitektur:**
1. **HTTP-stealth = HOST-side**, bukan guest. TLS/JA3 fingerprint masquerade butuh socket + kripto native; guest WASM tanpa WASI tidak boleh menyentuh jaringan. Guest hanya melihat *hasil request* (payload) lewat host function `wcb:net/request` yang di-allowlist per manifest (SEC-WCB-03, #492/#470).
2. **Self-healing/ekstraksi = GUEST WASM plugin**: logika adaptif yang paling sering berubah (rules, auto-match saat HTML berubah) justru yang paling aman di-sandbox — crash parser → evict instance, engine tetap hidup (nilai WCB #375). Update plugin tanpa restart core (manifest/versi pin).
3. **RAM <15 MB mode HTTP-stealth** tercapai karena: tanpa browser worker, tanpa render; guest ≤ 32 MiB *linear memory* tapi instance nyata ber-commit sesuai kebutuhan (host-streaming, bukan materialisasi penuh). Plugin kecil (~KB–hundreds KB). Pengukuran RSS instance scrapling dimasukkan ke BENCH-REPRO agent8 (lihat §4).

## 3. Peta gate keamanan (SEC-WCB-01..04) pada adapter

| Gate | Di mana diuji | Bukti |
|---|---|---|
| SEC-WCB-01 (import allowlist, no WASI) | validate/install (2-gate lapis-1) | `wcb-spike` gate 01a–01d PASS |
| SEC-WCB-02 (≤32 MiB, pooling) | engine config + runtime | `wcb-spike` gate 02a/02b PASS (cap 512 pg; min-2GiB ditolak) |
| SEC-WCB-03 (egress allowlist) | host fn `wcb:net/request` + manifest host-allowlist | *belum* — butuh fixture manifest+host fn lanjutan |
| SEC-WCB-04 (manifest_sig Ed25519 terpadu) | VALIDATE + first-load re-verify (TOCTOU dua-tahap #621) | *belum* — kontrak agent7 #611/#624 + agent10 #602/#612 |

## 4. Budget memori (estimasi — ukur agent8)

| Komponen | RSS estimasi | Catatan |
|---|---|---|
| Engine WCB (shared, pooling) | ~17–28 MiB (dev, terukur spike) | baseline engine; 1 utk semua node |
| Guest plugin scrapling instance | ≤ 32 MiB linear; committed sesuai muatan | RSS nyata menunggu pengukuran BENCH-REPRO |
| Host adapter (fetch buffer, rules) | buffer stream host-side, tidak materialisasi | host-streaming #467/#680 |
| **Klaim target** | **HTTP-stealth < 15 MB di atas baseline engine** | perlu verifikasi agent8 (release, disiplin #474) |

Catatan jujur: <15 MB adalah **target & kriteria terukur** — bukan klaim lulus. Diverifikasi agent8 + agent5 sebelum tanda tangan L2a/L2 scraping.

## 5. Dependensi & urutan eksekusi

1. (DONE) Kontrak node Scrapling → agent9 (NODES-CAMOFOX-SCRAPLING-SPEC v1.0).
2. (DONE) Spike L2a wasmtime 32MiB → `AGENT6-WCB-SPIKE-REPORT.md` (7/7 PASS, dev).
3. **IN PROGRESS (dokumen ini)**: desain adapter → review agent3 (design security), agent10 (guardrail), agent9 (kontrak node), agent1 (engine admission).
4. Setelah desain disetujui: integrasi crate `wcb/` (wasmtime + host fn `wcb:net/request` dgn impersonation di sisi host) + fixture plugin scrapling (extraction rules) — task implementasi (usulan `W1-WCB-IMPL`/Wave-2 lanjutan).
5. (Dari #777-E) Replay-safe: egress direkam di RecordSet; replay = baca rekaman (bukan fetch) — koordinasi W2-DETERM (agent10 spec siap). Diterapkan saat integrasi crate.
6. BENCH-REPRO RSS + determinisme → agent8 → tutup gate L2a & target <15 MB.

## 6. Pertanyaan untuk disetujui (agar tidak menabrak kepemilikan)

- Q1: HTTP-stealth host-side via crate `impersonate`/curl_cffi binding = ranah adapter agent6, atau connector native agent9? (usul: host lib adapter milik agent6 substrate; node memanggil via trait — konsisten Kunci-8).
- Q2: rules self-healing versi awal cukup sebagai plugin WASM SDK-v1 (Rust→wasm32), bukan Python scrapling asli? (paritas fungsional scrapling ≠ menyalin Python runtime; konfirmasi scope paritas ke matt/agent9).
- Q3: task implementasi node penuh (dgn gateway) masuk Wave-2 task ini atau Wave-3? (task queue saat ini: W2-NODE-SCRAPLING IN_PROGRESS agent6).

*Dokumen desain agent6 utk W2-NODE-SCRAPLING — nol kode kernel; spike kode sudah terpisah (wcb-spike, L2a).*
