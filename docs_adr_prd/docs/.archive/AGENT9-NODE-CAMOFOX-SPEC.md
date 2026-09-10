# AGENT9-NODE-CAMOFOX-SPEC — Desain Node `n8n-nodes-camofox-browser` (Lapis 1)

**Penulis:** agent9 (ROLE_INTEGRATION) · **Tanggal:** 2026-09-09 · **Versi:** v0.1 (DESIGN — freeze #386 dipatuhi, 0 kode)
**Mandat:** fern #744 (Mandat Pemilik Proyek: integrasi camofox-browser & scrapling) · ARSITEKTUR-SCRAPING-BERLAPIS.md §1 Lapis 1 ("Pelaksana: @agent9") · NODES-CAMOFOX-SCRAPLING-SPEC.md §2 · queue W2-NODE-CAMOFOX (P1, ROLE_INTEGRATION).
**Posisi:** node komunitas → jalur 5 katalog (INTEGRATION-SPEC §3.3); masuk Lapis 1 (proteksi ketat); pasangan `scrapling` (scrapling = HTTP stealth ringan; camofox = full anti-detect browser).

---

## 1. Model parameter (padanan `INodeProperties` → `ParameterSchema`; kompilasi via arah-A agent4)

Gaya n8n community node: `resource: browser` + `operation`. Semua field boleh expression `{{...}}` (eval per-item, prinsip F-C4).

| # | Field (name) | Tipe IR | Wajib | Default | Catatan |
|---|---|---|---|---|---|
| 1 | `resource` | `OptionsField` | ✅ | `browser` | fixed satu resource v1 |
| 2 | `operation` | `OptionsField` | ✅ | `navigate` | `navigate` \| `extractContent` \| `click` \| `screenshot` \| `evaluate` (NODES-SPEC §2.1) |
| 3 | `url` (navigate) | `StringField` | ✅ (navigate) | — | validasi URL eksak; **SSRF-guard §4 sebelum fetch** |
| 4 | `timeoutMs` | `NumberField` | ⬜ | `30000` | min 1000, max 30000 (hard-cap worker §2) |
| 5 | `waitUntil` | `OptionsField` | ⬜ | `domcontentloaded` | `domcontentloaded` \| `networkidle` |
| 6 | `selector` (extractContent/click) | `StringField` | ✅ (op tsb) | — | |
| 7 | `outputFormat` (extractContent) | `OptionsField` | ⬜ | `markdown` | `markdown` \| `html` \| `text` |
| 8 | `delayAfterMs` (click) | `NumberField` | ⬜ | `2500` | jendela 2000–5000 (polite delay guardrail §4) |
| 9 | `fullPage` (screenshot) | `BooleanField` | ⬜ | `false` | |
| 10 | `quality` (screenshot) | `NumberField` | ⬜ | `80` | 1–100 |
| 11 | `script` (evaluate) | `StringField` (codeEditor) | ✅ (evaluate) | — | **sandboxed** (WCB/QuickJS isolate — BUKAN eval bebas); dilarang akses jaringan/FS dari script (egress tetap via host §5) |
| 12 | `options.ignoreRobots` | `BooleanField` | ⬜ | `false` | override robots.txt — dicatat di audit log + default false (guardrail NODES-SPEC §5.2) |

## 2. ResourceHint & memory (Hard-Cap 500MB — NODES-SPEC §2.2)

| Operasi | SideEffect | Alasan (pola tabel agent4 §9: default aman) |
|---|---|---|
| `navigate` | `Idempotent` | GET semantics |
| `extractContent` / `screenshot` | `Idempotent` | read-only |
| `click` | **`NonIdempotent`** | klik bisa submit/mengubah state server (POST-like) |
| `evaluate` | **`NonIdempotent`** | script bisa apa pun — default aman |

- `buffering`: `Batch` (output teks); screenshot binary → `Streaming` (binary path PRD-1 §8.2, DEFER sampai data-plane binary).
- `weight`: **berat** — worker browser penuh.
- `max_concurrency`: **1** (NODES-SPEC: maks 1 instans aktif; antrean admission agent1 §5.2).
- Worker = **ephemeral CDP**: spawn → operasi → auto-terminate (≤30s); **cgroup 150MB/worker**; kill OOM = `FailedRetry` (bukan crash engine); RAM transien dibebankan ke budget governor, 0 persisten (F-7).

## 3. Output kanon (→ Token Condenser, NODES-SPEC §4)

```
{ url, status, content_format: markdown|html|text|png,
  content_ref,          // payload via data-plane (payload_ref), BUKAN blob JSON raksasa
  pii_redacted: true,   // sanitasi PII pra-log (§4)
  took_ms }
```
Output = **TAINTED-EXTERNAL** (invarian-taint agent1 §3) → wajib lewat sanitasi agent5 sebelum kondenser/AI Agent.

## 4. Guardrails (mandat agent10 ARSITEKTUR §4 + NODES-SPEC §5 — di-enforce node, bukan imbauan)

1. **SSRF blocklist wajib pra-fetch**: tolak `127.0.0.0/8, ::1, 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 169.254.0.0/16, metadata cloud (169.254.169.254)` + resolusi DNS dicegah redirect ke IP privat. Pelanggaran = error `E-SSRF-BLOCKED` (fail-closed).
2. **Polite delay**: antar-request ke domain sama min 2.0–5.0s (randomized); `delayAfterMs` dijendelai.
3. **robots.txt**: dicek otomatis; `ignoreRobots=true` eksplisit → dicatat audit (siapa/kapan/mengapa) — tidak pernah senyap.
4. **PII redaction**: email/tel/token disaring dari output & log eksekusi (modul sanitasi agent5).
5. **Lisensi**: template Hub yang memakai node ini wajib atribusi (H-03 agent10).

## 5. Batas implementasi & interface lintas-agent

- **Runtime browser = agent6 (WCB)**: node ini TIDAK menjalankan browser di-process; ia membangun request deklaratif → host menjalankan worker CDP ephemerall (klaim-A agent6 #…, konsisten W3-INGRESS §1). Interface slot: `wcb:browser/*` (spawn/navigate/evaluate/terminate) — detail kontrak = agent6.
- **Skema & kompilasi = agent4**: deklarasi §1 dikompilasi via arah-A (`8n-schema-compiler`); registry `(kind=camerafox… kind="n8n-nodes-camofox-browser", typeVersion=1)`.
- **Etika anti-detect — KETEGANGAN JUJUR yang wajib diputuskan**: aturan katalog kita "UA jujur teridentifikasi" (D2 §5.3) vs fungsi inti camofox = fingerprint masking. Posisi dokumen ini: camofox dibatasi **hanya untuk sumber yang ToS-nya mengizinkan akses programatik** tapi memakai proteksi bot agresif (mis. feed resmi di belakang Cloudflare); DILARANG untuk menembus paywall/login tanpa izin. **Keputusan akhir = agent5 (etika) + agent10 (compliance)** — diajukan sebagai SCRAP-E1 (lihat §6). Sebelum diputuskan, node tidak masuk katalog live (review_status ≠ live).

## 6. Gate verifikasi (falsifiable, post-freeze; diajukan ke kerangka agent5)

| Gate | Kriteria PASS | Reproduksi |
|---|---|---|
| SCRP-CF-1 | SSRF corpus 20 URL (privat/localhost/metadata/DNS-rebind) → 20/20 ditolak E-SSRF-BLOCKED; 0 request keluar | harness uji unit + capture socket |
| SCRP-CF-2 | worker auto-terminate ≤30s pada 20 run; peak RSS worker ≤150MB (cgroup) | uji beban + baca cgroup memory.peak |
| SCRP-CF-3 | polite delay: 3 request berurutan domain sama → gap ≥2.0s & ≤5.0s | uji waktu + mock server |
| SCRP-CF-4 | robots.txt disallow → request TIDAK dikirim; ignoreRobots → terkirim + entri audit | mock robots |
| SCRP-CF-5 | output 10 halaman uji → PII (email/tel/token tanaman) 0 di output & log | scan check-secrets |
| SCRP-CF-6 | evaluate script coba fetch()/FS → ditolak sandbox (0 egress selain host-allowlist) | uji isolasi WCB |

## 7. Terbuka

- **SCRAP-E1** (BLOCKING-ethics): kebijakan penggunaan anti-detect (agent5+agent10) — lihat §5.
- Binary screenshot path: menunggu dukungan binary data-plane (DEFER, dicatat).
- `verify_status` node di katalog Hub: `verified-tofu` sampai golden run + pin (SEC-HUB-01 analog).
