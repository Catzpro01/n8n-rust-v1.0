# n8n-rust v.1

Port Rust dari n8n. Target: **sefleksibel mungkin, ringan, cepat, efisien** —
dengan tampilan **hampir sama seperti n8n asli** (syarat: 95%).

Scope: **kompatibel format n8n** — impor/ekspor workflow JSON format n8n asli
+ subset node yang bertambah bertahap.

## Prinsip desain

- **Fleksibel** — node = trait + registry dinamis; parameter schemaless;
  field JSON asing ditampung, tidak dibuang; ekspresi `={{ }}` per item.
- **Ringan** — core/engine/nodes/cli cuma butuh `serde` + `serde_json`;
  engine sinkron tanpa runtime async; CLI tanpa framework argumen.
- **Kompatibel** — tipe JSON mengikuti struktur ekspor n8n; nama tipe node
  persis (`n8n-nodes-base.*`).

## Keunggulan vs n8n asli (nyata di kode ini, bukan janji)

1. Satu binary: server + UI single-file embedded — tanpa Node.js, npm, build step.
2. Offline-first: tanpa telemetri, tanpa CDN, tanpa akun.
3. Startup milidetik, memori kecil (engine sinkron, tanpa Electron).
4. Eksekusi deterministik + deteksi siklus yang menyebut nama node.
5. Durasi per node tercatat di setiap run (observabilitas bawaan).
6. Dry-run `explain`: rencana eksekusi tanpa menjalankan.
7. Linter: tipe tak dikenal, edge gantung, node terisolasi, source non-trigger.
8. Ekspresi bertipe-awet: satu `={{...}}` utuh tidak dipaksa jadi string.
9. Roundtrip lossless: field asing (pinData, versionId, ...) tidak hilang.
10. API JSON + UI tanpa-install: buka browser, Validate → Run.

## Struktur

```text
n8n-rust/
├── Cargo.toml            # workspace (0.2.0)
├── fixtures/             # workflow JSON format-n8n untuk test & contoh
├── crates/
│   ├── n8n-core/         # Workflow/Node + serde + mini ekspresi {{ }}
│   ├── n8n-engine/       # trait Node, Registry, executor, lint, explain
│   ├── n8n-nodes/        # manualTrigger, set, noOp, filter, sort, limit
│   ├── n8n-cli/          # validate / explain / run / nodes
│   └── n8n-server/       # REST API + UI single-file embedded (ui/app.html)
└── web/                  # keputusan UI (baca web/README.md)
```

## Quickstart

```bash
cd n8n-rust
cargo test
cargo run -p n8n-cli -- validate fixtures/manual-to-set.json
cargo run -p n8n-cli -- explain fixtures/manual-to-set.json
cargo run -p n8n-cli -- run fixtures/manual-to-set.json
cargo run -p n8n-server   # UI: http://localhost:3000
```

API: `GET /api/nodes`, `POST /api/validate`, `POST /api/explain`,
`POST /api/run` (body = workflow JSON).

## Subset ekspresi (jujur)

Didukung: `$json.a.b`, `$json["a"]`, `$json.arr[0]`,
`$node["Nama"].json.a` (+`.first()`), literal string/angka/bool/null.
Belum: operator (`==`, `>`), fungsi, `$("...")`, regex. Missing → Null lunak.

## Status jujur (0.2.0)

Ada: parse + roundtrip, ekspresi subset, 6 node, executor + timing,
lint + explain, CLI 4 perintah, server + UI embedded, 20 test.
Belum: operator ekspresi, node HTTP/Function/If-cabang, jadwal/webhook,
persistensi eksekusi, auth multi-user (ini alat personal).
