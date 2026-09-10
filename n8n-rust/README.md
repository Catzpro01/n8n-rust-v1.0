# n8n-rust v1 — v0.5.0

Implementasi ulang [n8n](https://n8n.io) (workflow automation) dalam Rust:
satu binary CLI + satu binary server HTTP dengan UI web embedded — tanpa
Node.js, tanpa database, tanpa telemetri. Format workflow JSON kompatibel
n8n (impor file n8n asli; node yang belum didukung dilaporkan eksplisit).

> Status: v0.5.0 — fidelity pass terhadap n8n asli: 12 node memakai bentuk
> parameter & perilaku n8n (conditions, mode code, responseMode webhook…).
> Personal project, dipakai sendiri. `cargo test` wajib hijau di mesin
> dengan toolchain Rust (sandbox ini tidak punya cargo).

## Isi

- [Mulai cepat](#mulai-cepat)
- [CLI](#cli)
- [Server HTTP + webhook](#server-http--webhook)
- [Node yang didukung](#node-yang-didukung)
- [Conditions If/Filter](#conditions-iffilter)
- [Ekspresi `={{ }}`](#ekspresi---)
- [Node Code (rhai)](#node-code-rhai)
- [ScheduleTrigger + cron](#scheduletrigger--cron)
- [UI web](#ui-web)
- [Struktur crate](#struktur-crate)
- [Beda disengaja vs n8n](#beda-disengaja-vs-n8n)
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
| `GET/POST/PUT/PATCH/DELETE /hook/:path` | picu workflow; method dicek vs `httpMethod` |

Contoh webhook end-to-end (default n8n = GET):

```bash
# 1. daftarkan: workflow berisi node webhook (httpMethod GET) -> set
curl -s -X POST localhost:3000/api/hooks -H 'Content-Type: application/json' \
  -d '{"path":"demo","workflow":{...}}'
# 2. picu via GET + query
curl -s 'localhost:3000/hook/demo?tag=a'
# node webhook meng-emit SATU item:
# {"method":"GET","path":"demo","query":{"tag":"a"},"headers":{...},"body":null}
```

Method yang salah → `404` (n8n mendaftarkan webhook per method+path).
Tanpa node webhook di workflow → semua method diterima (warisan 0.4).

Respon diatur node webhook (persis nama parameter n8n):

| Parameter | Nilai | Arti |
|-----------|-------|------|
| `httpMethod` | `GET` (default) \| `POST` \| `PUT` \| `PATCH` \| `DELETE` | method yang diterima |
| `responseMode` | `onReceived` (default) \| `lastNode` | kapan & apa yang direspon |
| `responseData` | `firstEntryJson` (default) \| `allEntries` \| `noResponseBody` | bentuk body lastNode |
| `responseCode` | angka, default `200` | status HTTP lastNode |

`onReceived` mengembalikan **laporan run penuh** (beda disengaja: n8n
asli mengembalikan ack "Workflow was started"; run lokal sinkron —
laporan lebih berguna). `lastNode` menunggu workflow selesai lalu
mengembalikan item pertama / semua item node terakhir dieksekusi.

Run manual (CLI, `/api/run`, tombol Run di UI) tanpa payload membuat node
webhook meng-emit placeholder `{"mode":"manual"}`.

## Node yang didukung

| Tipe | Parameter perilaku (nama persis n8n) |
|------|--------------------------------------|
| `manualTrigger` | emit `[{}]` |
| `scheduleTrigger` | emit field persis n8n (UTC) + echo `rule`; jadwal riil via cron |
| `webhook` | `path`, `httpMethod`, `responseMode`, `responseData`, `responseCode` |
| `set` | `mode` manual/raw; `assignments` + `fields.values`; `include` all/none/selected/except + `includeFields`/`excludeFields`; `options.dotNotation`; raw = `jsonOutput` |
| `noOp` | teruskan apa adanya |
| `filter` | `conditions` (fallback: string `condition` lawas) |
| `sort` | `type` simple/random/code; `sortFieldsUi.sortField[]` (`fieldName`, `order` ascending/descending); `options.disableDotNotation` |
| `limit` | `maxItems` + `keep` firstItems/lastItems |
| `if` | `conditions` → cabang `[true, false]` (fallback `condition` lawas) |
| `httpRequest` | fan-out 1 request/item; `options.timeout` ms (default 300000); responseFormat autodetect/json/text |
| `code` | `mode` runOnceForAllItems/runOnceForEachItem; script rhai |
| `function` | alias `code` |

Detail per node:

- **set**: `include: all` (default) mulai dari salinan input; `none` dari
  kosong; `selected`/`except` memakai daftar koma `includeFields` /
  `excludeFields` (dot-notation, basename untuk path bertitik — persis
  n8n). Field baru selalu ditulis terakhir via dot-notation
  (`options.dotNotation`, default true). `fields.values[]` lawas
  (`stringValue`/`numberValue`/…) didukung; `objectValue`/`arrayValue`
  string dicoba parse JSON. Mode `raw`: `jsonOutput` di-render lalu harus
  jadi object.
- **sort**: multi-field, string case-insensitive, field hilang di SEMUA
  item → error `Couldn't find the field 'x' in the input data` (persis
  n8n). `type: random` = acak (xorshift, tanpa crate rand); `type: code`
  ditolak eksplisit (butuh JS — gunakan node Code).
- **limit**: `maxItems` > jumlah item → semua item; `keep: lastItems` =
  N terakhir. (`count` lawas tetap diterima bila `maxItems` absen.)
- **httpRequest**: `options.timeout` milidetik (default 300000 = 5 menit,
  sama seperti n8n). Format dibaca dari path asli n8n
  `options.response.response.responseFormat` (alias pendek
  `options.responseFormat` juga diterima): `autodetect` (coba JSON lalu
  string), `json` (wajib JSON valid), `text` (mentah). `file` ditolak
  eksplisit (tanpa penyimpanan binary). Status ≥ 400 TIDAK error — selalu
  output `{status, headers, body, url}` (beda disengaja: engine belum
  punya error-output).

## Conditions If/Filter

Object `conditions` persis format n8n (`filter`-type parameter):

```json
{"combinator": "and", "conditions": [
  {"leftValue": "={{ $json.age }}", "rightValue": 18,
   "operator": {"type": "number", "operation": "gte"}}
], "options": {"caseSensitive": false, "typeValidation": "loose"}}
```

`leftValue`/`rightValue` me-resolve `={{ }}` dulu (seperti n8n), lalu
dievaluasi. `combinator`: `and`/`or`. Opsi default = n8n:
case-insensitive (`ignoreCase` default true) + validasi tipe STRICT
(`looseTypeValidation` default false).

Operator per tipe (disalin dari tabel n8n-workflow):

- string: `empty notEmpty equals notEquals contains notContains
  startsWith notStartsWith endsWith notEndsWith regex notRegex`
- number: `empty notEmpty equals notEquals gt lt gte lte`
- dateTime: `empty notEmpty equals notEquals after before
  afterOrEquals beforeOrEquals` (ISO-8601/RFC3339 atau ms epoch)
- boolean: `empty notEmpty true false equals notEquals`
- array: `contains notContains lengthEquals lengthNotEquals lengthGt
  lengthLt lengthGte lengthLte empty notEmpty`
- object: `empty notEmpty`
- semua tipe: `exists notExists`

Mode loose mengonversi (`"20"` → 20, boolean ala `Boolean()` JS —
`"false"` → true!). Regex `/pola/flags` (flags `i`/`m`/`s`; tanpa
pelindung timeout — pola tepercaya saja). Beda disengaja: operator tak
dikenal → error eksplisit (n8n: warn + false).

## Ekspresi `={{ }}`

String parameter yang diawali `=` dirender per item: `={{ $json.nama }}`,
`={{ $node["Set"].json.x }}` (+ alias n8n `={{ $('Set').json.x }}`),
`$now` (RFC3339 ms UTC), `$today` (tengah malam UTC), operator
`== != > < >= <= && || !`, fungsi `len()`, `upper()`, `lower()`.
Nilai non-string ikut dirender rekursif.

## Node Code (rhai)

Pengganti JavaScript/Python n8n = [rhai](https://rhai.rs) (embed, tanpa
runtime eksternal). `mode` persis n8n:

- `runOnceForAllItems` (default): variabel `items` (array) tersedia dan
  wajib array saat keluar; item non-object dibungkus `{"value": x}`.
- `runOnceForEachItem`: script jalan per item dengan `item` + `index`;
  `item` akhir array → di-spread, kalau tidak → satu item.

> Keamanan: engine rhai berjalan **tanpa sandbox**. Jangan jalankan script
> dari sumber tak tepercaya. Untuk personal use ini disengaja: fleksibel
> penuh, tanggung jawab penuh.

## ScheduleTrigger + cron

Node meng-emit field yang sama seperti n8n (`timestamp`, `Readable date`,
`Readable time`, `Day of week`, `Year`, `Month`, `Day of month`, `Hour`,
`Minute`, `Second`, `Timezone` = UTC) + echo `rule` (ekstensi). Ia TIDAK
menjalankan jadwal — delegasikan ke cron/systemd:

```cron
*/5 * * * * /opt/n8n-rust/n8n-cli run /opt/n8n-rust/wf/laporan.json --save /var/log/n8n-rust/laporan.json
```

Bentuk `rule` mengikuti n8n: `{"interval": [{"field": "days"}]}`
(field: seconds/minutes/hours/days/weeks/months/cron).

## UI web

Editor visual di `GET /`: tambah 12 tipe node (template parameter bentuk
n8n), drag-node, drag-dari-port untuk edge (If: pertama = true, kedua =
false), klik edge untuk hapus, edit parameters JSON, Export JSON,
Validate/Explain/Run, badge urutan + durasi. Detail: [web/README.md](web/README.md).

## Struktur crate

```text
crates/
  n8n-core/    model Workflow + parser + expr ($node alias, $now/$today) (2 + 11 test)
  n8n-engine/  planner topo + executor + linter (6 test)
  n8n-nodes/   12 node + filter.rs conditions (28 + 8 test)
  n8n-cli/     validate/explain/run --save/nodes
  n8n-server/  axum: UI + REST + hooks (multi-method, responseMode) + ring runs
fixtures/manual-to-set.json   workflow contoh (dipakai 1 test e2e)
web/README.md                 dokumentasi UI
```

Total: **55 test** (`cargo test --workspace`).

Referensi perilaku n8n asli (dibaca saat v0.5.0, implementasi tetap
orisinal): `packages/workflow/src/node-parameters/filter-parameter.ts`,
`nodes/{Set/v2,If/V2,Filter/V2,Code,Webhook,Schedule,Transform/{Limit,Sort},HttpRequest/V3}`.

## Beda disengaja vs n8n

Perbedaan perilaku yang dipilih sadar (bukan bug):

- `onReceived` mengembalikan RunReport, bukan ack "Workflow was started".
- HTTP status ≥ 400 tidak error (output `{status,…}` biasa).
- Operator/combinator conditions tak dikenal → error (n8n: warn + false).
- `array contains` object memakai kesetaraan mendalam (n8n: referensi JS).
- Angka vs null → false (n8n: koersi JS).
- Sort type `code` ditolak (butuh JS); regex tanpa pelindung timeout.
- Timezone schedule selalu UTC (n8n: timezone workflow).

## Batasan yang disengaja

- State server in-memory (hooks + riwayat hilang saat restart).
- Eksekusi sinkron sekuensial (satu workflow satu thread blocking).
- Hook path satu segmen.
- Subset ekspresi kecil (tanpa ternary, tanpa `$items()`, tanpa JMESPath).
- rhai tanpa sandbox — script tepercaya saja.
- DB, auth multi-user, antrean, dan telemetry: TIDAK ADA — dan tidak
  direncanakan untuk personal use.
