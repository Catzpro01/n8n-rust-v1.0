# AGENT7-W1-MCP-TRANSPORT-IMPLEMENTATION-PLAN — MCP Transport & Ingress Adapter (Wave-1, P0)

**Penulis:** agent7 (ROLE_AI_MCP) · **Tanggal:** 2026-09-09 · **Status:** REVIEW_READY (task W1-MCP-TRANS)
**Task:** `W1-MCP-TRANS` — Implementasi MCP Transport dan Ingress Adapter · **Wave:** 1 · **Prioritas:** P0
**Patuh Zero-Code Mandate:** dokumen ini = **rencana implementasi & kontrak antarmuka siap-eksekusi**; sketsa antarmuka bersifat desain (bukan kode yang dikomit ke crate mana pun — konsisten pola spec tim AGENT1/6/9). Kode aktual baru ditulis setelah PRD-3 disahkan pemilik & freeze dibuka.
**Dasar keputusan terkunci:** D-1 adapter ganda · D-2 stdio dulu · D-3 mode binary tunggal (`engine mcp`) · D-5 authoring-only (F-8/MCP-11) · D-7 review.
**Dokumen induk:** AGENT7-MCP-INTEGRATION-SPEC v0.4 · AGENT7-PROTOCOL-ROSTER v0.1 · AGENT7-JIT-ROUTING-REGISTRY v0.2 · AGENT7-MCP-SESSION-THREATMODEL v0.1.

---

## 1. Lingkup tugas & definisi selesai

**Lingkup W1-MCP-TRANS:** transport + ingress adapter server MCP engine — menerima koneksi klien (coding agent), menjamin pembingkaian pesan yang benar, routing JSON-RPC ke handler, pemetaan kesalahan, dan kepatuhan protokol (keluarga 2025 & 2026 per D-1) — TANPA tools/resources konten (itu W1-MCP-TOOLS, dependen pada tugas ini).

**Definisi selesai (DoD) saat freeze dibuka:**
1. `engine mcp --transport stdio` melayani handshake/discover + echo-routing; MCP Inspector lulus daftar metode (A7-1).
2. Keluarga 2026 (stateless, `server/discover`, `_meta` per-request) + keluarga 2025 (`initialize`, `instructions`, sesi versi lama) keduanya lulus conformance pada v1.
3. RAM persisten server = 0; transien per-request dibebaskan usai respons (F-7).
4. Uji negatif: permintaan eksekusi/fetch → ditolak (MCP-11, H-3); input invalid → Tool/Resource Execution Error, bukan Protocol Error (SEP-1303).
5. Redaksi oracle #394 aktif di batas facade (A7-6) — transport tak boleh melewati lapisan redaksi.
6. Review peer (matt/agent1) + QA (agent5) menandatangani checklist §7.

## 2. Posisi dalam binary & struktur modul (desain)

```
engine (binary tunggal, PRD-2 §2 / T-3)
└── subcommand `mcp`
    ├── mod transport        ← W1-MCP-TRANS (tugas ini)
    │   ├── stdio.rs           (v1; stdout=protokol murni, stderr=log)
    │   ├── frame.rs           (pembingkaian JSON-RPC: newline-delimited; batas ukuran)
    │   ├── adapter_2025.rs    (initialize/instructions/notifications; protocolVersion 2025-06-18|2025-11-25)
    │   ├── adapter_2026.rs    (server/discover; _meta per-request; stateless)
    │   └── router.rs          (dispatch method → handler; timeout; rate-limit dasar)
    ├── mod handler           ← handler tipis (tidak ada logika bisnis; W1-MCP-TOOLS melengkapi isi)
    ├── mod facade            ← SERVICE FACADE (kontrak agent1 #503; redaksi di sini, 1x)
    └── mod registry          ← registri URI + katalog (W3; dibaca per-request, tanpa state)
```

Batasan: transport & router TIDAK menyimpan state sesi apa pun (stateless, revisi MCP 2026-07-28); tidak ada dependensi engine lain di transport (dapat diuji dengan mock facade — prasyarat W7).

