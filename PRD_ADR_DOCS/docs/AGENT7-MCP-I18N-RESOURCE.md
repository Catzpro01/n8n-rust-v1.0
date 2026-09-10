# AGENT7-MCP-I18N-RESOURCE — Desain MCP Resource Deskripsi Multibahasa (mandat #547)

**Penulis:** agent7 (ROLE_AI_MCP) · **Tanggal:** 2026-09-09 · **Status:** DRAFT v0.2
**Riwayat:** v0.1 (06:55) → **v0.2**: O-I3 terjawab — komponen "on-demand translation" diklaim agent6 sebagai WCB-TRANS (#574); catatan konvergensi ditambahkan
**Mandat:** Pemilik proyek via fern #547/#548/#549 — "UI Self-Documenting & Translasi 40 Bahasa"
**Bidang agent7 (#547):** *"Ekspos deskripsi multi-bahasa via MCP Resource `n8n://templates/{id}?lang={code}` agar AI Agent di negara mana pun langsung paham."*
**Patuh freeze #386:** dokumen — nol kode.
**Owner konten:** agent4 (skema documentation{}, manifest) · agent2 (tabel `workflow_i18n` + bundle <500KB + FTS5) · agent7 (desain/permukaan MCP resource).

---

## 1. Konteks & sumber kebenaran (dari #547 + manifest agent4 v0.3)

- Setiap workflow/template punya **3 pilar deskripsi**: `[CARA KERJA]`, `[FUNGSI]`, `[TUJUAN]` + **node notes** (evolusi stickyNote — muncul di 46% template, temuan matt).
- **Manifest TETAP satu bahasa sumber** (`source_lang`, lean — agent4 v0.3). Terjemahan = **derived data** di tabel `workflow_i18n(workflow_id, node_id, lang_code, field, translation)` milik agent2 (Layer 0, FTS5 lintas bahasa; bundle istilah otomasi 40 bahasa terkompresi <500KB).
- Aturan kunci (agent4 v0.3): pembaruan terjemahan **TIDAK menaikkan content_version manifest** — menghindari invalidasi receipt palsu.
- Mandat mengharuskan AI agent di negara mana pun bisa memahami template: deskripsi, cara kerja, fungsi, tujuan, dan note node.

## 2. Desain permukaan MCP

### 2.1 Resource template + parameter bahasa

```
n8n://templates/{id}            → bahasa sumber (source_lang) — perilaku default
n8n://templates/{id}?lang={code} → 3 pilar + ringkasan node notes dalam bahasa tsb
```

- `lang` = kode BCP-47/ISO 639-1 (id, en, zh, es, ar, ja, fr, de, pt, ru, ko, …).
- Payload per-request **satu bahasa** — bukan 40 bahasa sekaligus (budget token & RAM; P-2 fondasi).
- Daftar bahasa tersedia + % cakupan terjemahan disertakan sebagai metadata payload (tanpa request terpisah).

### 2.2 Isi payload (≤500 token inti — tokenizer #419a)

```json
{ "uri": "n8n://templates/{id}?lang=id",
  "lang": "id", "source_lang": "en",
  "available_langs": ["en","id","zh","es","ar","ja"], "coverage": {"id": 1.0},
  "content_version": 3,          // versi MANIFEST (naik hanya saat konten sumber berubah)
  "i18n_rev": 7,                 // versi baris terjemahan (agent2) utk bahasa ini — deteksi basi terjemahan
  "body": {
    "cara_kerja": "…", "fungsi": "…", "tujuan": "…",     // pilar #547
    "nodes": [ {"name": "http-1", "role_note": "…"} ]    // ringkasan node notes ≤1 baris/node
  },
  "detail_ref": "workflow_i18n:rowid…" }                 // pointer file/baris utk teks penuh
```

### 2.3 Aturan bahasa (fallback deterministik)

1. `lang == source_lang` → teks sumber (selalu ada).
2. `lang` tersedia di `workflow_i18n` → terjemahan; sertakan `i18n_rev`.
3. `lang` TIDAK tersedia → **fallback ke source_lang** + field `"lang_fallback": true` (eksplisit, bukan diam) → klien tahu teks bukan bahasa yang diminta.
4. `lang` tak dikenal/format salah → **error standar** resource (URI tak dikenal) + daftar `available_langs` di pesan. (SEP-1303: kesalahan input = Tool/Resource Execution Error, bukan Protocol Error.)
5. Tidak ada request multi-bahasa sekaligus di v1; katalog bahasa cukup dari metadata.

### 2.4 Kait Hub (mandat #524 + #547)

- Template Hub memakai skema yang sama → `hub_inspect {hub_id, lang}` dan resource `hub://skills/{hub_id}/manifest?lang=` mengikuti aturan yang sama (deskripsi ≤120 token per ai_skill tetap dari bahasa sumber; teks panjang via resource ini).

## 3. Keamanan & integritas

- Terjemahan = data terverifikasi (agent2, Layer 0): **tidak ada terjemahan on-the-fly via LLM di jalur MCP v1** — menghindari prompt-injection via terjemahan tidak terjaga dan biaya tak terduga ("on-demand translation" #547 = keputusan terpisah, di luar resource ini sampai ada pemilik & gate).
- Isi resource = data; dirender escaped; taint rule berlaku bila teks sumber dari web (template Hub: TAINTED-EXTERNAL sampai disanitasi agent5).
- Redaksi oracle #394: terjemahan tidak boleh memuat rahasia (sumber teks = deskripsi, bukan data runtime).

## 4. Gate penerimaan (falsifiable; satuan eksplisit)

| ID | Gate | Target | Cara ukur |
|---|---|---|---|
| I-1 | Jangkauan bahasa | 40/40 bahasa → terjemahan ATAU fallback eksplisit (0 payload kosong/tanpa penanda) | 40 request sintetis per template uji |
| I-2 | Budget token | payload per bahasa ≤500 token inti (tokenizer #419a) | ukur tokenizer |
| I-3 | Konsistensi | `lang==source_lang` identik dgn dokumentasi{} manifest; terjemahan ≠ kosong saat diklaim tersedia | diff otomatis 20 template |
| I-4 | Error handling | lang tak dikenal → error standar + daftar bahasa (20/20) | uji sintetis |
| I-5 | Versi | `content_version` tak berubah saat hanya terjemahan berubah; `i18n_rev` naik | 20 kasus: update terjemahan saja |
| I-6 | Fallback | template dgn cakupan terjemahan 0 → `lang_fallback:true` + source_lang (100%) | uji template kurang-terjemahan |

## 5. Ketergantungan & pembagian kerja

| Item | Pemilik | Catatan |
|---|---|---|
| Skema `documentation{}` + `i18n{source_lang, translations_ref}` di manifest | agent4 | sudah (manifest v0.3 `e235afeb…`) |
| Tabel `workflow_i18n` + FTS5 + bundle <500KB | agent2 | sumber derived data; `i18n_rev` per (workflow, lang) |
| Kait UI kanvas (3 pilar + node notes) | agent4/agent9 (frontend nanti) | di luar MCP |
| Tokenizer #419a | matt | angka token |
| On-demand translation (WCB-TRANS) | agent6 (klaim #574) + agent5 (gate) | komponen opsional pasca-v1 |
| Pemetaan URI di registri | agent7 | REGISTRY v0.2 |

## 6. Pertanyaan terbuka

- O-I1: `lang` default bila parameter hilang = source_lang (rekomendasi) vs en — perlu konfirmasi matt.
- O-I2: node notes per-node di resource ini (ringkasan ≤1 baris) vs resource terpisah `n8n://nodes/{name}?lang=` — rekomendasi: ringkasan di sini + detail via pointer (hemat permukaan).
- O-I3: "on-demand translation" kustom (#547) — **komponen diklaim agent6 sebagai WCB-TRANS (#574, sandboxed translation worker)**; rekomendasi v1 tetap: terjemahan terjaga (data terverifikasi) lebih dulu; WCB-TRANS menyusul dengan gate anti-injection + kebijakan biaya milik agent6/agent5. Catatan konvergensi: klaim agent6 selaras — resource MCP v1 membaca tabel `workflow_i18n`; output WCB-TRANS (bila disetujui) ditulis balik ke tabel yang sama sebagai derived data ber-`i18n_rev`.

## 7. Referensi

Mandat #547/#548/#549; `AGENT4-WORKFLOW-HUB-MANIFEST.md` v0.3 (`e235afeb…`); `AGENT7-JIT-ROUTING-REGISTRY.md`; fondasi MCP (INTEGRATION-SPEC v0.4, PROTOCOL-ROSTER, THREATMODEL); spec MCP 2026-07-28 (ttl/cache).

*Ditulis oleh agent7 (ROLE_AI_MCP). Koreksi & review dipersilakan.*
