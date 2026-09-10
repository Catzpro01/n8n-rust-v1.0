# AGENT7-MCP-CONFORMANCE-TESTS — Kasus Uji Conformance MCP Server (siap-eksekusi)

**Penulis:** agent7 (ROLE_AI_MCP) · **Tanggal:** 2026-09-09 · **Status:** v0.1 — lampiran W1-MCP-TRANS/TOOLS (DONE)
**Tujuan:** mengubah gate C-1…C-10 / T-1…T-8 / A7-* menjadi kasus uji dengan input & expected output konkret, agar eksekusi Wave 1 pasca zero-code = menjalankan tabel ini (pola: test plan agent3 #703 untuk 21 gates). Bukan fitur baru — hanya operasionalisasi gate yang sudah disetujui.
**Patuh freeze #386:** dokumen uji, bukan kode.

---

## 1. Cara pakai

1. Setiap kasus punya ID unik, gate rujukan, langkah, input (frame JSON-RPC), dan expected.
2. Jalankan via MCP Inspector ATAU pipe stdio langsung (baris = 1 frame JSON).
3. Lolos = actual cocok expected. Gagal = catat di #error-log (format ERR-xxx) + laporkan.
4. Lingkungan uji: mock facade (tanpa engine penuh) — kontrak `AGENT7-W1-MCP-TRANSPORT-PLAN §3`.

## 2. Transport & framing (gate C-1, C-7)

| ID | Gate | Input (stdio, 1 baris/frame) | Expected |
|---|---|---|---|
| TC-01 | C-1 | JSON valid 1 baris `{"jsonrpc":"2.0","id":1,"method":"server/discover"}` | respons 1 baris JSON valid; stdout TANPA baris lain |
| TC-02 | C-1 | 1 baris berisi 2 objek JSON (concat) | parse error -32700; koneksi tetap hidup |
| TC-03 | C-1 | frame > 1 MiB (padding) | ditolak; error kesalahan internal/ukuran; tanpa crash |
| TC-04 | C-1 | UTF-8 invalid di tengah frame | parse error -32700 (atau per-aturan framing); tanpa hang |
| TC-05 | C-7 | 50 sesi valid campuran | stdout = 0 baris log; seluruh log di stderr |
| TC-06 | C-1 | EOF setelah request terakhir | proses exit 0; tanpa pesan tersisa |

## 3. Keluarga 2026 — stateless (gate C-2, C-9, C-10)

| ID | Gate | Input | Expected |
|---|---|---|---|
| TC-10 | C-2 | `server/discover` (tanpa sesi apa pun sebelumnya) | `{protocolVersions: [2025…, 2026…], capabilities:{tools,resources,prompts}, serverInfo}` |
| TC-11 | C-2 | `tools/list` LANGSUNG tanpa discover (2 request berurutan tanpa initialize) | sukses 2x — bukti stateless (C-9) |
| TC-12 | C-2 | request dgn `_meta.io.modelcontextprotocol/protocolVersion: "2026-07-28"` | sukses; respons memuat `_meta.io.modelcontextprotocol/serverInfo` |
| TC-13 | C-2 | request dgn protocolVersion tak dikenal | UnsupportedProtocolVersionError / ditolak dgn pesan versi |
| TC-14 | C-10 | koneksi diputus di tengah request (kill stream) | klien kirim ulang dgn request-id baru → sukses (tanpa redelivery) |
| TC-15 | C-9 | 100 request berurutan tanpa sesi | 100/100 sukses; RSS idle kembali ke baseline setelahnya |

## 4. Keluarga 2025 — initialize (gate C-3)

| ID | Gate | Input | Expected |
|---|---|---|---|
| TC-20 | C-3 | `initialize {protocolVersion:"2025-06-18", capabilities, clientInfo}` | hasil memuat `instructions` = capsule ≤500 token (tokenizer #419a) + capabilities benar |
| TC-21 | C-3 | `notifications/initialized` setelah initialize | no-op, koneksi tetap hidup |
| TC-22 | C-3 | `tools/list` SEBELUM initialize | ditolak sesuai aturan lifecycle 2025 (handshake wajib) |
| TC-23 | C-3 | initialize dgn versi 2025-11-25 | negosiasi: server pilih versi tertinggi yang didukung bersama |

## 5. Kesalahan & metode (gate C-4, MCP-11/H-3)

| ID | Gate | Input | Expected |
|---|---|---|---|
| TC-30 | C-4 | `{"method":"tidak-ada"}` | -32601 method not found |
| TC-31 | C-4 | `tools/call` tanpa params | -32602 invalid params |
| TC-32 | C-4 | JSON tidak valid | -32700 parse error |
| TC-33 | MCP-11/H-3 | `tools/call {name:"execute_workflow"}` (jika klien coba) | -32601 (method tidak tersedia) ATAU isError dgn pesan "eksekusi tidak via MCP" |
| TC-34 | MCP-11/H-3 | `tools/call {name:"fetch"}` / method eksekusi apa pun | tidak tersedia 100% (20 varian nama) |
| TC-35 | H-3b | `hub_list`/`hub_inspect` saat sesi (mock network monitor) | 0 panggilan jaringan keluar (est_token dari metadata) |
| TC-36 | SEP-1303 | `validate` dgn input invalid (workflow_id tak ada) | **Tool Execution Error**: isError:true + content terstruktur (BUKAN Protocol Error) |

## 6. Tools & diagnostik (gate T-1, T-5, A7-5)

| ID | Gate | Input | Expected |
|---|---|---|---|
| TC-40 | T-1 | `tools/list` | 5 tool: inspect_workflow, patch_node, validate, get_receipt, preflight; skema JSON Schema 2020-12 valid |
| TC-41 | T-5/A7-5 | `validate` workflow dgn ref gantung (`E-REF-DANGLING`) | `{valid:false, diagnostics:[{severity:ERROR, code:"E-REF-DANGLING", node, path, hint, autofix}]}` |
| TC-42 | T-5/A7-5 | `validate` expression by-index (`E-EXPR-REFERENCE`) | kode E-EXPR-REFERENCE + hint `$('Nama')` |
| TC-43 | T-5/A7-5 | `validate` expression kredensial (`E-EXPR-CREDENTIAL`) | HARD REJECT, tanpa autofix |
| TC-44 | T-5/A7-5 | `validate` float-precision | `{valid:true, warnings:[{code:"W-EXPR-FLOAT-PRECISION"}]}` — lolos + tercatat |
| TC-45 | T-5/A7-5 | 20 skenario sintetis (10 valid / 10 rusak) | kode benar 20/20; perbaikan ≤3 iterasi |
| TC-46 | T-1 | `patch_node` RFC6902 valid | `{applied:true}` + receipt ringkas |
| TC-47 | T-1 | `patch_node` RFC6902 invalid (path salah) | `{applied:false, diagnostics[]}` — tanpa crash |
| TC-48 | T-5 | `get_receipt` utk workflow ter-patch (receipt lama) | receipt di-re-lint / flag stale (H-7b) — TIDAK menyajikan receipt basi tanpa penanda |
| TC-49 | T-1 | `preflight` workflow butuh kredensial | daftar REF kredensial (bukan nilai) + estimasi fanout |

## 7. Resource routing & i18n (gate T-2, T-3, T-4, I-1…I-6, H-7)

| ID | Gate | Input | Expected |
|---|---|---|---|
| TC-50 | T-2 | `resources/list` | 13 URI (9 n8n:// + 4 hub://) sesuai REGISTRY v0.5; tanpa duplikasi |
| TC-51 | T-2 | `resources/read n8n://templates/{id}` | payload ≤500 token + metadata {content_owner, content_version, source_ref} |
| TC-52 | T-3/I-1 | `read n8n://templates/{id}?lang=id` (id tersedia) | 3 pilar (cara_kerja/fungsi/tujuan) bahasa Indonesia + i18n_rev |
| TC-53 | T-3/I-6 | `read ?lang=xx` (xx TIDAK tersedia) | fallback source_lang + `lang_fallback:true` |
| TC-54 | T-3/I-4 | `read ?lang=zz-invalid` | error standar + daftar available_langs |
| TC-55 | I-5 | update terjemahan saja (i18n_rev naik, content_version tetap) | content_version TIDAK berubah; i18n_rev naik |
| TC-56 | I-2 | `read ?lang={40 bahasa}` satu per satu | tiap payload ≤500 token (tokenizer #419a) |
| TC-57 | H-7/T-4 | `hub_inspect` dgn receipt.content_version < manifest.content_version | `receipt_outdated: true` + "re-execute needed" |
| TC-58 | T-2 | `read hub://skills/{hub_id}/manifest` (status draft) | TIDAK muncul di hub_list (hanya live); read langsung boleh/info status |
| TC-59 | T-2 | `read uri-tak-dikenal://x` | error standar resource tak dikenal |

## 8. Taint & redaksi (gate T-7, T-8, A7-6, H-4)

| ID | Gate | Input | Expected |
|---|---|---|---|
| TC-60 | T-8/H-4 | 50 skenario injeksi berstrata (5×10: data-driven, covert, tool-poison, node-name, cache-basi — agent3 #537) | tiap strata lulus pass criteria eksplisit; 0 instruksi tersembunyi tereksekusi |
| TC-61 | T-8 | Hub output → Code node → HTTP node (simulasi) | label taint TIDAK tercuci (TAINTED-EXTERNAL sampai sanitasi) |
| TC-62 | T-7/A7-6 | 50 sesi sintetis dgn fixture kredensial | 0 kecocokan oracle #394 (pola + daftar nilai fixture) |
| TC-63 | T-7 | workflow jahat berisi "BEGIN…PRIVATE KEY" di nama node | output escaped; 0 kebocoran pola |
| TC-64 | A7-6 | seluruh output tools/resources discan | 0 leak (definisi agent5 #489/#490) |

## 9. Ringkasan cakupan

| Gate | Kasus | Status |
|---|---|---|
| C-1/C-7 framing & log | TC-01…06 | siap |
| C-2/C-9/C-10 stateless 2026 | TC-10…15 | siap |
| C-3 initialize 2025 | TC-20…23 | siap |
| C-4/MCP-11/H-3/H-3b error & larangan | TC-30…36 | siap |
| T-1/T-5/A7-5 tools & diagnostik | TC-40…49 | siap |
| T-2/T-3/T-4/I-*/H-7 resource & i18n | TC-50…59 | siap |
| T-7/T-8/A7-6/H-4 taint & redaksi | TC-60…64 | siap |

Catatan: angka token final menunggu tokenizer rujukan #419a (matt). Angka ≤1 MiB frame & timeout = nilai dev, kalibrasi mesin 2-CPU oleh agent8 (A7-4).

## 10. Referensi

AGENT7-W1-MCP-TRANSPORT-PLAN (98d0f827) · AGENT7-W1-MCP-TOOLS-PLAN (08ecd4b3) · AGENT7-JIT-ROUTING-REGISTRY v0.5 (458f1117) · AGENT7-MCP-I18N-RESOURCE v0.2 (0933ddc7) · AGENT7-MCP-SESSION-THREATMODEL (210dda56) · AGENT3 #537 (strata injeksi) · agent5 #489/#490 (oracle LEAK).

*Ditulis oleh agent7 (ROLE_AI_MCP). Review dipersilakan — terutama agent1 (kontrak tools) & agent5 (gate keamanan).*
