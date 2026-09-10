# UI n8n-rust — DIPUTUSKAN (revisi): Rust-served single-file UI

## Keputusan awal (2026-09-10): Opsi A — Vue 3 + VueFlow + Axum

Dipilih karena syarat kemiripan 95%: editor n8n asli berbasis Vue.

## Revisi (2026-09-10): UI single-file embedded di binary Rust

Alasan revisi (syarat user: murni Rust, personal, seefisien mungkin):

- VueFlow butuh npm + toolchain JS + build step — bertentangan dengan
  "satu binary, tanpa install, offline-first".
- Kemiripan visual adalah soal HTML/CSS, bukan framework: kanvas SVG +
  kartu node + panel properti bisa meniru n8n tanpa framework.
- Hasil: `crates/n8n-server/ui/app.html` — satu file, tanpa dependensi
  eksternal, disajikan Rust via `include_str!`. Nol build step.

VueFlow tetap menjadi **upgrade path** masa depan: protokol REST
(`/api/nodes|validate|explain|run`) adalah kontrak stabil, jadi folder
`web/` ini bisa diganti implementasi Vue kapan saja tanpa menyentuh engine.

## Arti "95% sama" (operasional, bukan slogan)

- Layout: kanvas node + panel properti kanan + toolbar — sama.
- Node cards: nama, tipe, status/urutan eksekusi, timing — sama.
- Interaksi inti: drag-node, klik-untuk-detail, Validate/Explain/Run — sama.
- Yang BOLEH beda (5%): connect-edge via drag (v1 baca edge dari JSON),
  branding/logo, tema minor, menu lanjutan di luar v1.

## Terbuka (calon tiket)

1. Connect-edge via drag di kanvas + simpan kembali ke JSON.
2. Edit parameters langsung di panel (saat ini baca-saja).
3. Impor `.json` n8n asli yang besar (uji + virtualisasi bila perlu).
4. Upgrade VueFlow bila kemiripan single-file mentok (kontrak REST siap).
