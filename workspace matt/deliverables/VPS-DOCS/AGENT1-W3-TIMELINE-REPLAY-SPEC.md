# AGENT1-W3-TIMELINE-REPLAY-SPEC — Snapshot Replay Engine (DRAF Mitra A)

**Pemilik draf:** agent1 (ROLE_CORE) · **Status: DRAFT v0.7** (nol kode — kontrak interface, SOP #864 pilar 3)
**REVIEW-#939 (agent10, diserap-penuh):** kunci posisional + `attempt` (1-based, non-null); `item_seq` stabil lintas-retry (posisi input asli, sparse utk retry parsial); identitas logis via `output_ref` BLAKE3-128 ter-salt (BUKAN item_seq); kolom jembatan `output_ref` di `node_output`; cross-link digest WAJIB salt `salt_exec` sama (anti-re-identifikasi, Recital 26).
**KOREKSI-#924 (matt, grep-terverifikasi):** tipe `PayloadRef`/`SpillId`/`SpillRef`/`uuid::Uuid`/`Bytes` TAK ADA di kernel (0 kecocokan; uuid MELANGGAR gate allowlist 4-dep). Spec ini mengikat ke tipe kernel terverifikasi-gate: `ItemList::{Inline,Spilled}`, `SpilledList`/`SpillPath`, `ContentId(u64, C-06-terbuka)`. Glosarium 'payload-ref' = KONSEP; tipenya = `ItemList`.
**Mandat:** matt #903 §3.1 (KLAIM W3-TIMELINE-REPLAY P0; gandeng agent2 utk event-log storage).
**Blokir klaim:** W2-DETERM-ENFORCE belum DONE (#906) — dokumen ini kerja spec-first yg diizinkan.
**Dasar (RULING-3-#1000-patuh: §2-BAHASA-BERSAMA-NON-BINDING, tak-dikutip):** `TimelineEvent` = tipe-BARU-didefinisikan-spec-ini-§3 (BUKAN kutipan §2); serialisasi = KANON-JSON-KETAT-§7-sendiri; kontrak Mitra-C
AGENT4-AGENT6-DETERM-REPLAY-INTERFACE v0.1 (98902a48), DDL agent2
`001_initial_schema.sql` (event_log/checkpoint/node_output), AGENT10-W2-DETERM-ENFORCE-SPEC,
AGENT10-ITEM-LINEAGE-SPEC v1.0 (kontrak-kolom utk storage = preseden).

## 1. Tujuan (satu kalimat)

Time-travel debugging yg jujur: dari `event_log` + snapshot periodik, rekonstruksi
transisi state per-node pada sembarang `seq`, dan verifikasi replay (eksekusi-ulang
== rekaman) — tanpa pernah mengklaim "verified" utk rentang yg tak tercakup rekaman.

## 2. Taksonomi `event_type` (nilai kanon utk kolom `event_log.event_type`)

| event_type | Ditulis oleh | Isi `event_json` (skema §3) |
|---|---|---|
| `node.started` | executor | node_id, attempt, input_ref (`ItemList::Inline\|Spilled` + checksum_blake3) |
| `node.finished` | executor | node_id, output_ref (`ItemList`), output_canon_sha256, duration_ms |
| `fetch.recorded` | host-fn / http node | = replay-record Mitra-C §4 (status, header terpilih, body ItemList) |
| `prng.seeded` | engine | rng, seed (`workflow:node:exec`), seq_no |
| `items.spilled` | spill store | spill_path, item_count, total_bytes, checksum (G-C7) |
| `checkpoint.saved` | engine | = baris checkpoint (completed/in_flight/event_seq) |
| `exec.retry` | engine | from_seq, reason, new_attempt (cabang eksplisit, bukan overwrite) |
| `gate.verdict` | determ-enforce | DETERMINISTIC/CONDITIONAL/NON_DETERMINISTIC + deviation_id? |

Aturan: `seq` monoton per execution; append-only; retry = event baru (riwayat tak ditulis-ulang).

## 3. Skema `TimelineEvent` (JSON kanon, disimpan di `event_json`)

```json
{
  "kind": "timeline-event", "schema_version": 1,
  "execution_id": "<uuid>", "seq": 42, "node_id": "http_1", "attempt": 1,
  "event_type": "node.finished",
  "state_digest": "<blake3_hex 64>",   // blake3(salt_exec || prev || kanon(event)); SALT-WAJIB-#939-butir-7
  "prev_digest": "<blake3_hex 64>",    // rantai tamper-evident (genesis = 64 '0')
  "io_canon_sha256": "<64-hex>",       // kanon input/output JSON (sha256 = CHECKSUM kanon, L2c)
  "payload_ref": {"type": "ItemList::Inline|ItemList::Spilled", "spill": "<SpillPath String|null>", "bytes": 123},
  "ts": "<ISO-8601>"
}
```

- `state_digest[i] = blake3(salt_exec || prev_digest[i-1] || kanon(event[i]))` → rantai putus = terdeteksi.
- KANON-JSON-KETAT (pelajaran RFC-#945/agent10-C-02): `kanon(event)` = serialisasi struct field-TETAP (urutan deklarasi, tanpa map/hash — NOL HashMap di skema); `schema_version` pin set-field; versi mayor serializer di-pin di Cargo (repro 5-tahun, argumen #939-butir-5). JSON ARBITRER tak pernah masuk digest — io di-pra-hash (`io_canon_sha256`) lebih dulu. Bila skema butuh map dinamis di masa depan → migrasi ke framing-biner-u32-BE ala AGENT10-EXEC-ENVELOPE-SPEC §2, bukan JSON-kanonik-ad-hoc.
- Body besar tidak inline: `ItemList::Spilled(SpilledList)` → `node_output` (storage_kind spill|blob + checksum_blake3).
- PII: redaksi di facade 1x (pola agent7 #607); event menyimpan digest + ref, bukan PII mentah.

## 4. Snapshot (seek O(1))

- Tiap K event (default K=1000, selaras envelope K=1000 agent10) engine menulis snapshot:
  `{execution_id, snap_seq, state_vec: [{node_id, state_digest, output_ref}], prev_digest}`.
- KONTRAK-KOLOM utk @agent2 (butuh DDL; Mitra-A handshake §7-Q2): tabel baru
  `timeline_snapshot(execution_id, snap_seq PK, snapshot_json, created_at)` ATAU
  kolom `snapshot_json` di `checkpoint`. Rekomendasi saya: tabel baru (checkpoint = recovery,
  snapshot = debugging; domain separation C-02).
- Replay `replay_to(exec, seq)`: snapshot ≤ seq terbesar → terapkan event (snap_seq..seq].

## 5. API Replay Engine (kontrak trait Rust — signature, bukan implementasi)

```rust
trait TimelineReader {                       // milik agent1 (crates/executor|timeline/)
    fn events(&self, exec: &str, from: u64, limit: usize) -> Result<Vec<TimelineEvent>, ReplayError>;
    fn verify_chain(&self, exec: &str) -> Result<u64, ReplayError>;  // -> verified_through_seq
}
trait Snapshotter {
    fn snapshot_at(&self, exec: &str, seq: u64) -> Result<Snapshot, ReplayError>;
}
trait ReplayVerifier {                        // butuh W2-DETERM-ENFORCE RecordSet
    fn replay_to(&self, exec: &str, seq: u64) -> Result<StateDiff, ReplayError>;
    // StateDiff: {seq, matches_record: bool, first_deviation: Option<u64>}
}
enum ReplayError { Missing { seq: u64 }, Corrupt { seq: u64, want: String, got: String }, Io(String) }
```

- Jendela jujur (pola agent10 A.3): kembalikan `verified_through=N`; tak pernah "VERIFIED" polos.
- Replay-vs-fetch: ikut semantik Mitra-C §5 (wajib replay utk retry/canary; MissingReplayRecord fail-closed).

## 6. Gates penerimaan (TTR-1..TTR-7, diukur saat implementasi)

| Gate | Klaim | Orákel |
|---|---|---|
| TTR-1 append-order | seq monoton, tak ada overwrite | 10k event acak → SELECT berurut == tulis |
| TTR-2 chain-verify | 1-byte edit event_json → Corrupt(seq tepat) | mutasi arsip (bukan DB hidup) |
| TTR-3 snapshot-seek | replay_to(seq) == state aktual @seq | banding eksekusi live 5k event |
| TTR-4 replay-equality | re-eksekusi deterministik == rekaman | RecordSet agent10 + diff kosong |
| TTR-5 honest-window | rentang tak-tercakup → UNVERIFIED, bukan PASS | potong 10% ekor → klaim menurun |
| TTR-6 spill-ref | body > ambang resolve via node_output + checksum cocok | 100MB payload, RAM < 500MB |
| TTR-7 retry-branch | retry = event baru; riwayat lama utuh | double-run → dua cabang terbaca |

## 7. Pertanyaan handshake (bantahan ≤60 mnt, lalu saya kunci default)

- Q1 → @matt: K snapshot default 1000 (selaras envelope)? Retensi event (unbounded vs ring)?
- Q2 → @agent2 (Mitra A): tabel `timeline_snapshot` baru vs kolom di `checkpoint`? + migrasi-003: event_log + node_id + output_index + attempt + INDEX-(execution_id,node_id,attempt,output_index,seq) + kolom-jembatan output_ref di node_output? Ambang Inline/Spill utk event (256KB? — selaras #899)?
- Q3 → @agent10: retensi PII di event_json (redaksi + TTL?) + fail-closed MissingReplayRecord cukup utk GDPR-audit?
- Q4 → @agent4/@agent6: pin `input_canon_sha256` N-rules versi utk replay equality (TTR-4)?
- Q5 → @matt: lokasi kanonik crate: `crates/timeline/` baru vs submodul `executor/`? (menunggu pola merge #903 §1; CATAT konflik-nama #924: roster-#914 `engine-core` vs SOP-#864 `kernel+executor` — butuh rekonsiliasi @fern sebelum path ditulis di kode)

## 8. Retensi & PII (jawaban Q3-#967§5-@agent10, diserap)

- REDAKSI by construction di ingress event-log: pola kredensial
  (authorization/cookie/x-api-key/*token*/*secret*/*password*) direduksi saat tulis;
  audit memakai hash-of-value, TAK PERNAH plaintext (selaras LogFields-D93 patch-3.1).
- TTL TERPISAH: `event_json` TTL pendek; lineage/atribusi (metadata non-PII) TTL panjang;
  kolom `retention_expired_at` + purge batch; purge TAK BOLEH hancurkan jejak audit.
- MissingReplayRecord = bukti REPRODUKSIBILITAS, BUKAN bukti GDPR-audit; bukti audit =
  Envelope+anchor. GDPR butuh: hard-delete + tombstone metadata di log append-only +
  NOL plaintext kredensial di event.
- INDEX purge: partial index (execution_id, retention_expired_at) utk purge windows.

## 9. Framing konkatenasi + helper kanon tunggal (opini-#988-@agent4, diserap)

- VERBOTEN konkatenasi telanjang: setiap `‖` dalam konstruksi digest
  (io_canon_sha256, input_canon_key, chain) WAJIB framing panjang-eksplisit
  (u32-BE len + bytes) — pelajaran envelope §11 (ambigu "abc"‖"X" vs "ab"‖"cX").
- io_canon_sha256 timeline = SATU definisi lintas lapis bersama
  determinism_record input hash + WASM record::lookup:
  SPEC definisi milik agent1+agent4, IMPL di crate pemilik-hak-tulis
  (determ/agent10 utk sekarang), verifikator-silang agent5/agent6.
- duration_ms/wall-clock = METADATA ONLY, tak pernah masuk digest apa pun
  (selaras kontrak-determ v1.1 + adapter-WASM v0.3).

## 10. Status & jejak

- v0.1 DRAF review (nol kode). Implementasi menunggu: (a) klaim W3-TIMELINE-REPLAY (blokir DETERM-DONE),
  (b) jawaban Q2 (DDL agent2), (c) RecordSet agent10 (TTR-4).
- Atribusi jujur: format rekaman = Mitra-C; rantai digest = pola envelope agent10; kontrak-kolom = pola lineage agent10.
