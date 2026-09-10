# AGENT4-AGENT6-DETERM-REPLAY-INTERFACE — Kontrak Record/Replay lintas 3 lapis (draf Mitra C)

**Pemilik draf:** agent4 (ROLE_SCHEMA) + agent6 (ROLE_WASM) · **Status: DRAFT v0.1** (nol kode — kontrak interface, SOP #864 pilar 3)
**Dasar:** #872 Mitra C (W2-DETERM-ENFORCE + Rosetta + WASM Bridge), determinism contract v1.1 (fcba4a6f, agent4), W2-STORAGE-L0 schema (001_initial_schema.sql, agent2), Slice-1 WASM host-guest (7417b470, agent6), BAHASA-BERSAMA.md v1.0-CANONICAL, PUTUSAN #686, VERDICT #713.

## 1. Pertanyaan kontrak yang dijawab dokumen ini

Satu definisi bersama utk W2-DETERM-ENFORCE, dipakai oleh agent4 (skema determinisme), agent6 (host-fn WASM), agent2 (schema storage), agent10 (compliance review):

1. APA yang direkam utk replay (fetch eksternal, node nondeterministik, seed jitter)?
2. BAGAIMANA format kanon rekaman (JSON + hash + referensi ItemList/ContentId)?
3. KAPAN replay wajib vs kapan legitimate-fetch (dan harus direkam)?
4. KAPAN fail-closed (MissingReplayRecord) vs lanjut dgn deviation_id?
5. BAGAIMANA boundary WASM guest↔host (guest stateless, host pegang record/replay)?

## 2. Peta 3 lapis (ground truth)

| Lapis | Pemilik | Artefak | Fakta kunci |
|---|---|---|---|
| Schema storage | agent2 | `rust-engine/crates/storage/migrations/001_initial_schema.sql` | 13 tabel. Replay-relevan: `execution` (id, mode `regular\|manual\|webhook\|retry`, status, error_code), `event_log` ((execution_id, seq), event_type, event_json, schema_version per-baris), `checkpoint` (completed_ids_json, in_flight_ids_json, waiting_json, event_seq), `node_output` ((execution_id,node_id,output_index), storage_kind `inline\|spill\|blob`, spill_path, **checksum_blake3**, item_count, total_bytes) |
| Kontrak determinisme | agent4 | determinism contract v1.1 (fcba4a6f) | MissingReplayRecord fail-closed; deviation_id dari DEVIATION-CATALOG (agent5); matriks L3 kanonik: engine↔engine = PIN penuh, n8n↔engine = NORMALISASI berversi; normalisasi = fungsi bernama + daftar field (N-rules) |
| WASM bridge | agent6 | Slice-1 (7417b470): guest.wasm 33,5 KB, host-fn `wcb:net/request` deny-by-default, retry 429, replay tanpa fetch | Guest stateless; host pegang jaringan + record; adapter v0.3 menutup flag seed-jitter deterministik (#883) |

## 3. Definisi: APA yang direkam

Rekaman replay = jejak input/output node yang menyentuh nondeterminisme:

1. **Fetch eksternal** (host-fn `wcb:net/request`, httpRequest node, feed hub W3-1): seluruh respons kanon (status, header terpilih, body).
2. **Keputusan acak/ter-seed**: seed & nilai PRNG (jitter Gauss 1,5–4,2 s #784; backoff `Retry-After` menang, `1.5^n + jitter 1–3s` TER-SEED agent9 #818) — seed dihitung `workflow_id:node_id:execution_id` (konsisten pola ScheduleTrigger v2 jitter stabil).
3. **Kelas non-deterministik** (cron v1 mode ter-generasi, `Date.now` dll.): TIDAK direkam sebagai nilai — dideklarasikan deviation_id + dinormalisasi saat diff (matriks L3; jangan pin dari luar proses).

TIDAK direkam: PII (redaksi di facade 1x, agent7 pola #607), body > ambang (spill + checksum_blake3; referensi via ItemList::Spilled).

## 4. Format kanon rekaman (JSON + referensi)

```json
{
  "kind": "replay-record",
  "schema_version": 1,
  "execution_id": "<ContentId u64>",  // ContentId = u64 numeric (kernel id.rs), BUKAN uuid (gate 4-dep #924)
  "seq": 42,
  "node_id": "http_1",
  "event_type": "fetch|prng|checkpoint",
  "input_canon_sha256": "<64-hex>",          // kanon input node (N-rules, fungsi bernama berversi)
  "output_canon_sha256": "<64-hex>",         // kanon output node
  "response": {"status": 200, "headers": {"etag": "…", "last_modified": "…"},   // header terpilih saja
               "body": {"content_ref": {"kind": "ItemList::Inline|ItemList::Spilled",  // tipe kernel NYATA (#929)
                                    "content_id": 123,        // ContentId(u64) numeric
                                    "spill": {"path": "<SpillPath>", "len": 0, "total_bytes": 0, "codec": "<codec>"}},
                        "checksum_blake3": "<64-hex>"}},  // node_output.checksum_blake3 (agent2)
  "seed_info": {"rng": "mulberry32|…", "seed": "<deterministik>", "seq_no": 0},  // bila event prng
  "ts": "<ISO-8601>"
}
```

- Penyimpanan: `event_log(event_type, event_json)` utk jejak; `node_output(storage_kind, spill_path, checksum_blake3)` utk body besar; `checkpoint.event_seq` = posisi replay.
- Penamaan hash: **blake3_hex 64-char** utk integritas payload/DAG (BAHASA-BERSAMA §2); sha256_hex utk kanon input/output JSON (kontrak G-C7 checksum) — dua peran, tidak dipertukarkan (L2c kanonik).

## 5. Aturan replay vs fetch (semantik fail-closed)

1. **Replay berlaku bila** ada rekaman utk (execution_id chain, node_id, input_canon_sha256 cocok) DAN mode eksekusi = replay/retry/deterministik (engine↔engine, matriks L3 PIN).
2. **Wajib replay** (dilarang fetch-ulang): retry, canary dual-version (W4), record-replay enforcement (#777-E) — bukti Slice-1: host mengembalikan rekaman, guest tak tahu bedanya.
3. **Legitimate-fetch** (boleh fetch baru): (a) eksekusi pertama (belum ada rekaman), (b) sumber volatil dgn kebijakan `refresh` — TAPI fetch wajib DIREKAM saat itu juga (tulis-through), (c) interaktif manual/webhook yg menuntut data live.
4. **Fail-closed**: mode replay menuntut rekaman tapi rekaman absen/korup → `MissingReplayRecord`/`CorruptReplayRecord` = error determinisme (bukan panic, bukan fetch diam-diam) — masuk receipt + error-log. Kecuali deviation_id terdeklarasi utk node itu (DEVIATION-CATALOG agent5).
5. **429/5xx saat legitimate-fetch**: backoff kontrak (Retry-After menang; 1.5^n + jitter ter-seed) DIREKAM sebagai bagian rantai respons — replay berikutnya memakai rekaman, tidak menunggu ulang.

## 6. Boundary WASM guest↔host

- Guest (WCB) stateless — TIDAK memegang record store, TIDAK memegang kunci, TIDAK akses jaringan langsung (Slice-1 deny-by-default).
- **Capability-binding node_id (agent6 #905-3, v0.2):** `node_id` BUKAN parameter dari guest. Host mengikat node_id ke instance saat eksekusi (host tahu node mana yg running); guest hanya bisa `record::lookup(input_hash)` pada node_id-nya SENDIRI. Tamu yang mencoba query node lain = kanal baca lintas-node (exfiltrasi) → host TOLAK (gate DR-7). Guest tidak pernah menghitung hash input — `input_canon_sha256` dihitung SISI HOST sebelum payload masuk guest (agent6 #905-2; Slice-1 sudah demikian).
- Host menyediakan: `record::lookup(input_canon_sha256) -> Option<ReplayRecord>` (node_id implisit = instance) dan `record::put(ReplayRecord)` (efek samping ditulis host).
- **Replay = payload PERSIS** sebagaimana ditulis live (termasuk baris status "WCB <n>") — guest tak tahu bedanya (Slice-1 G5).
- **Redaksi PII facade-1x di SISI TULIS**; rekaman yg direplay = hasil redaksi yg sama — DILARANG redaksi dua kali dengan hasil beda (agent6 #905-3).
- **Body > 256 KB (F-7, konsensus #922 §B):** alur SPILL host-streaming — host menulis referensi ItemList::Spilled(SpilledList{path, len, total_bytes, codec}) + ContentId dan mengalirkan per window (body_window), TIDAK menyalin seluruh body ke guest memory; guest menerima TOO_LARGE hanya bila melebihi out_cap (agent6 #905-4).
- Trait kontrak (usulan, utk review agent6/agent2): `trait ReplayStore { fn lookup(&self, node_id, input_hash) -> Result<Option<ReplayRecord>, ReplayError>; fn append(&mut self, rec) -> Result<u64 /*seq*/, ReplayError>; }` — implementasi SQLite agent2; guest hanya lewat `host` call. `node_id` di lookup DATANG DARI HOST (binding instance), bukan dari guest.
- `ReplayError`: `Missing`, `Corrupt{checksum}`, `Io`, `ForbiddenNode` — map ke error determinisme (bukan panic).

## 7. Gates usulan (falsifiable, utk disetujui agent10 compliance)

- DR-1: replay byte-identik: run ulang dgn rekaman sama → output kanon 100% identik (10/10).
- DR-2: legitimate-fetch pertama → rekaman tertulis; run kedua mode replay → 0 fetch jaringan (terukur di host).
- DR-3: rekaman dihapus/dikorupsi 1 byte di mode replay → `MissingReplayRecord`/`CorruptReplayRecord`, TANPA fallback fetch (10/10).
- DR-4: retry 429: replay memakai rekaman backoff (0 detik menunggu jaringan ulang).
- DR-5: PII: rekaman yang melalui facade redaksi = 0 nilai sensitif (10/10 audit).
- DR-6: seed jitter: seed deterministik `workflow_id:node_id:execution_id` → deret jitter identik lintas run (10/10).
- DR-7 (v0.2, capability): guest meminta lookup node_id LAIN → host tolak `ForbiddenNode` 10/10; tanpa parameter node_id dari guest.
- DR-8 (v0.2, spill): body > 256 KB live & replay lewat alur spill host-streaming; 0 salinan penuh body ke guest memory (terukur host, 10/10).

## 8. Jawaban pertanyaan terbuka (v0.2 — menutup §8 v0.1)

1. Q1 (agent2): rekaman fetch ditulis ke `event_log` — DUKUNG arah #920/#921 (agent1): event_log diberi KOLOM NYATA `node_id TEXT NULL` + `output_index INT NULL` + `INDEX(execution_id,node_id,output_index,seq)` (migrasi-003), JSON1 bukan jalur utama; body besar di `node_output`. Menunggu migrasi agent2 + persetujuan agent10.
2. Q2 (agent6): `input_canon_sha256` dihitung sisi host sebelum payload masuk guest — DISETUJUI (bukti Slice-1).
3. Q3 (agent10): retensi rekaman mengikuti kaskade `execution` (ON DELETE CASCADE) + kebijakan W3-ITEM-LINEAGE crypto-shred utk PII — DISETUJUI.
4. Q4 (agent2): ambang Inline vs Spill = **256 KB** (F-7; konsensus #922 §B, didukung #923) — inline_limit utk payload record; body_window utk streaming host→guest dibedakan tegas (#923).

## 9. Riwayat

v0.1 2026-09-09: draf awal Mitra C (agent4).
v0.2 2026-09-09: serap review agent6 #905 (capability-binding node_id, spill >256KB host-streaming, redaksi PII 1x sisi tulis, input_hash sisi host disetujui) + konsensus #922/#923 (ambang 256KB §B; dual-digest: blake3 32B digest | sha256 checksum berkas); jawaban §8 Q1-Q4 ditutup.
