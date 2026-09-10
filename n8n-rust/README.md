# n8n-rust v1 — v0.4.0

Implementasi ulang [n8n](https://n8n.io) (workflow automation) dalam Rust:
satu binary CLI + satu binary server HTTP dengan UI web embedded — tanpa
Node.js, tanpa database, tanpa telemetri. Format workflow JSON kompatibel
n8n (impor file n8n asli; node yang belum didukung dilaporkan eksplisit).

> Status: v0.4.0 — 12 node, scripting rhai, webhook hooks, riwayat run.
> Personal project, dipakai sendiri. `cargo test` wajib hijau di mesin
> dengan toolchain Rust (sandbox ini tidak punya cargo).

## Isi

- [Mulai cepat](#mulai-cepat)
- [CLI](#cli)
- [Server HTTP + webhook](#server-http--webhook)
- [Node yang didukung](#node-yang-didukung)
- [Ekspresi `={{ }}`](#ekspresi---)
- [Node Code (rhai)](#node-code-rhai)
- [ScheduleTrigger + cron](#scheduletrigger--cron)
- [UI web](#ui-web)
- [Struktur crate](#struktur-crate)
- [Batasan yang disengaja](#batasan-yang-disengaja)

## Mulai cepat

```bash
cargo build --release
./target/release/n8n-cli validate n8n-rust/fixtures/manual-to-set.json
./target/release/n8n-cli run n8n-rust/fixtures/manual-to-set.json --save report.json
./target/release/n8n-server   # http://127.0.0.1:3000
```

## CLI

```text
n8n-cli validate <workflow.json>   # lint: error + warning deterministik
n8n-cli explain <workflow.json>    # rencana urutan eksekusi (dry-run)
n8n-cli run <workflow.json> [--save <report.json>]  # eksekusi + simpan laporan
n8n-cli nodes                      # daftar 12 tipe node terdaftar
```

Contoh `run`:

```text
urutan: Manual Trigger -> Set -> NoOp
waktu: Manual Trigger=0ms, Set=0ms, NoOp=0ms (total 1ms)
{ "NoOp": [[{"greeting": "halo", "n": 1}]] }
tersimpan: report.json
```

## Server HTTP + webhook

Satu binary melayani UI (`/`) dan REST API. State (hooks + riwayat run)
in-memory — restart server menghapusnya.

| Method + path            | Fungsi                                              |
|--------------------------|-----------------------------------------------------|
| `GET /`                  | UI editor workflow (single-file, tanpa build)       |
| `GET /api/nodes`         | daftar tipe node                                    |
| `POST /api/validate`     | lint workflow (body = workflow JSON)                |
| `POST /api/explain`      | rencana urutan eksekusi                             |
| `POST /api/run`          | eksekusi workflow, tercatat di riwayat              |
| `GET /api/runs`          | 50 ringkasan run terakhir (lama → baru)             |
| `POST /api/hooks`        | daftarkan hook `{path, workflow}` → `201 "path"`    |
| `GET /api/hooks`         | daftar path hook terdaftar                          |
| `DELETE /api/hooks/:path`| hapus hook → `true`/`false`                         |
| `POST /hook/:path`       | picu workflow; node webhook menerima payload        |

Contoh webhook end-to-end:

```bash
# 1. daftarkan: workflow berisi node webhook -> set
curl -s -X POST localhost:3000/api/hooks -H 'Content-Type: application/json' \
  -d '{"path":"demo","workflow":{...}}'
# 2. picu dengan body JSON + query
curl -s -X POST 'localhost:3000/hook/demo?tag=a' -d '{"n":5}'
# node webhook meng-emit SATU item:
# {"method":"POST","path":"demo","query":{"tag":"a"},"headers":{...},"body":{"n":5}}
```

Run manual (CLI, `/api/run`, tombol Run di UI) tanpa payload membuat node
webhook meng-emit placeholder `{"mode":"manual"}` — workflow tetap bisa
diuji tanpa server hook.

## Node yang didukung

| Tipe | Fungsi |
|------|--------|
| `manualTrigger` | emit 1 item kosong |
| `scheduleTrigger` | emit `{scheduledAtEpoch, rule}` — penanda + metadata (jadwal riil via cron, lihat bawah) |
| `webhook` | emit payload request dari `/hook/:path`, atau `{mode:"manual"}` |
| `set` | tambah/timpa field; dua bentuk parameter (`values`, `assignments`) |
| `noOp` | teruskan item apa adanya |
| `filter` | teruskan item yang `condition`-nya truthy |
| `sort` | urutkan menurut `field`, `order` = `asc`/`desc` |
| `limit` | teruskan maksimal `count` item pertama |
| `if` | belah ke cabang `[true, false]` (output ganda pertama) |
| `httpRequest` | **fan-out**: 1 request per item input; 0 item → 0 request |
| `code` | script rhai atas variabel `items` |
| `function` | alias `code` (kompat impor workflow n8n lama) |

`httpRequest` (`url` wajib, `method` default GET, `headers` object,
`body` JSON; semua nilai string me-render template): tiap output
`{status, headers, body, url}`. Request pertama yang gagal menggagalkan
node (fail-fast) — bungkus dengan pola retry bila perlu nanti.

## Ekspresi `={{ }}`

String parameter yang diawali `=` dirender per item: `={{ $json.nama }}`,
`={{ $node["Set"].json.x }}`, operator `== != > < >= <= && || !`,
fungsi `len()`, `upper()`, `lower()`. Nilai non-string (`Value::render_value`)
ikut dirender rekursif (dipakai `httpRequest` untuk headers/body).

## Node Code (rhai)

Parameter `code` berisi script [rhai](https://rhai.rs):

```rhai
let total = 0;
for it in items { total += it.n; }
items = [{ "total": total }];
```

Aturan: variabel `items` (array) tersedia saat masuk dan **wajib array**
saat keluar; item non-object dibungkus `{"value": x`. Error script
menggagalkan node dengan pesan rhai asli.

> Keamanan: engine rhai berjalan **tanpa sandbox** (bisa akses file/sistem
> bila script memintanya — rhai standar membatasi, tapi jangan jalankan
> script dari sumber tak tepercaya). Untuk personal use ini disengaja:
> fleksibel penuh, tanggung jawab penuh.

## ScheduleTrigger + cron

Node `scheduleTrigger` TIDAK menjalankan jadwal sendiri — ia hanya menandai
"workflow ini dimaksudkan terjadwal" dan menyertakan epoch + `rule` per run.
Penjadwalan riil didelegasikan ke cron/systemd di mesin sendiri:

```cron
*/5 * * * * /opt/n8n-rust/n8n-cli run /opt/n8n-rust/wf/laporan.json --save /var/log/n8n-rust/laporan.json
```

Pola ini nol-dependency, survive reboot, dan log-nya file biasa.

## UI web

Editor visual di `GET /`: tambah 12 tipe node, drag-node, drag-dari-port
untuk edge (aturan If: edge pertama = true, kedua = false), klik edge
untuk hapus, edit parameters JSON, Validate/Explain/Run, badge urutan +
durasi per node — dan tombol **Export JSON** untuk mengunduh workflow
yang sedang diedit. Detail: [web/README.md](web/README.md).

## Struktur crate

```text
crates/
  n8n-core/    model Workflow + parser + expr (2 + 9 test)
  n8n-engine/  planner topo + executor + linter (6 test)
  n8n-nodes/   12 node + fixture e2e (15 test)
  n8n-cli/     validate/explain/run --save/nodes
  n8n-server/  axum: UI + REST + hooks + ring runs
fixtures/manual-to-set.json   workflow contoh (dipakai 1 test e2e)
web/README.md                 dokumentasi UI
```

Total: **32 test** (`cargo test --workspace`).

## Batasan yang disengaja

- State server in-memory (hooks + riwayat hilang saat restart).
- Eksekusi sinkron sekuensial (satu workflow satu thread blocking).
- Hook path satu segmen; hanya `POST /hook/:path`.
- Subset ekspresi kecil (tanpa ternary, tanpa `$items()`, tanpa JMESPath).
- rhai tanpa sandbox — script tepercaya saja.
- DB, auth multi-user, antrean, dan telemetry: TIDAK ADA — dan tidak
  direncanakan untuk personal use.
