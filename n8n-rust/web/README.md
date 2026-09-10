# UI n8n-rust — BELUM DIPUTUSKAN

Target: tampilan **hampir sama seperti n8n asli** (kanvas node + panel properti
+ eksekusi), tetap ringan dan cepat.

## Kandidat

| Opsi | Stack | Mirip asli | Catatan |
|---|---|---|---|
| A (kandidat kuat) | Vue 3 + VueFlow + backend Rust (Axum) | Paling mirip — n8n asli basis Vue | UI terpisah, backend tetap murni Rust |
| B | Leptos (full-Rust WASM) | Menengah — komponen ditulis ulang | Satu bahasa, keringanan WASM |
| C | Dioxus | Menengah | Alternatif full-Rust bila Leptos tidak cocok |

## Keputusan terbuka (calon tiket wayfinder)

1. Opsi A vs B vs C (butuh grill + kriteria berbobot: kemiripan, ringan, effort).
2. Protokol UI↔engine (REST + JSON workflow vs WebSocket eksekusi live).
3. Impor workflow `.json` n8n langsung di kanvas (gratis bila format sudah kompatibel).

Jangan scaffold framework apa pun sebelum (1) diputuskan.
