# W4-HUB-AUTOUPDATE — Spesifikasi Auto-Update Workflow Hub (draf agent4)

**Pemilik:** agent4 (ROLE_SCHEMA) · **Wave 4 · P2 · Status: IN_PROGRESS (CLAIMED 2026-09-09 via swarm-task; W3-EXEC-ENVELOPE + W3-HUB-INGRESS = DONE)**
**Patuh freeze #386:** dokumen — nol kode. Zero-code mandate = Wave 0-1 (#723); W4 = Wave 4 →
deliverable kini = SPEC FINAL + kontrak integrasi; prototipe kode menunggu pembukaan wave (konfirmasi fern).
**Dasar:** PRD-3-CANONICAL-APPROVED (07:53 — standar tertinggi; K-8 SAH → `[N]` terbuka;
K-3 Opsi A SAH #726; §5 L2c urutan lapisan anchor), mandat #524/#526 (Hub),
manifest v0.6 (059231ba), katalog hub-intel-types v1.2 (033924f6, kind=scrape),
agent9 W3-HUB-INGRESS (64fa9fd1), W3-EXEC-ENVELOPE crate v0.1 + AGENT10-EXEC-ENVELOPE-SPEC v1.2 (6286c1ef),
katalog hub-intel-types (a1e9b4ef), SEC-HUB-01 (agent10 #633), PUTUSAN #686,
VERDICT #713 (6 syarat — diserap §4a/§6/§8), gate #374, #435, #540, OPEN-HUB-1.

## 1. Batas (anti-tumpang-tindih dgn W3-1 agent9)

| | W3-1 (agent9, TRANSPORT) | W4 (agent4, KEPUTUSAN KONTEN) |
|---|---|---|
| Fungsi | KAPAN & BAGAIMANA bytes tiba: jadwal cron 5-field (otoritas engine, #540/#435), rate-policy (host-global OPEN-INTEG-12), etag kondisional, timeout | APA YANG TERJADI setelah kandidat tiba: verifikasi pin/schema-pin, diff-gate, keputusan swap/notify/alarm, kenaikan content_version, receipt, retensi |
| Output | CANDIDATE bundle (lihat §3) | KEPUTUSAN + state baru (manifest/registri/receipt) |

W4 TIDAK fetch; W3-1 TIDAK memutuskan naik-versi. Keduanya bertemu di kontrak §3.

## 2. State machine keputusan

```
CANDIDATE bundle (dari W3-1)
 -> 1 VERIFIKASI JANGKAR (#686): (a) SUMBER STATIC: sha256 vs VALUE-PIN katalog (WAJIB,
      nullable) — GAGAL -> REJECT + alarm + versi lama TETAP live; tanpa pin = VERIFIED_TOFU
      eksplisit. (b) SUMBER VOLATILE: TANPA value-pin; jangkar = katalog + ETag (W3-1) +
      record-replay (W2-DETERM-ENFORCE fail-closed). (c) OPSIONAL: structural_guard
      (schema-pin per-template, default false) -> deteksi break format API.   [0 mati senyap]
 -> 2 STAGE: parse via Rosetta (L-1) + kanon normalizer. Parse gagal -> REJECT + alarm.
 -> 3 DIFF-GATE (gate #374, mode deterministik #368):
      - template TANPA sumber eksternal (murni transformasi): eksekusi dual-version
        live vs candidate, seed sama (OPEN-HUB-1). Output kanon IDENTIK -> SILENT SWAP
        (live = candidate; tercatat di audit log Hub; content_version TETAP — tidak ada
        notifikasi palsu, receipt tidak invalidasi palsu). BERBEDA -> lihat (a).
      - template DENGAN fetch eksternal (intel/feed): diff nilai TIDAK mungkin identik
        (harga berubah) -> diff = STRUKTURAL: kanon-struktur output (set field, tipe,
        outputs.max_tokens tetap) + kontrak per-node (deviation_ids); ETag/record-replay
        jadi jangkar input (W2-DETERM-ENFORCE agent10); structural_guard aktif -> plus
        schema-pin sumber.
   (a) perubahan perilaku (output kanon beda / struktur beda) -> content_version+1,
       update{last_check,next_check}, NOTIFIKASI subscriber. Kebijakan on_change manifest:
       "new-version" (auto-live kandidat) vs "notify-only" (kandidat disimpan, approve
       manual -> live). DEFAULT v1 = notify-only + auto-live hanya setelah template punya
       >=2 siklus update stabil berturut (canary-lite; koordinasi konsep dgn W4-CANARY-EXEC
       agent1, BUKAN mencaplok: canary = runtime multi-package, ini = stabilitas template).
   GAGAL (fetch error dari W3-1 / source down) -> retensi versi lama + alarm (fail-open-
   informatif, konsisten agent9 W3-1).                                  [0 template mati]
```

## 3. Kontrak interface W3-1 -> W4 (CANDIDATE bundle)

```json
{
  "hub_id": "hub-0001", "source_id": "coingecko-simple-price",
  "ts_trigger": "…",              // timestamp TRIGGER deterministik (engine-scheduler)
  "etag": "…", "http_status": 200,
  "raw_size_bytes": 12345,        // ukuran byte mentah; disimpan sbg FILE (disk), bukan RAM (F-7)
  "fetch_meta": {"ok": true, "timeout_s": 0,                 // JANGKAR-TRANSPORT W3-1 saja (A-1 #954):
                 "etag": "…", "last_modified": "…",          // verifikasi pin/schema-pin = WEWENANG W4
                 "content_blake3": "<64-hex>"},              // digest konten (C-02) utk record-replay
  "payload_ref": "…"              // path file kandidat; 0 penyalinan konten ke memori
}
```

**Kasus 304 Not-Modified (A-2 #954):** `http_status=304` = bundle SAH tanpa konten baru
(`payload_ref` = null/ref cache) → W4 verdict `retain` + NO change + `last_check`/`next_check`
TETAP MAJU — etag kondisional adalah jangkar utama sumber volatil (§4.2), 304 = jalur paling
sering; tanpa aturan ini last_check hanya maju saat konten berubah.

**etag/last-modified absen (A-3 #954):** beberapa RSS tak menyediakan — record-replay TETAP
jangkar (etag = efisiensi kondisional, BUKAN syarat); content_blake3 (bila ada) sbg pembanding.

Keputusan W4 -> {verdict: silent-swap|promote|notify|reject|retain-alarm, new_content_version?,
reason, alarm_level} — ditulis ke state Hub (registri agent7) + receipt. Receipt menyertakan
pin_snapshot yang diambil SAAT keputusan (S3 #713): {pin_id, pin_kind: value|schema|null,
pin_value_sha256, verdict, tsa_anchor_id: null|<id>} + `content_blake3` (n2 #954) — lihat §6.

## 4. Kebijakan pin & jangkar (OPEN-INTEG-11 — PUTUSAN agent10 #686, Plt. Security Gatekeeper)

PUTUSAN: DUA KELAS, bukan satu kebijakan (diterima penuh; selaras SEC-HUB-01 butir 2-3):

1. SUMBER STATIC (file/dataset ber-versi): pin expected_sha256 WAJIB (nullable) di katalog;
   tanpa pin = VERIFIED_TOFU eksplisit. Mismatch -> REJECT + alarm + re-pin pemilik katalog
   (re-pin HANYA `authorized_by` manusia/pemilik katalog — S1 #713; mesin tidak mengubah pin).
   Setiap expected_sha256 WAJIB menyebut `hash_domain`: "raw-bytes" (byte mentah file dari
   kanal distribusi — pilihan utk static source) atau "canon-json"+versi kanon (S4 #713;
   kanonikalisasi = fungsi bernama + daftar field berversi, wilayah agent1+agent4). Status
   label: lihat §4a.
2. SUMBER VOLATILE (semua 16 sumber katalog kini): pin TIDAK diwajibkan — value-pin = alarm
   palsu permanen. Jangkar v1 = KATALOG + ETag/cache kondisional (W3-1) + RECORD-REPLAY
   (W2-DETERM-ENFORCE agent10: MissingReplayRecord fail-closed). Status verified-live DIBACA
   = VERIFIED_TOFU ("hidup dari VPS tanggal T", BUKAN "asli dari penerbit").
3. SCHEMA-PIN = OPSIONAL defense-in-depth (bukan kewajiban): per-template opt-in
   `update.structural_guard: true` (default false). Mendeteksi BREAK FORMAT API (field
   hilang/tipe berubah) — beda dari value drift; jarang false-alarm. Kapan aktif: template
   dgn output kontrak ketat (dikonsumsi AI/machine). SSE: guard per-event.
   Status lulus = **SHAPE-VERIFIED** (schema-pin mengunci BENTUK, bukan ISI — S2 #713),
   BUKAN "verified"; "verified" hanya utk value-pin cocok + anchor (S1). Plausibility
   envelope (rentang nilai wajar per-tipe) = perluasan opsional post-v1 saat guard ON.
4. ETIKA (agent10 #686): sumber "etika-pending" BOLEH uji internal, DILARANG utk template
   live sampai gate etika (agent5) disetujui. Konsekuensi manifest: review_status=live
   mensyaratkan SEMUA sources[] lolos gate etika; review_status=live = lolos review
   pipeline, BUKAN keaslian (konsisten SEC-HUB-01 & VERIFIED_TOFU).
5. Konsekuensi v1: template Hub dari sumber volatil = TOFU by-design; klaim produk jujur:
   "terverifikasi hidup + terekam", bukan "dijamin asli".

## 4a. Taksonomi status jangkar & aturan label (VERDICT #713 agent10 — DISERAP PENUH, v0.3)

| Kondisi (v1) | Enum internal | Label API/UI | Boleh "verified"? |
|---|---|---|---|
| value-pin cocok + anchor TSA eksternal (W0-ANCHOR-SPEC D-A6, agent10) | `anchored_verified` | "terverifikasi-terjangkar" | YA — satu-satunya |
| value-pin cocok, TANPA anchor TSA | `pin_match` | "pin cocok (TOFU-pending)" | TIDAK (S1) |
| schema-pin lulus (struktur/format) | `shape_verified` | "struktur cocok (SHAPE-VERIFIED)" | TIDAK (S2) |
| jangkar katalog+ETag+record-replay / tanpa pin | `verified_tofu` | "hidup dari VPS tgl T (TOFU)" | TIDAK (S6) |
| pin mismatch / format API berubah | `reject` | "REJECT + alarm" | — |

- **S1:** anchor eksternal = satu-satunya lapisan penahan root NOPASSWD (kanonik §5 L2c,
  urutan lapisan). Tanpa anchor, status jujur = kelas TOFU/pending. W0-ANCHOR-SPEC
  (agent10, IN_PROGRESS) = prasyarat sebelum klaim `anchored_verified` muncul di produk.
- **pin_change_log** (katalog agent9 — pemilik katalog): tiap entri set/ubah/lepas pin wajib
  `authorized_by` = manusia/pemilik katalog; mesin hanya mengusulkan + alarm, TIDAK mengubah
  pin sendiri (S1).
- **S3 — RECEIPT:** snapshot pin SAAT keputusan diambil & disimpan: `{pin_id, pin_kind:
  value|schema|null, pin_value_sha256, verdict, tsa_anchor_id (null|<id>)}` — lihat §6.
- **S5 — BUKTI REJECT permanen:** sha256 kandidat + ringkasan selisih + verdict; byte mentah
  kandidat disimpan 30 hari lalu crypto-shred (rujuk AGENT10-ITEM-LINEAGE-SPEC §6, agent10).
- **S6 — LABEL:** enum & label API/UI wajib BEDA antar kelas; dilarang merender
  verified_tofu/shape_verified/pin_match sebagai "verified". Gate: AU-7 (§8).
- **S4 — hash_domain:** lihat §4.1 (expected_sha256 wajib sebut domain byte; kanonikalisasi
  = fungsi bernama berversi, wilayah agent1+agent4).

## 5. Rate-policy & governor (OPEN-INTEG-12 — endorse + batas lapis)

- HOST-GLOBAL governor per source_id (agent9, fetch-governor) = eksekutor batas global.
- Manifest rate_policy = KONTRAK per-template (seberapa sering TEMPLATE ini polling + cap
  byte/timeout); nilai efektif = max(batas-global sumber, min_interval template) — SATU
  titik penegakan (governor), TANPA double-accounting. Lapis beda dgn admission eksekusi
  agent1 §5.2 (agent1 #658: tiga-governor konsisten pola berlapis).

## 6. Konsumen state & efek samping

- Manifest: template_sha256 baru, content_version (aturan §2), update{last_check,next_check}.
- Receipt: {template_sha256, content_version, deviation_ids, exec-verdict ref, ts,
  pin_snapshot {pin_id, pin_kind: value|schema|null, pin_value_sha256, verdict,
  tsa_anchor_id: null|<id>}} — snapshot pin diambil SAAT keputusan (S3 #713);
  receipt-outdated rule (agent3 #545.4): naik content_version -> receipt lama FLAG outdated.
- Bukti REJECT (S5 #713): {sha256 kandidat, ringkasan selisih, verdict} permanen di
  error-log/registri; byte mentah kandidat dipertahankan 30 hari lalu crypto-shred
  (konsisten AGENT10-ITEM-LINEAGE-SPEC §6; 0 byte tersisa tanpa shred-record).
- Registri agent7: URI hub://skills/{hub_id}…, freshness utk hub_list (kelas freshness).
- i18n: TERJEMAHAN TIDAK tersentuh (i18n_rev terpisah — #583) kecuali konten sumber teks
  berubah (itu bagian output template, bukan metadata).
- Notifikasi: saluran internal (bukan platform eksternal); alarm -> error-log/registri.

## 7. Biaya & RAM

Diff-gate dual-version = eksekusi nyata 2x per update terjadwal (jarang: sekali per
jadwal/sumber). Kandidat & artefak = file di disk; 0 pertumbuhan RAM persisten (F-7).
Verifikasi pin = parse JSON kecil; deterministik; tanpa jaringan tambahan.

## 8. Gates (falsifiable, satuan eksplisit)

- AU-1 (=HUB-2): sumber berubah isi -> content_version+1 + notif; sumber mati -> retain +
  alarm; 10 skenario, 0 mati senyap.
- AU-2: perubahan kosmetik byte (whitespace/reformat) dgn output kanon identik -> silent
  swap + tercatat; content_version TETAP; 10/10 tanpa notifikasi palsu.
- AU-3: (statis) pin katalog DISET + hash beda -> REJECT + alarm + versi lama live 10/10;
  (volatil) perubahan NILAI -> tanpa alarm 10/10; (opsional structural_guard ON) format API
  berubah (field hilang/tipe ganti) -> REJECT + alarm + versi lama live 10/10; nilai saja
  berubah dgn guard ON -> TIDAK alarm 10/10 (guard tidak false-alarm pd value drift).
- AU-4: determinisme keputusan: 2 run diff-gate seed sama = keputusan sama 10/10.
- AU-5: rollback: versi lama (sha-addressed) restore manual 10/10 kapan pun.
- AU-6 (integrasi): W3-1 -> W4 -> registri -> MCP freshness konsisten 10/10 (agent9/agent7).
- AU-7 (label jujur — S6 #713): tiap keputusan berstatus pin/TOFU/shape dirender label kelas
  yang benar (§4a); 0 kemunculan verified_tofu/shape_verified/pin_match dirender "verified"
  di API/UI/registri — 10/10.
- AU-8 (integrasi envelope — v0.4): tiap keputusan update punya entri envelope (rantai BLAKE3
  + anchor TSA, §10) terverifikasi anchored/unanchored-jujur; riwayat update bisa diverifikasi
  offline dari artefak — 10/10.

## 10. Integrasi Dynamic Release Envelope (W3-EXEC-ENVELOPE — DONE, agent10; judul task W4)

W4 = "Hub auto-update + **dynamic release envelopes**". Envelope (crate `w3-exec-envelope`
v0.1, agent10: rolling hash-chain BLAKE3 + anchor TSA eksternal, 21 test + 30 gate + E2E
FreeTSA) = jangkar audit riwayat eksekusi & keputusan. Integrasi W4 (kontrak, nol kode):

1. TIAP KEPUTUSAN UPDATE (silent-swap|promote|notify|reject|retain-alarm) dicatat sbg ENTRY
   ENVELOPE oleh W4: `{event: hub-update-decision, hub_id, content_version lama→baru,
   template_sha256 lama→baru, verdict, reason_ref, ts_trigger}` → masuk rantai (penulisan
   via host/registri, domain separation C-02: konteks `hub-autoupdate` ≠ `execution`).
2. ANCHOR: head rantai keputusan Hub di-anchor TSA periodik (jalur agent10/W0-ANCHOR);
   verifier memakai semantik A1.5b/A.3: `VERIFIED_ANCHORED` | `VERIFIED_UNANCHORED` — jendela
   jujur; dilarang label "verified" polos utk UNANCHORED (konsisten #713 S1/S6).
3. RELEASE ENVELOPE sbg TRANSPORT kandidat (post-v1): kandidat template dirilis ber-envelope
   {template_sha256, envelope_head_ref} → W4 memverifikasi head ANCHORED sblm promote.
   v1: CUKUP catatan keputusan ber-envelope (butir 1-2) — transport ber-envelope = OPEN post-v1.
4. Gate: AU-8 — tiap keputusan update punya entri envelope terverifikasi (anchored/unanchored
   jujur), 10/10; riwayat update bisa diverifikasi offline dari artefak (pola E2E-5 #README).

## 11. Rujukan & koordinasi

Manifest v0.5 (d036f4a3) §3/§6/§8/§12; DETERMINISM-CONTRACT v1.1 (fcba4a6f) §7; agent9
W3-HUB-INGRESS (64fa9fd1) + katalog (a1e9b4ef); agent10 SEC-HUB-01 + PUTUSAN OPEN-INTEG-11
(#686) + VERDICT #713 (6 syarat, §4a) + W0-ANCHOR-SPEC (IN_PROGRESS, prasyarat anchored)
+ W2-DETERM-ENFORCE (record-replay fail-closed); agent1 #536/#540/#550/#688;
PRD-3-CANONICAL-APPROVED §4 (tabel Wave) §5 L2c (urutan lapisan anchor).
Keputusan diminta utk PRD-3: default on_change (usul: notify-only + canary-lite §2a),
OPEN-INTEG-12 (rate-policy §5), OPEN-HUB-1 engine dual-version dukungan, gate etika sumber
(agent5) sbg prasyarat review_status=live (§4.4).

## 12. Riwayat

v0.1 2026-09-09: PREP (dependensi queue W3-EXEC-ENVELOPE/W3-HUB-INGRESS belum DONE).
v0.2 2026-09-09: serap PUTUSAN agent10 #686 (dua kelas pin; volatil tanpa pin, jangkar =
katalog+ETag+record-replay; schema-pin -> structural_guard opsional; etika-pending).
v0.3 2026-09-09: serap VERDICT #713 (S1 anchor TSA + authorized_by, S2 SHAPE-VERIFIED + label,
S3 receipt pin_snapshot, S4 hash_domain, S5 bukti REJECT + shredding 30 hari, S6 label & AU-7);
selaraskan dgn PRD-3-CANONICAL-APPROVED (standar tertinggi; K-8 SAH, K-3 Opsi A — tidak
mengubah kontrak W4).
v0.4 2026-09-09: CLAIMED (W3-EXEC-ENVELOPE + W3-HUB-INGRESS DONE); status IN_PROGRESS.
Selaraskan dasar: manifest v0.6 (059231ba, BYOK/kind=scrape), katalog v1.2 (033924f6),
W3-EXEC-ENVELOPE crate v0.1 (agent10, 6286c1ef). Tambah §10 integrasi Dynamic Release
Envelope (tiap keputusan update = entri envelope + anchor TSA; AU-8; release-envelope
transport = OPEN post-v1). Tipe data rujukan = kanonik kernel (ItemList/ContentId, #924/#929).
v0.5 2026-09-09: serap REVIEW SILANG agent9 #954 (boundary W3-1<->W4, ENDORSE + 3 temuan + 2 nit):
A-1 fetch_meta.schema_pin DIHAPUS (verifikasi pin = wewenang W4; fetch_meta = jangkar-transport
etag/last_modified/content_blake3); A-2 http_status=304 didefinisikan (verdict retain, last_check
maju, payload_ref null); A-3 etag absen -> record-replay tetap jangkar; n1 raw_bytes ->
raw_size_bytes; n2 receipt + content_blake3.

