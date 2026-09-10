# UI n8n-rust — DIPUTUSKAN: Opsi A

## Keputusan (2026-09-10; syarat user: kemiripan 95% dengan n8n asli)

**Opsi A: Vue 3 + VueFlow + backend Rust (Axum).**

Alasan: satu-satunya stack yang realistis mencapai 95% — editor n8n asli
berbasis Vue, dan VueFlow adalah primitif kanvas terdekat. Opsi full-Rust
(Leptos/Dioxus) menuntut penulisan ulang seluruh komponen visual → tidak
mencapai 95% dalam effort wajar.

## Arti "95% sama" (operasional, bukan slogan)

- Layout: kanvas node tengah + panel properti kanan + toolbar/eksekusi — sama.
- Node cards: ikon, nama, status eksekusi, badge error — sama.
- Interaksi inti: drag-node, connect-edge, klik-untuk-konfigurasi,
  eksekusi-manual + highlight jalur — sama.
- Yang BOLEH beda (5%): branding/logo, tema warna minor, menu lanjutan
  (admin, user-management, template store) — di luar v1.

## Terbuka (calon tiket)

1. Protokol UI↔engine (REST + JSON workflow; WebSocket untuk eksekusi live).
2. Crate server Axum: serve workflow JSON + endpoint run + serve build web/.
3. Impor `.json` n8n langsung di kanvas.
4. Scaffold web/ (Vite + Vue + VueFlow) — BUTUH akses npm registry
   (terblokir di sandbox ini; jalankan lokal/VPS).

Kandidat yang ditolak: B (Leptos), C (Dioxus) — alasan di atas.
