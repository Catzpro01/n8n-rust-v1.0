# AGENT7 — KONTRAK L4-FALLBACK: Eskalasi Hydration Dinamis untuk W2-NODE-SCRAPLING
**Versi:** 0.1 | **Tanggal:** 2026-09-09 | **Status:** DESAIN (0 kode) — peran agent7 sesuai ruling matt #903 (Opsi A, pemisahan peran ketat)
**Penulis:** agent7 (ROLE_AI_MCP) · **Mandat:** matt #903 §2 (L4 FALLBACK: integrasi fallback Puppeteer MCP untuk hidrasi dinamis) · ARSITEKTUR-SCRAPING-BERLAPIS §3 · AGENT9-SCRAPE-L1-DESIGN §5 (L1 tidak mengevaluasi skrip; kebutuhan `__NEXT_DATA__` = langsung L4) · AGENT7-W2-SCRAPE-L4-SPEC v0.2 (b1c147fa) · AGENT9-NODE-SCRAPLING-SPEC (4db7cdd2)

---

## 1. Posisi kontrak ini
- Agent3 = implementor W2-NODE-SCRAPLING (ruling #903). Dokumen ini = **kontrak antarmuka eskalasi** yang agent3 rujuk saat menyambung hook `mode: browserFallback` → operasi L4; agent9 = reviewer gerbang (SCRP-SL-1..6).
- **Tidak menyentuh file implementasi agent3**; hanya definisi kontrak + fixture kontrak (siap di qa/l4-hydration).
- Eskalasi yang dimaksud: **Jalur-A→Jalur-B** pada L4 (§2 spec L4) dan **L1 httpStealth → L4** pada scrapling (NODES-SPEC §3 `browserFallback`).

## 2. Pemicu eskalasi (deterministik, bukan heuristik kabur)
`mode=httpStealth` (default L1) mengeksekusi `smartExtract/cssQuery/xpathQuery/batchScrape`. Eskalasi ke L4 hanya bila **salah satu** kondisi terpenuhi (urutan cek):
1. `L4_HYDRATION_NOT_FOUND`-setara: payload HTML awal tak memuat `__NEXT_DATA__`/`__NUXT__` ATAU
2. `L4_SELECTOR_MISSING`: selector path absen di payload (layout berubah — indikasi data di-fetch klien/hydration dinamis) ATAU
3. hasil L1 `empty` dan `autoHeal=true` + `mode=browserFallback` eksplisit (default `false` — eskalasi tak pernah otomatis tanpa izin pengguna, kecuali aturan Hub template).

**Determinisme:** keputusan eskalasi = fungsi `(html_digest, keys_ada, selector_path, konfigurasi)` — direkam di metadata Item (`item.meta.escalated: L1->L4`, reason ∈ {no_hydration, missing_selector, empty_autoheal}); saat replay RecordSet, keputusan diulang dari rekaman (dilarang fetch ulang — #777-E / W2-DETERM-ENFORCE).

## 3. Kontrak pemanggilan (API-eksekusi node, BUKAN MCP — F-8/MCP-11)
```
L4::extractHydration {
  url,                      // sama dgn URL L1 yang gagal (tanpa rewrite)
  keys: ["__NEXT_DATA__","__NUXT__"],
  selector_path,            // opsional; mapping dari rules L1 bila tersedia
  max_bytes: 512000,
  timeout_ms: 30000,        // batas jalur-B (CDP) lebih mahal
}
-> HydrationItem { source: script_tag|live_global, key, payload_json,
                   selector_path, partial_reason, truncated, byte_len }
```
- Error L4 (taxonomy spec L4 §3.1) dipetakan ke output node scrapling: `L4_HYDRATION_NOT_FOUND`/`L4_SELECTOR_MISSING` → item `{url, status:'failed', error_code, escalated:true}` (fail-open-informatif, konsisten AGENT9-SPEC §3); `L4_NAV_BLOCKED`/`L4_EVAL_DENIED` → failed tanpa retry (agent10 #826-a).
- Hasil L4 **tidak** melewati self-healing parser L1 (data sudah terstruktur) — langsung ke Item dengan `input_digest` pin ingress.

## 4. Pemetaan operasi L1 → L4 (hanya yang bermakna)
| Operasi L1 gagal | Alternatif L4 | Catatan |
|---|---|---|
| `cssQuery(sel)` di halaman SPA | `extractHydration` + selector_path (bila data ada di props) | paling hemat; tanpa browser penuh |
| `smartExtract(rules)` empty | `evaluate` CDP utk state pasca-hydration (Redux/Vue) | butuh biner Chromium (G-L4-9) |
| `xpathQuery` di konten virtual | `evaluate` + query DOM post-hydration | butuh biner Chromium |

## 5. Guardrail eskalasi (tetap berlaku penuh — spec L4 §7)
SSRF sama (tolak saat authoring) · polite delay antar-domain 2–5 s · robots.txt (eskalasi = request baru → cek ulang) · PII redaksi sebelum log · token CDP acak per sesi tak pernah di log · atribusi lisensi template.

## 6. Fixture uji kontrak (tersedia di qa/l4-hydration, 16/16 gates)
8 korpus HTML (Next 12/13/14, charset-varian, NUXT objek, tanpa-hydrasi, malformed, huge) — subclass untuk uji eskalasi:
- f5_plain (tanpa hydrasi) → eskalasi reason=no_hydration
- selector absen → reason=missing_selector
- payload `empty` legit → TIDAK eskalasi (data nol sah — kontrak §3.1 L4)

## 7. Gate lintas (usul utk review agent9/agent3, falsifiable)
| Gate | Kriteria | Metode |
|---|---|---|
| G-ESC-1 | Keputusan eskalasi deterministik: 100 run → hash keputusan identik | run berulang fixture |
| G-ESC-2 | Eskalasi tanpa `browserFallback=true` TIDAK pernah terjadi (0 kasus di 50 fixture) | uji negatif |
| G-ESC-3 | Metadata `item.meta.escalated` + reason selalu terisi saat eskalasi (0 missing) | uji integrasi |
| G-ESC-4 | Replay: 100% keputusan dari RecordSet, 0 fetch ulang | saat integrasi W2-DETERM |

## 8. Dependensi & status
- Kontrak WHAT scrapling: AGENT9-SPEC (4db7cdd2) — agent3 implementor.
- `mode=browserFallback` field: sudah ada di AGENT9-SPEC §1 (field 9) — dokumen ini hanya mengisi definisi perilaku eskalasinya.
- Jalur-B (evaluate live): menunggu keputusan biner Chromium (G-L4-9) — **bukan blocker** kontrak ini (fixture jalur-A sudah tervalidasi).
- Review: agent9 (gate SCRP-SL), agent3 (hook), agent10 (guardrail §5).
