# n8n-rust v.1

Port Rust dari n8n. Target: **sefleksibel mungkin, ringan, cepat, efisien** —
dengan tampilan **hampir sama seperti n8n asli**.

Scope v1 (diputuskan 2026-09-10): **kompatibel format n8n** — impor/ekspor
workflow JSON format n8n asli + subset node yang bertambah bertahap
(selaras K-3 opsi B: kompatibel format, bukan paritas penuh).

## Prinsip desain

- **Fleksibel** — node = trait + registry dinamis; parameter schemaless
  (`serde_json::Value`) seperti n8n; field JSON asing ditampung, tidak dibuang.
- **Ringan** — dependensi minimal (`serde`, `serde_json`); engine sinkron tanpa
  runtime async; CLI tanpa framework argumen.
- **Kompatibel** — tipe JSON mengikuti struktur ekspor n8n
  (`nodes[]`, `connections{}`, `type`, `typeVersion`, `position`, ...).

## Struktur

```text
n8n-rust/
├── Cargo.toml            # workspace
├── fixtures/             # workflow JSON format-n8n untuk test & contoh
├── crates/
│   ├── n8n-core/         # tipe Workflow/Node/Connections + serde roundtrip
│   ├── n8n-engine/       # trait Node, Registry, executor topo-order
│   ├── n8n-nodes/        # node bawaan: manualTrigger, set (subset), noOp
│   └── n8n-cli/          # validate / run / nodes
└── web/                  # UI (status: BELUM diputuskan — baca web/README.md)
```

## Quickstart

```bash
cd n8n-rust
cargo test
cargo run -p n8n-cli -- validate fixtures/manual-to-set.json
cargo run -p n8n-cli -- run fixtures/manual-to-set.json
cargo run -p n8n-cli -- nodes
```

## Status jujur (skeleton 0.1.0)

Yang sudah ada: parse + roundtrip format n8n, eksekutor topo-order
deterministik, 3 node bawaan, CLI. Yang BELUM: ekspresi n8n (`={{ }}`),
pass-through node disabled, node HTTP/Function, server/API, UI.
Lihat `web/README.md` untuk keputusan UI yang masih terbuka.
