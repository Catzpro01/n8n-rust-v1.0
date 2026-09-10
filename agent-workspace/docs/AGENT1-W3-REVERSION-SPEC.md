# W3-TIMELINE-REPLAY -- Reversion-side specification (agent1) -- v3.2

Status: SPEC v3.2 (v3.1 + status-verifikasi-3-nilai + Q-cleanRunData). Task W3-TIMELINE-REPLAY P0 IN_PROGRESS agent1.
Lane: 53e(b)/54f -- query-layer (agent10, W3-REPLAY-QUERY-LAYER P0) = prerequisite; this spec = execution-layer (reversion).
Hard condition: W3 consumes the read layer; no second reconstruction; differing shape needs = change request.

## 1. Objective
Reversion Engine: given execution_id + target T (attempt-boundary, seq), restore engine-managed state to T -- or refuse with visible reason (fail-closed). Read-only replay/reconstruction is NOT in W3 scope.

## 2. Non-objectives
- Order reconstruction (query-layer).
- Un-erase: erasure_log append-only; reversion NEVER resurrects erased material (53 S6.2 fail-closed).
- Audit proof: replay is not evidence (W2 D-D4: Envelope+anchor only).
- Kompensasi (counter-action per node) = NON-GOAL W3 (#1551 S2): n8n tak punya invers per node; paritas mensyaratkan tak-divergen; rollback terbukti via golden. BUKAN follow-up.

## 3. Dependencies (measured 2026-09-09)
DONE: W2-DETERM-ENFORCE (RecordSet v1, replay-miss FAIL D-D2, clock window 300s D-D3, G-D1..D8 green). DONE: W3-ITEM-LINEAGE. DONE: W3-EXEC-ENVELOPE.
DONE-SKEMA: W2-SCHEMA-ATTEMPT-MATERIAL (agent2, d4b52fe): node_output PK kini (execution_id, node_id, output_index, attempt); pola rebuild SQLite; row warisan attempt=0 = penanda-bukan-data.
CLAIMED (build pending): W3-REPLAY-QUERY-LAYER (agent10, P0, #1567 -- INSERT done, deadlock-CLAIM dibuka #1627 S2; agent10 ukur-ulang vs d4b52fe #1617 S6: semua kolom S4 ADA).
MIGRASI SKEMA (#1551 S5 -- TERJAWAB tak-ada, per-tabel, ERA-001): grep ALTER TABLE atas migrations per sql + migration.rs = 0 hit. event_log/task/node_output didefinisikan HANYA di 001 (:88/:99/:134) dan tak pernah disentuh sesudahnya: 002 = 3 CREATE VIEW only (:4/:20/:29, v_execution_detail/v_execution_task_summary/v_orphan_intents); 004 = 4 CREATE TABLE baru only (:12 lineage_edge/:41 erasure_log/:58 lineage_lookup/:80 lineage_ext) + CREATE INDEX; tak ada berkas 003 (skipped, terverifikasi #1601). PENGECUALIAN-BERTANGGAL: 005_add_attempt.sql (d4b52fe, rebuild node_output + attempt) = migrasi PERTAMA yang menyentuh node_output; jawaban era-001 tetap benar untuk masanya.
PRASYARAT TERUKUR: executor = STUB 48-byte (lib.rs:1) -- implementasi reversion juga tergantung semantik executor didefinisikan (S8 F-Q3).

## 4. Query-layer interface requirements (agent10 builds against this)
replay_stream(execution_id, from_seq):
- yields items ordered by seq_start (LIN-1): seq, node_id, attempt, output_index, output_ref, content_hash (R18), data_class, erasure status (present/absent-destroyed_at/indeterminate), material (Item from spill, digest-REVERIFIED, or redacted marker -- never plain PERSONAL/erased).
- re-entrant: multiple passes + seek to seq_start.
- fail-closed: digest mismatch / missing spill / missing lineage_ext row / erasure-indeterminate = visible ERROR, never silent skip.
- output discipline: deterministic bytes; no duration_ms/wall-clock/color in machine output.
- C2 RESOLVED (R56 S1): material attempt-N tak-tersedia --> (a) fail-closed ERROR: material for attempt N of node ID not available (overwritten by attempt M). Reversion to seq T cannot be computed. Zero writes performed. (b) latest+metadata DITOLAK (data salah dengan catatan; fold menelan apa pun). (c) digest-only DIIZINKAN hanya sebagai API TERPISAH bernama-beda untuk konsumen audit; satu fungsi DILARANG mengembalikan digest saat material diminta.
- IDENTITAS MATERIAL (syarat konsumen #1561): material WAJIB selalu membawa identitasnya (attempt-aktual + output_ref + digest + status-verifikasi), tak-pernah-bare-bytes.
- STATUS-VERIFIKASI 3-NILAI (jawab #1617 S6 SETUJU, dukung #1620, ratifikasi fern #1627 S3; konkuren matt pending): VERIFIED (bukti cocok); VERIFICATION_FAILED (bukti tak-cocok --> fail-closed ERROR); NOT_VERIFIABLE_BY_DESIGN (PERSONAL tanpa bukti-polos, desain GDPR --> BUKAN error). Nilai ketiga BOLEH muncul di keluaran --json sebagai metadata klasifikasi (bukan isi data).

## 5. Stream vs materialization -- RATIFIED STREAM (#1551 S0)
STREAM (re-entrant + seekable), bukan materialisasi. Alasan: cap 500MB; fold prefiks satu lintasan; akses-acak-masa-lalu via seek. Counter agent10 wajib berupa kebutuhan reversion konkret yang streaming tak bisa layani; bila tak ada, confirm + bangun sesuai S4.

## 6. Reversion semantics (W3 execution side; lokasi: crates/executor #1551 S4)
- Target T = attempt-boundary. Default assumption: attempt-boundary. Change requires explicit decision + migration. (C1 R56 S3.)
- KUNCI W2 WAJIB (#1551 S1): reversion di bawah mode-deterministik (Recorder/Replayer).
- Replay-miss = FAIL + tolak + nol tulis. UX: ERROR cannot revert to seq N -- recorded determinism data missing for node ID at seq M. Zero writes performed. (Gate R6.)
- Fold prefix via query-layer stream; tulis atomik (all-or-nothing; R3).
- Erasure in range: ERROR, zero writes (R2). Retention-destroyed: PARTIAL + explicit list (R5).
- Repeat: NO-OP + UNCHANGED via state digest (R4).
- 55e-1 LAPORKAN EFEK EKSTERNAL yang ikut di-rollback: WARNING N nodes in reverted range had external side effects that CANNOT be undone (daftar node+seq+jenis). (Gate R7.)
- Lokasi: modul crates/executor (semantik eksekusi milik executor; storage tetap mekanisme). Query-layer tetap crates/storage (APPROVED agent2 #1554, API baca existing cukup).

## 7. Acceptance gates (falsifiable, R27/50e mutants; sha-before != sha-after wajib)
- R1: reversion-to-T restores bytes reconstructed by query-layer (golden byte-identical, deterministic).
- R2: erased material in range --> visible ERROR + zero writes. Mutant: skip erasure check --> plain appears = FAILS.
- R3: digest mismatch mid-reversion --> STOP + no partial state. Mutant: allow partial write --> detected.
- R4: repeat --> UNCHANGED no-op. Mutant: rewrite anyway --> state digest moves = FAILS.
- R5: retention-destroyed in range --> PARTIAL + explicit list.
- R6 (#1551): hilangkan cek replay-miss --> reversion sukses-berstate-menyimpang --> test HARUS gagal.
- R7 (#1551/55e-1): hilangkan daftar efek eksternal --> output tanpa peringatan --> test HARUS gagal.

## 8. Temuan bukti (verified file:line)
F-Q3 (atomisitas entri, joint agent1+agent10 -- evidence posted, agreement pending agent10): event_log (001:88) append-only PK (execution_id, seq), TANPA kolom attempt; attempt di lineage_lookup (004:58, seq_start/seq_end) + task (001:99). executor/src/lib.rs = STUB 48-byte: TAK ADA applier/transisi-state. Pertanyaan atomik-per-entri-vs-per-attempt TAK TERJAWAB dari kode. DEFAULT BERLAKU (attempt-boundary, S6).
F-MAT --> BUG PARITAS (R56 S2 TERJAWAB #1570, VERIFIKASI-INDEPENDEN-agent3-CONFIRMED-#1575, matt-verifikasi-sendiri-R58 S6, DIPERBAIKI agent2 d4b52fe): upstream n8n@2.38.5 fcf21f5efc63 MENYIMPAN output attempt lama -- IRunData array per node (workflow/src/interfaces.ts:3383-3386); upsertTaskData push-or-merge-per-slot tanpa truncate (core/.../workflow-execute.ts:2115-2123); persist seluruh executionData tanpa trim (cli/.../save-execution-progress.ts:24-28) ke TEXT blob (db/.../execution-data.ts:11-12). Maka PK node_output (001:134) tanpa attempt = divergensi paritas --> DIPERBAIKI via 005 (lane @agent2, commit d4b52fe). Row warisan attempt=0 = penanda; (a) fail-closed TETAP wajib selamanya (R58 S5 syarat-1). Clone bukti: /mnt/extra-storage/agent1-work/n8n-upstream @ fcf21f5efc63 (milik agent1, terbaca grup).
Q-CLEANRUNDATA (TERBUKA, R58 S6): bila partial re-run upstream menghapus entri array attempt (cleanRunData), itu JALUR KEDUA menuju material-hilang selain overwrite-retry -- BELUM ditelusur (di luar pertanyaan retry R56 S2). Sampai terbukti sebaliknya, reversion+query-layer berasumsi (a) fail-closed menutup KEDUANYA (material tak-ada --> ERROR, apa pun sebabnya).

-- agent1, 2026-09-09. v3.2: v3.1 + status-3-nilai (#1617 S6 SETUJU, #1627 S3) + Q-cleanRunData (R58 S6) + 005-tercatat + F-MAT-dipenuhi-d4b52fe. v3.1 sha 7afc2945 digantikan.
