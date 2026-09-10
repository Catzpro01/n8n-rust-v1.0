# R-6 SLICE-1 — Kindmap: tipe n8n → NodeKind + version u16 (crates/rosetta) — FINAL (RATIFIED)

**2026-09-09 · agent4 · Task: RULING 30 matt (#1188) + otorisasi fern (#1206/#1207). Status: SLICE-1 RATIFIED-FINAL · R-6 IN_PROGRESS (slice-3 ParameterSchema menunggu keputusan matt/fern).**

## Ratifikasi
- **agent1 #1241**: RATIFIED-WITH-6-ROW-CORRECTIONS — mekanis OK (147 baris, sorted-unik, API fail-loud, version_u16 checked, sortedness test, error-vacuous disetujui, R-28 dicatat). REPRO 53/53 terhalang env agent1 (registry offline) — diakui jujur, bukti agent4 berdiri di env-nya.
- **matt RULING 35 (#1249)**: 6 koreksi agent1 DISAHKAN + KEEP splitInBatches. Prinsip: **TITIK MENYATAKAN HIERARKI, BUKAN PEMISAH KATA**.
- 6 koreksi DITERAPKAN di kode: `mailerLite→mailerlite`, `sendInBlue→sendinblue`, `nextCloud→nextcloud`, `nocoDb→nocodb`, `wooCommerce→woocommerce`, `wooCommerceTool→woocommerce.tool`.

## Kelas penamaan (tercatat di doc modul kindmap.rs, RULING 35)
1. **vendor.product** (hierarki nyata): google.sheets, aws.s3, aws.rekognition, microsoft.excel, http.request.
2. **brand token-tunggal** (satu merek, digabung): mongodb, mailerlite, sendinblue, nextcloud, nocodb, woocommerce, linkedin, whatsapp, youtube, sendgrid, highlevel, noop, n8n.
3. **split deskriptif KEEP** (agent1 #1241): hacker.news, one.simple.api, ai.transform, date.time (+*Tool).
4. **node inti n8n — nama upstream dipertahankan** (RULING 35): splitInBatches → splitInBatches. ANGGOTA KELAS, bukan pengecualian tunggal; node inti camelCase baru masuk kelas ini tanpa keputusan baru.

## Bukti final
- 53/53 test hijau (36 lib + 2 binding + 4 corpus_m1 + 4 corpus_m2 + 3 kanon_corpus + 2 kindmap_corpus + 2 manifest_wcb), clippy --all-targets 0.
- Gate korpus: 171/171 file, SEMUA node base terpetakan; tipe base unik 147 == tabel; non-base → NonBase (opaque R-2).
- tree@sha `158df21d` (src .rs + Cargo.toml; fixtures tak di-hash).
- Verifikasi: `cd /opt/agent-workspace/rust-engine && CARGO_TARGET_DIR=/mnt/extra-storage/cargo-target-rosetta cargo test -p rosetta`

## Konteks terkait
- P1 TypeVersion (#1188) TERTUTUP: pengukuran korpus minor maks = 9, desimal 2 digit = 0; fail-closed (47/47 → 53/53, #1228). Diffgate-L3 tidak lagi terblokir sisi TypeVersion.
- Konvergensi formula RULING 31: komitmen 1-helper, tanpa rewrite mandiri (#1227); namespace saya = `determ.record`.
- RULING 28: numeric_id!→INTEGER, NodeId/NodeKind→TEXT — dicatat utk konsumen skema.

## Slice berikutnya
- Slice-3 (ParameterSchema per tipe dgn bukti validate thd korpus / verifikasi upstream) — menunggu keputusan @matt/@fern: lanjut sekarang atau nanti.
