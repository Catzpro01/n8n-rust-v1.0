# ADR-0007: Custom node — 8 tier (keputusan Pemilik: "3 tier kurang, mau lebih")

**Status:** DITERIMA — daftar final 8 tier dikonfirmasi Pemilik ronde 2 ("setujui 8nya"), 2026-09-10.

| # | Tier | Medium | Build ulang? | Sandbox? | Posisi |
|---|---|---|---|---|---|
| 1 | **Script Node** | JS tertanam (rquickjs) | tidak | ya (isolasi V8-like, tanpa akses sistem) | paling mudah |
| 2 | **Python Node** | Python tertanam (PyO3) | tidak | ya (resource limit) | reuse skrip Python |
| 3 | **WASIM → WASM Node** | Rust/JS/→wasm | tidak (hot-load) | **ya, kuat** (wasmtime) | tier serius; crate `nodes-wasm` sudah ada |
| 4 | **Native Rust crate** | implement trait `Node` | ya (compile) | tidak (dipercaya) | tercepat, beban superberat |
| 5 | **External Process/Microservice** | bahasa apa pun | tidak | ya (proses terpisah, IPC lokal) | node legacy/bahasa apa pun |
| 6 | **AI-Generated Node** | deskripsi bahasa natural → engine tulis kode + test + paketkan | sesuai hasil | sesuai hasil | "node instan dari kalimat" |
| 7 | **Sub-Workflow Node** | workflow lain sebagai node | tidak | ya (budget diwarisi) | komposisi (kebutuhan Pemilik eksplisit) |
| 8 | **MCP Tool Node** | tool server MCP apa pun | tidak | ya (MCP = protokol terdefinisi) | ekosistem MCP = node gratis |

Ketentuan lintas tier: setiap tier melewati **budget guard** (CPU/RSS/waktu per
eksekusi), **kontrak input/output item** yang sama, dan **satu contoh jadi** di repo
supaya "mudah" itu terbuktikan, bukan diklaim.
