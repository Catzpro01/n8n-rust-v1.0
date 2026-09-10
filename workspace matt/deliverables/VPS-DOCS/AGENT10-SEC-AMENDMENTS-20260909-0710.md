# AGENT10 — AMANDEMEN GATE KEAMANAN (Plt. Security Gatekeeper)
## Kontrak pengesahan per mandat fern #617/#621 — 2026-09-09 07:10 UTC

| | |
|---|---|
| **Otoritas** | fern (Chief Supervisor): #617 delegasi ROLE_SECURITY_QA → agent10 (Plt. Security Gatekeeper, didampingi agent3); #621 pengesahan gate + instruksi "mutasikan ke matriks QA/Security spec". |
| **Status** | DRAF MUTASI — dokumen ini memuat teks gate yang **telah disahkan**; file `AGENT5_QA_SECURITY_SPEC.md` (milik agent5) **tidak saya sentuh** (freeze #386 + prinsip kepemilikan; izin tulis disk juga tidak ada). Mekanisme pemasukan ke spec resmi menunggu keputusan fern/agent5 bila ingin di-merge (lihat §5). |
| **Induk** | AGENT10-COMPLIANCE-CRYPTO-AUDIT.md v1.2 (C-01…C-09, §2.11, §2.12), AGENT10-EXEC-ENVELOPE-SPEC.md v1.0-PREP+consensus |

---

## 0. Ringkasan keputusan yang disahkan

| ID | Keputusan | Disahkan | Sumber |
|---|---|---|---|
| 1a | CI verifikasi jangkar saat fetch template dari Hub | **gate SEC-HUB-01** | fern #621 butir 1a |
| 1b | Verifikasi-ulang saat first-execution | **kontrak engine** (verify-or-refuse; uji eksekusi), **bukan** gate terpisah | fern #621 #621, saran agent1 #619 |
| 2 | G-C6 = KAT keyed BLAKE3 + `derive_key` 2-argumen + `chain_key` eksplisit | **disahkan** | fern #621 butir 2 |
| 3 | Dual-key overlap rotasi | **gate SEC-TRUST-01** | fern #621 butir 3 (usul agent3 #605) |
| — | SEC-08 kriteria lulus diperluas (rewrite konsisten + jendela UNANCHORED) | **amandemen wajib** per C-01 (terbukti empiris §2.11) | agent10 audit; G-C1 |

## 1. AMANDEMEN SEC-08 (usul penggantian/penambahan kriteria lulus)

Konteks asli (agent5): "(a) hapus satu baris di tengah → verifikasi hash chain gagal; (b) akses kredensial selalu tercatat walau gagal; (c) log tidak memuat nilai kredensial."

**Kekurangan (C-01):** (a) menguji penyerang yang lebih lemah dari model ancaman PRD-2 §11.1 (root lokal). Penyerang yang dapat **mengubah baris ke-k lalu merekomputasi h_k…h_n** menghasilkan rantai yang sepenuhnya konsisten; tanpa jangkar eksternal, verifier tidak bisa membedakan riwayat asli vs riwayat yang ditulis ulang dengan rapi. Terbukti empiris (audit §2.11 baris 11–12): rewrite + recompute → verifikasi internal `true`; verifier hanya bisa tahu bila ada salinan root dari luar.

**Teks tambahan (wajib):**
- (d) ubah 1 entri tengah lalu **rekomputasi seluruh chain + root** → verifikasi tetap **GAGAL**. (Runtime/CI = gate **G-C1**; kriteria ini TIDAK ada di SEC-08 asli — gate lama lolos justru pada ancaman yang sama.)
- (e) tanpa jangkar eksternal (checkpoint-root + anchor WORM/TSA, K=N=1.000) → verifier WAJIB mengembalikan `VERIFIED_UNANCHORED` untuk ekor sejak anchor terakhir; **tidak pernah** "VERIFIED" polos.
- Konstruksi lengkap: `AGENT10-EXEC-ENVELOPE-SPEC.md` §3 (keyed chain, `derive_key(CTX, chain_key)` 2-arg — KOREKSI-1) + §5 (checkpoint/anchor).

## 2. SEC-HUB-01 (gate baru — disahkan #621 butir 1a)

**Masalah:** `template_sha256` (manifest v0.4:30) membuktikan file cocok dengan nilai yang dibaca dari **manifest yang sama**; penyerang yang bisa menulis keduanya = total-rewrite tak terdeteksi (kelas C-01 skala kecil, H-02).

**Kriteria lulus (masing-masing wajib):**
1. Fetch → `template_sha256` mismatch → **tolak + alarm** (deteksi korupsi; TOFU eksplisit, pernyataan agent4 v0.4 §2 diterima).
2. Sumber pinned (`expected_sha256`): nilai **diverifikasi terhadap KATALOG agent9 (`hub://intel-types`, single source of truth)** — nilai pin di dalam manifest **bukan** jangkar; mismatch → tolak + alarm; versi lama tetap live (0 mati senyap).
3. Sumber tanpa entri katalog (atau pinned tanpa jangkar katalog) → status **`VERIFIED_TOFU`** eksplisit — tidak pernah `VERIFIED_ANCHORED`.
4. `review_status != "live"` → tolak install (gate pipeline review sebelum live, agent4 v0.4).
5. Post-v1 (skema naik): `manifest_sig` Ed25519 + `kid` + `trust_anchor` di registri agent7; kanon `{schema_version, hub_id, template_sha256, content_version} ‖ BLAKE3(sorted(deviation_ids))` (usul agent3 #605); gagal verifikasi → tolak install.
6. Post-v1: verifikasi ulang saat **first-execution** — **kontrak engine** verify-or-refuse (agent1 #603/#619), diuji dengan eksekusi nyata, bukan gate CI terpisah.
7. Signature tanpa `kid` tak dikenal → tolak.

**Catatan posisi terhadap v0.4:** agent4 memilih jangkar v1 = pin `expected_sha256` per-sumber + pipeline review, Ed25519 = OPEN post-v1. Saya **terima** selama dua syarat dipenuhi: (a) pin diverifikasi terhadap **katalog** agent9 (bukan dibaca dari manifest yang sama), dan (b) status yang dilaporkan jujur (`VERIFIED_TOFU`, bukan "terverifikasi"). Tanpa (a), "pin eksternal" adalah TOFU dengan nama lain — dan itu tidak memenuhi SEC-HUB-01 butir 2. Konfirmasi agent4 + agent9 atas butir (a) diminta.

## 3. SEC-TRUST-01 (gate baru — disahkan #621 butir 3)

**Kriteria lulus:**
- (a) sign dengan kid lama selama overlap → verifikasi LULUS, pubkey terpetakan dari registri;
- (b) setelah overlap berakhir → kid lama DITOLAK;
- (c) `revoked` → DITOLAK seketika;
- (d) tanpa `kid` / `kid` tak dikenal → DITOLAK;
- (e) klien tanpa fetch registri ulang → gagal verifikasi (tidak pernah memakai cache basi diam-diam).

**Kontrak:** `kid` + `valid_from` + `valid_until` + `revoked` di registri (agent7 untuk `hub_signing_key`; `enc_key_version` untuk storage per SEC-02). Rotasi = dual-key overlap.

## 4. SEC-TRANSIENT (angka terukur; gerbang L2 PRD-3)

- `size_of::<blake3::Hasher>()` = **1.920 B** (blake3 1.8.7, rustc 1.98.1; §2.11 baris 3), `sha2::Sha256` = 112 B, `h_prev`/`ckey` = 32 B masing-masing.
- Aturan: **satu** hasher dipakai ulang berurutan (reset, bukan instansiasi baru); dua hasher paralel = 3.840 B = 96 % batas 4 KB → dilarang tanpa persetujuan fern/agent8.
- Anggaran dilaporkan: **64 B persistent + 1.920 B transien/eksekusi** — bukan "32 byte" (#327).
- Gate: (a) test `size_of` menyertakan versi crate; (b) 2 eksekusi paralel → ≤ 4 KB/eksekusi; (c) reset vs instansiasi baru → digest identik.

## 5. L2-READINESS CHECK (perintah fern #637 poin 5: "standby verifikasi L2") — 07:15 UTC

Gerbang L2 PRD-3 v3.0.0-FINAL berisi 3 klausul. Statusnya per dokumen yang terbit saat cek (semua [TERBUKTI-STATIS]):

| Klausul L2 | Teks | Status verifikasi | Bukti |
|---|---|---|---|
| (1) `SEC-WCB-01..04` | "Eksekusi WASM community node terisolasi ketat dalam linear memory 32MB" | **TERDEFINISI** — gate lengkap + manifest_sig Ed25519 + TOCTOU dua-tahap + trust anchor registri agent7 | AGENT6-WCB-SPEC v0.3 (sha dc7e7ec5…, #629); AGENT6-PRD3-WCB-INPUT (sha c3886341…) |
| (2) `SEC-TRANSIENT` | "BLAKE3 sequential hasher mematuhi batas transien 4KB" | **LULUS level desain** — `size_of::<blake3::Hasher>()` = 1.920 B ≤ 4 KB (terukur, §2.11 audit baris 3); aturan **satu hasher** (2 hasher = 3.840 B = 96 % batas → dilarang tanpa persetujuan) | AGENT10-EXEC-ENVELOPE-SPEC §4; audit §2.11 |
| (3) G-C6 keyed derive_key | "Keyed derive_key untuk Execution Envelope dan audit trail (G-C6)" | **P3-01 BLOCKER (label)** — yang disahkan #621 butir 2: KAT keyed BLAKE3 + `derive_key` 2-arg + `chain_key` eksplisit = **G-C6**; keyed chain rewrite-detected = **G-C1**. PRD-3 L2 menulis "G-C6" di konteks keyed chain → label salah. Koreksi teks diusulkan #639 poin 4 (milik fern; tidak saya edit) | audit P3-01 (AGENT10-PRD3-COMPLIANCE-REVIEW.md); #639 |

**Kesimpulan:** L2 siap diuji begitu implementasi dimulai (pasca sign-off). Tidak ada blocker implementatif dari sisi gate definition; satu label (3) menunggu koreksi editorial fern. Tidak ada uji runtime yang bisa dijalankan pra-sign-off (zero-code).

## 6. PUTUSAN GATEKEEPER — OPEN-INTEG-11 (kebijakan pin katalog agent9) — 07:45 UTC

Masalah: `hub-intel-types.json` (agent9, 07:23, 16 sumber) belum punya field pin; `pin_policy = "BELUM DIPASANG — OPEN-INTEG-11 (butuh keputusan agent4+agent10; jangan TOFU-palsu)"`. 16/16 status `verified-live` (sebagian `verified-live-teknis (BUKAN approved-etika)`).

**KEPUTUSAN agent10 (Plt. Security Gatekeeper) — dua kelas pin, bukan satu kebijakan untuk semua:**
1. **Sumber STATIC** (file/dataset versi, konten immutabel — mis. arsip paket, JSON statis ber-versi): `expected_sha256` **WAJIB** di katalog (field nullable); kalau tidak ada → status `VERIFIED_TOFU` eksplisit, bukan `ANCHORED`.
2. **Sumber VOLATILE API** (16/16 yang ada sekarang — harga, RSS, ticker): pin **TIDAK diwajibkan** (nilai berubah per menit; pin = alarm palsu permanen). Jangkar mereka = **katalog + ETag/cache + record-replay** (W2-DETERM-ENFORCE). Status jujur = `verified-live` → dibaca sebagai **`VERIFIED_TOFU`** (klaim: "terlihat hidup dari VPS pada tanggal T", BUKAN "asli dari penerbit").
3. **Gate etika:** sumber berstatus `verified-live-teknis (BUKAN approved-etika)` **TIDAK BOLEH** muncul sebagai sumber template LIVE sampai gate etika disetujui — sebagai Plt. gatekeeper, saya set status perantara `etika-pending` (dapat dipakai untuk pengujian internal, dilarang untuk rilis/live).
4. **Field penambahan yang diminta:** katalog menambah `pin: {status: "required|not-applicable", sha256: null|hex}` per sumber dan `verify.status` menambah opsi `verified-tofu` (teruji live, tanpa pin) — konsisten SEC-HUB-01 butir 2–3.
5. Konsekuensi bagi v1: **semua** template Hub dari sumber volatile kini berstatus TOFU; tidak ada klaim keaslian; `review_status=live` HANYA berarti lolos review pipeline, bukan "sumber asli".

## 7. REVIEW PRD-3 v3.2 (matt, 07:23) — verifikasi P3-01…P3-05 sebagai gatekeeper

| Temuan | Status di v3.2 (verifikasi baris langsung) | Verdict |
|---|---|---|
| P3-01 (label G-C6) | **DIPERBAIKI** — G-C1a (hapus entri), G-C1b (ubah+rekomputasi = GAGAL — cacat lolos), G-C6 (KAT) dipisah; "satu gate gabungan bernama G-C6 akan menjadi... yang lolos" | ✅ CLOSED (v3.2 baris 257–263) |
| P3-02 (lineage hilang) | **DIPERBAIKI** — `W3-ITEM-LINEAGE` ditambahkan (baris 190–193; fern #400/#500 + agent1 #522) | ✅ CLOSED (tunggu queue) |
| P3-03 (kepemilikan ganda) | **DIUSULKAN SELESAI** — matt dukung "konstruksi crypto = agent10" (baris 209–212); tunggu ratifikasi fern | ⏳ |
| P3-04 (SEC-08 stale) | Masih menunggu agent5 (text spec 06:54 belum diganti); amandemen saya siap potong-tempel (§1) | ⏳ eksternal |
| P3-05 (i18n 500KB) | Bukan ranah gate crypto; tidak saya tangani | — |
| ADDENDUM-A1 | **DISERAP** — urutan lapisan benar (A2 anchor = menjamin; keyed = menaikkan biaya) + W0-ANCHOR-SPEC (baris 161) | ✅ selaras |
| SEC-TRANSIENT | "1 hasher = 47 % budget" (baris 127, #646) — konsisten angka 1.920 B/4.096 B = 46,9 % | ✅ |

Catatan: v3.2 = draft matt; status FINAL menunggu pengesahan fern/pemilik. Tidak ada temuan keamanan baru dari v3.2 yang berubah: 0 regresi vs v3.0.

## 8. Proses & batas

- **Yang saya lakukan:** mencatat pengesahan + teks gate di dokumen ini; mengingatkan di kanal; **tidak mengedit** `AGENT5_QA_SECURITY_SPEC.md` (milik agent5; freeze; tanpa izin tulis).
- **Yang diminta:** (1) fern/agent5 memutuskan mekanisme pemasukan — (i) agent10 menerima izin tulis eksplisit untuk spec tersebut, atau (ii) agent5 memasukkan saat sesi pulih (teks siap potong-tempel di dokumen ini), atau (iii) spec baru milik bersama (mis. `SEC-GATES-REGISTRY.md`) menjadi kanon sementara. (2) Konfirmasi agent4+agent9 atas SEC-HUB-01 butir 2 (pin dari katalog).
- Amandemen ini **nol kode**, nol perubahan file milik agen lain — patuh #386/#586.

— **agent10**, Plt. Security Gatekeeper (mandat #617/#621; peran asli ROLE_COMPLIANCE)

---

## 9. KONTRAK ERROR SPILL — E_SPILL_CORRUPT (08:02 — naik kelas dari kosmetik ke kebutuhan kontrak, #747 agent1, dikonfirmasi agent10 gatekeeper)

Verdict: **SETUJU — naik-kelas.** Varian error generik memaksa engine menebak sebab dari string pesan = rapuh dan tidak falsifiable. Kontrak error (storage↔engine) ditetapkan:

| Kondisi | Varian error (Rust) | Kode eksternal | Perilaku engine (kontrak) |
|---|---|---|---|
| Magic salah | `SpillError::InvalidMagic` | E_SPILL_CORRUPT | eksekusi → status CORRUPTED; jangan coba baca ulang |
| Checksum mismatch | `SpillError::ChecksumMismatch` | E_SPILL_CORRUPT | eksekusi → status CORRUPTED; retensi blob + alarm (bukan senyap) |
| Offset di luar jangkauan | `SpillError::OffsetOutOfRange` | E_SPILL_CORRUPT | sama |
| IO/izin | `SpillError::SpillIo` | E_SPILL_IO | jalur error berbeda (operasional, bukan korupsi) |

Gate: G-C7 ditambah butir — uji korupsi 1 byte mengembalikan `ChecksumMismatch` (bukan `Invalid` generik) dan kode `E_SPILL_CORRUPT`; uji magic salah → `InvalidMagic`. Ini kontrak, bukan saran.

---

## §10 — W0-HASH-FIX: hasil & status gate (agent10, sesi B, 2026-09-09)

- **Konteks:** audit C-05 — `testkit/src/lib.rs` berisi `blake3_hash()` yang isinya `std::hash::DefaultHasher` (SipHash-1-3 64-bit, non-kriptografis, 16-hex, tak stabil). Nama bohong terhadap isi → G-C5 merah, KAT tidak ada → G-C6 merah, divergen dengan kontrak data-plane SHA-256 64-hex → G-C7 merah.
- **Perbaikan (landing = `rust-engine/crates/testkit`, satu-satunya pohon yang memuat testkit di server):** `sha256_checksum()` = SHA-256 via `sha2 = "=0.10.8"` (pin workspace), `format!("{:x}")` 64-hex-lowercase = identik `data-plane/src/spill.rs` (G-C7 kurva). Call-site InMemorySpillStore::write diperbarui. Cargo.lock tidak berubah.
- **Uji (rustc 1.98.1, jalur murni):** 19/19 lolos — KAT `""` → e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855; KAT `"abc"` → ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad; format 64-lowercase; injektivitas; + semua uji lama (spill checksum-mismatch, GC, no-leaks, mock suite).
- **Gate:** G-C5 ✅ (scan 0 hit), G-C6 ✅ (KAT vektor FIPS 180-4), G-C7 ⏳ SEBAGIAN (algoritma+format identik; komparasi lintas-crate dialokasikan ke W0-SPILL-TEST).
- **Catatan:** `swarm-task complete W0-HASH-FIX` → DONE; artefak `/opt/agent-workspace/docs/AGENT10-W0-HASH-FIX-REPORT.md`; laporan kanal **#771** (termasuk tanggapan C-5 #764: fakta pohon — testkit hanya di rust-engine; opsi kebijakan (a)/(b) diserahkan ke fern/matt). Warning pre-existing `unused ctx` @lib.rs:347 bukan dari patch ini.

---

## §11 — Sesi sinkron #867/#872 (SOP Zero-Collision, BAHASA-BERSAMA, Mitra E) — agent10, 2026-09-09

- **Dekrit #864 (SOP 5 pilar) + #865 (pengesahan matt) + #867 (distribusi eksekutif) + #872 (Mitra E + BAHASA-BERSAMA)** diadopsi penuh. Poin kunci saya: (a) klaim atomik sebelum kode — TERBUKTI ditegakkan sistem: `claim W3-EXEC-ENVELOPE` ditolak "Kuota tugas aktif (Max 1 Active Task)" selagi W2-DETERM-ENFORCE aktif; (b) path ownership: `crates/auth/` & `envelope/` = eksklusif agent10; (c) shadow worktree + merge satu pintu (matt, syarat #865: test hijau + clippy 0 + review independen); (d) Mitra E = @agent10 & @matt (audit hash-chain, gate CI, merge kanonik).
- **W2-DETERM-ENFORCE → DIKLAIM (agent10)** dan implementor-phase SELESAI: crate `determ` di `/opt/agent-workspace/w2-determ-enforce` (RecordSet v1 BE-kanonik, EntryKind 5 kind, `redact_headers` G-D7, `body_hash` CTX_BODY, `recordset_hash`=BLAKE3(CTX_RECORD‖prefix), `recordset_sha256`); **cargo test --release = 9/9 (G-D1..D8 + KAT BLAKE3/SHA-256); clippy 0 warning; forbid(unsafe)**. Keputusan kontrak dinyatakan: D-D1 dual YA, D-D2 FAIL, D-D3 N=300 s, D-D4 replay≠audit. HANDOFF-READY dikirim (#896): @agent4 konfirmasi kolom determinism_record, @matt review.
- **W3-EXEC-ENVELOPE (sesi A, crate `w3-exec-envelope`)**: saya verifikasi ulang 21/21 + clippy 0 + E2E TSA nyata 10/10 (C-01 tertutup; head bed7f5021…). Status: klaim tertahan kuota Max-1; akan claim+complete+HANDOFF @matt setelah DETERM DONE (merge per #863 pasca W0-SPILL-TEST).
- **Review W0-ANCHOR v1.1 (V11.patch, agent1 #869) = PASS**: A4/D-A5 sesuai spec v1.1-C (seed-bootstrap --authorized-by exit-2 pra-row; rotasi --authorized-by+reason; verify last-pin-row + --expected-fp out-of-band; ANC-8e anti-vacuous). Batas jujur diterima (lookup historis pasca-rotasi = wave berikut).
- **W0-SPILL-TEST** (agent1): 11/12 — F1 Wajib (drop-writer leak .spill.tmp; impl-Drop agent2) → 12/12 → Wave 0 100%.
- **BAHASA-BERSAMA diadopsi** dgn catatan C-02: blake3_hex = digest; SHA-256 hex = checksum berkas (G-C7); jangan dirancukan.
- **Kunci SSH**: permission 0644 (restore snapshot) pernah menolak koneksi → chmod 600; beres.

---

## §12 — Sprint #867/#872/#903: 3 tugas tuntas + ruling §3.1 + identitas (agent10, 11:1x)

- **W2-DETERM-ENFORCE = DONE** (crate `determ`, 9/9 G-D1..D8+KAT, clippy 0; laporan docs/AGENT10-W2-DETERM-ENFORCE-REPORT.md; D-D1..D4: dual hash YA / FAIL fail-closed / N=300 s / replay≠audit).
- **W3-EXEC-ENVELOPE = DONE** (21/21 + clippy 0 + E2E 10/10 FreeTSA; C-01 tertutup; laporan AGENT10-W3-EXEC-ENVELOPE-REPORT.md; menunggu reviewer independen + KONFLIK-1 #917 + merge matt).
- **W0-CONTEXT-CONTRACT = DONE** (34+19 uji hijau, clippy 0; CT-01..08; artefak w0-context-contract/ = context_contract.rs + context-3.1.patch; laporan AGENT10-W0-CONTEXT-CONTRACT-REPORT.md).
- **RULING §3.1 = opsi (a)** (keputusan #912 fern+agent10): Logger → `&LogFields` (put/put_credential; redaksi by construction); `CredentialValue::as_value()` dihapus → `reveal()` doc(hidden) trusted D92; precedent D94 (:351-355 ExpressionScope); biaya NOL (hanya NoopLogger). Patch 77 baris → @agent1 aplikasi, @matt merge. CT-05a/05b ditahan (butuh varian ResourceExhausted/Timeout — usulan nama tertulis).
- **Identitas:** klaim 10:50:31 W2-DETERM = sesi paralel agent10 sendiri (dual-session #628), BUKAN pihak ketiga; #967 menjelaskan + rekomendasi tanda sesi pada pesan & keputusan hanya sah dari akun berakses VPS (tinjau F.3 matt). #863/#865 ditandai ORPHANED (bukan sumber otoritas; substansi SOP diakui matte di #912).
- **HUB-7 UNVERIFIED** (audit saya membuktikan HUB-6 20/20; HUB-7 note-binding = belum ada artefak/spesifikasi — C-9).
- **W3-ITEM-LINEAGE = DIKLAIM agent10** (required_role ROLE_COMPLIANCE; fallback ROLE_STORAGE/QA/CORE → #903 delegasi agent2 tak dapat diklaim ROLE_STORAGE; solusi: saya implementor, @agent2 reviewer storage/DDL Pilar 5). Rencana: spec v0.1 + shadow + patch.
- **W3-TIMELINE-REPLAY #910 Q3**: redaksi by construction + TTL eksplisit (event_json pendek; lineage metadata panjang) + fail-closed ≠ bukti GDPR-audit (hanya Envelope+anchor); FK: dukung #921 + partial index purge + blake3-hex cross-link (bukan SHA-256) + seq-range.
- Pesan: #967 (konsolidasi) + #tech-debate TOPIK-3.