## 3. Kontrak antarmuka (sketsa desain, bukan kode final)

### 3.1 Trait facade (permukaan yang dikonsumsi router)

```rust
// Desain antarmuka — bukan implementasi. Konsumen: router (transport).
trait McpFacade {
    fn discover(&self) -> DiscoverResult;            // 2026: server/discover; 2025: initialize
    fn call_tool(&self, name: &str, args: Value) -> ToolResult;   // {isError, content, taint_flags}
    fn read_resource(&self, uri: &str) -> ResourceResult;         // {payload <=500 token, metadata}
    fn list_resources(&self) -> Vec<ResourceMeta>;
    fn get_prompt(&self, name: &str, args: Value) -> PromptResult;
}
```

Pemetaan tool→service (agent1 #503, disetujui): `inspect_workflow`→WorkflowService · `patch_node`→WorkflowService · `validate`→ValidateService · `preflight`→ValidateService (perluasan) · `get_receipt`→ReceiptService · `schema`→SchemaService. Redaksi & validasi hanya di facade; handler tidak menyentuh kredensial (F-9).

### 3.2 Router — aturan (dipakai kedua adapter)

| Aturan | Nilai | Sumber |
|---|---|---|
| Ukuran frame maks | 1 MiB (dev; kalibrasi target 2-CPU) | G-5, SEP-1303 |
| Timeout per-request | 30 s (validate ≤ 10 s); timeout = kesalahan internal | A7-4, batas guardrail |
| Parse error | -32700; params invalid -32602; method tak dikenal -32601 | JSON-RPC 2.0 |
| Kesalahan input pengguna | **Tool Execution Error** (`isError:true` + content terstruktur), bukan Protocol Error | SEP-1303, AGENT7-ROSTER §5 |
| Redaksi | wajib lewat facade sebelum respons; oracle #394 | agent5 #489 |
| Stateless | tanpa sesi; state transien per-request | MCP 2026-07-28 |

### 3.3 Binding keluarga 2025 (legacy; initialize)

1. Terima `initialize {protocolVersion, capabilities, clientInfo}` → balas `{protocolVersion, capabilities{...}, serverInfo, instructions: capsule}` (capsule ≤500 token, isi agent1; catatan: revisi 2026 menghapus ini — adapter 2026 tidak memakai `instructions`).
2. `notifications/initialized` → abaikan (kompat).
3. Dispatch method berikutnya; `MCP-Protocol-Version` divalidasi per-request.
4. Response disertai serverInfo; versi tak dikenal → 400/UnsupportedProtocolVersionError sesuai binding.

### 3.4 Binding keluarga 2026 (stateless; primer)

1. `server/discover` → `{protocolVersions[], capabilities, serverInfo}` (iklankan 2 keluarga per ROSTER §3).
2. Setiap request membawa `_meta.io.modelcontextprotocol/protocolVersion` + `clientCapabilities` + `clientInfo`; respons menyertakan `_meta.io.modelcontextprotocol/serverInfo`.
3. Tanpa sesi: bila koneksi/stream putus, klien mengirim ulang request dengan request-id baru (spec 2026-07-28 — tidak ada redelivery SSE).
4. Metode daftar tetap: tools/list·tools/call·resources/list·resources/templates/list·resources/read·prompts/list·prompts/get (isi = W1-MCP-TOOLS).

### 3.5 stdio (v1, D-2)

- Satu pesan JSON-RPC per baris (newline-delimited, UTF-8, tanpa newline embedded).
- stdout = protokol murni; SEMUA log ke stderr (spec + praktik; juga berlaku revisi apa pun).
- Flush setelah tiap pesan; baca sampai EOF; exit 0 normal, 1 error fatal, 2 = dipanggil tanpa subcommand valid (dokumentasikan — ERR-007 style).
- Tidak pernah menulis apa pun selain frame ke stdout (termasuk banner/warning — banner idCloudHost hanya di shell login, tidak di pipe).

## 4. Alur pesan (contoh konkret keluarga 2026)

```
Klien                          Server (engine mcp --transport stdio)
  │ server/discover ──────────► │ discover() → {protocolVersions, capabilities, serverInfo}
  │ tools/list ───────────────► │ router → (kosong di W1; diisi W1-MCP-TOOLS)
  │ tools/call validate ──────► │ router → facade.validate → DiagnosticReport → redaksi → isError:false
  │ resources/read hub://… ───► │ router → registry resolve (hub_id↔URI) → facade.read_resource
  │ (request eksekusi) ───────► │ TIDAK ADA method → -32601 (uji negatif MCP-11/H-3)
```

Keluarga 2025 identik dengan `initialize` di depan dan `instructions` di hasil initialize (ROSTER §5).

## 5. Keputusan teknis yang masih terbuka (bukan blocker W1)

| Item | Pemilik | Catatan |
|---|---|---|
| SDK rujukan (Rust/TS) & revisi yang didukung | agent7 + matt (Fase 1) | 4 kriteria ROSTER §3; mengunci detail framing |
| Base URL `exec_ref` via kapabilitas server | agent1 #566 | dipakai di hub_inspect (bukan transport inti) |
| Angka timeout final mesin 2-CPU | agent8 (ukur) | sementara dev-values |
| Kapabilitas tambahan (elicitation dll.) | matt | default: tidak diiklankan v1 |

## 6. Checklist verifikasi sebelum REVIEW_READY → DONE

| # | Item | Cara uji | Gate |
|---|---|---|---|
| C-1 | Frame stdio valid/invalid (newline, ukuran, UTF-8) | 30 kasus sintetis | A7-1 |
| C-2 | Keluarga 2026: discover → dispatch → response `_meta` serverInfo | MCP Inspector | A7-1 |
| C-3 | Keluarga 2025: initialize → instructions hadir ≤500 token → dispatch | MCP Inspector + tokenizer #419a | A7-1/A7-3 |
| C-4 | Method tak dikenal → -32601; params invalid → -32602 | 20 kasus | A7-1 |
| C-5 | Permintaan eksekusi/fetch → ditolak (tidak tersedia) | 20 uji negatif | MCP-11/H-3 |
| C-6 | RAM idle ±server = 0 persisten; transien dibebaskan | RSS sampling 100 ms | A7-4/F-7 |
| C-7 | Log hanya di stderr (0 baris log di stdout) | scan 50 sesi | A7-4 |
| C-8 | Redaksi oracle #394 lolos pada output semua metode | 50 sesi sintetis | A7-6 |
| C-9 | Stateless: 2 request berturut tanpa initialize (2026) sukses | uji berurutan | spec 2026-07-28 |
| C-10 | Koneksi putus di tengah (2026) → klien kirim ulang, sukses | uji kill-stream | spec 2026-07-28 |

## 7. Ketergantungan & catatan untuk W1-MCP-TOOLS

- **W1-MCP-TOOLS** (dependen): isi tools/5 + JIT routing + hub:// permukaan — memakai router/facade dari tugas ini; antarmuka sudah disiapkan di §3.
- Tidak bergantung K-1…K-3 (bisa prototipe dengan mock facade — konsisten keputusan agent1 #503 "Fase-1+ via kontrak-kernel tak terblokir K-1 untuk desain").
- Freeze zero-code: kode aktual menunggu tanda tangan pemilik atas PRD-3 (PRD-3 §6); rencana ini siap dieksekusi hari pertama Wave 1.

## 8. Artefak terkait (induk desain)

`AGENT7-MCP-INTEGRATION-SPEC.md` v0.4 (9a30e459) · `AGENT7-PROTOCOL-ROSTER.md` (df912fc8) · `AGENT7-JIT-ROUTING-REGISTRY.md` v0.2 (e43ec6db) · `AGENT7-MCP-SESSION-THREATMODEL.md` (210dda56) · PRD-3-PERFECTION-CHECKLIST v3.0.0-FINAL (Wave 1: MCP Transport = agent7).

*Ditulis oleh agent7 (ROLE_AI_MCP) untuk task W1-MCP-TRANS. Review dipersilakan (D-7).*
