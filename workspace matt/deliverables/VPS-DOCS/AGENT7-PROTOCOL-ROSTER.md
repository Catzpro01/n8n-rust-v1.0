# AGENT7-PROTOCOL-ROSTER — Roster Protokol MCP & Permukaan Server (v1)

**Penulis:** agent7 (ROLE_AI_MCP) · **Tanggal:** 2026-09-09 · **Status:** DRAFT v0.1 (masukan PRD-3, matt)
**Keputusan terkunci:** D-1 = **adapter ganda** (pemilik, 2026-09-09) · D-2 = **stdio dulu** (pemilik, 2026-09-09)
**Induk:** `AGENT7-MCP-INTEGRATION-SPEC.md` (W2 dari §7 rencana kerja) · Patuh freeze #386.

---

## 1. Tujuan

Satu referensi yang memetakan: revisi protokol MCP mana yang relevan, permukaan (metode/capability) yang dijanjikan server v1, dan bagaimana adapter ganda (D-1) bekerja. Dokumen ini **bukan** pengganti AGENT1/AGENT4 — ia hanya mengikat sisi protokol & integrasi yang menjadi milik agent7.

## 2. Matriks revisi protokol

| Revisi | Transport | Handshake/sesi | `instructions` | Catatan | Sikap v1 |
|---|---|---|---|---|---|
| 2024-11-05 | stdio, HTTP+SSE | initialize | field hasil initialize | HTTP+SSE deprecated (2026-07-28: resmi Deprecated) | JANGAN dibangun |
| 2025-03-26 | stdio, **Streamable HTTP** | initialize + sesi (Mcp-Session-Id) | field hasil initialize | OAuth 2.1 diperkenalkan | Keluarga 2025: binding kompat |
| 2025-06-18 | stdio, Streamable HTTP | sama (keluarga 2025) | sama | Elicitation; hasil tool terstruktur; JSON Schema | Baseline kompat klien lama |
| 2025-11-25 | stdio, Streamable HTTP | sama | sama | Ikon; OIDC discovery; sampling bertools; JSON Schema 2020-12 | Opsional di keluarga 2025 |
| **2026-07-28** | stdio, Streamable HTTP | **TANPA sesi/handshake**; `server/discover`; versi+capabilities per-request di `_meta`; `subscriptions/listen`; Roots/Sampling/Logging deprecated | tidak lagi via initialize — lihat AGENT7-MCP-INTEGRATION-SPEC §2.2 | **Stateless total** — sejalan gate RAM-0 (F-7) | Keluarga 2026: binding primer |

Sumber: modelcontextprotocol.io changelog (diakses 2026-09-09).

## 3. Strategi adapter ganda (D-1)

- **Satu permukaan internal (MCP-agnostik):** `discover()`, `list_tools()`, `call_tool()`, `list_resources()`, `read_resource()`, `list_prompts()`, `get_prompt()`. Handler bisnis TIDAK boleh menyadari revisi protokol.
- **Dua binding:** (a) *keluarga 2025* — initialize handshake, `instructions` di hasil initialize, protocolVersion 2025-06-18 (atau 2025-11-25) → untuk klien lama yang belum migrasi; (b) *keluarga 2026* — stateless, `server/discover`, per-request `_meta` → primer untuk implementasi baru.
- **Kriteria pemilihan saat Fase 1 (checklist, diisi saat SDK/klien aktual dipilih):**
  1. SDK rujukan mana yang dipakai engine (Rust/TS) dan revisi apa yang didukungnya;
  2. klien yang ditarget (Claude Code/Cursor/agent internal) mendukung revisi apa;
  3. hasil MCP Inspector conformance untuk tiap binding (A7-1);
  4. keputusan dicatat sebagai ADR singkat (norma PROVISIONAL-FREEZE kernel §12 Fase 0).
- **Catatan anotasi untuk dokumen fondasi** (usulan; agent1/agent4 dapat menyalin ke dokumennya):
  > *Catatan kompatibilitas protokol (agent7, 2026-09-09): bagian yang menyebut "disuntik saat connect via MCP instructions" berlaku untuk revisi MCP 2024-2025 (instructions = field hasil initialize). Revisi 2026-07-28 menghapus handshake/sesi; mekanik injeksi dipetakan ulang di AGENT7-PROTOCOL-ROSTER §3 + AGENT7-MCP-INTEGRATION-SPEC §2.2. Isi konseptual (capsule ≤500 token, pull-only, JIT) tidak berubah.*

## 4. Permukaan server v1 (janji minimum)

| Metode (keluarga 2026) / setara (keluarga 2025) | Isi | Pemilik konten |
|---|---|---|
| `server/discover` (2026) / `initialize` (2025) | identity, versi didukung, capabilities | agent7 |
| `tools/list`, `tools/call` | 5 tools: `inspect_workflow`, `patch_node`, `validate`, `get_receipt`, `preflight` | agent1 (skema parameter & hasil) |
| `resources/list`, `resources/templates/list`, `resources/read` | 9 URI `n8n://` (registri: `AGENT7-JIT-ROUTING-REGISTRY.md`) | agent1/2/3/4/5 per URI |
| `prompts/list`, `prompts/get` | 1 prompt: `workflow-authoring-loop` (argumen: workflow_id opsional) | agent7 (merakit dari agent1/3/4) |
| `notifications/*` | tidak ada di v1 (kecuali wajib per revisi) | — |

**Tidak dijanjikan v1:** eksekusi workflow (F-8), Roots/Sampling/Logging (deprecated 2026-07-28; tidak dibangun), HTTP+SSE, Streamable HTTP (Fase 2+, D-2), subscriptions/listen (menyusul bila ada push kebutuhan).

## 5. Kontrak kesalahan (pemetaan)

| Lapisan | Kode | Contoh |
|---|---|---|
| JSON-RPC | -32700 parse; -32600 request; -32601 method; -32602 params; -32603 internal | kesalahan transport/frame |
| MCP (keluarga 2025) / setara | -32002 resource tak dikenal; dst. sesuai revisi | URI salah |
| **Tool execution** (semua) | hasil `isError: true` + `content` terstruktur (SEP-1303: kesalahan validasi input = Tool Execution Error, bukan Protocol Error — supaya model bisa self-correct) | `E-EXPR-SCOPE` dsb. dari DiagnosticReport |

Format `content` untuk hasil tool validasi: blok JSON `DiagnosticReport` (kontrak AGENT4 §3) dibungkus terstruktur + ringkasan ≤1 baris per node (kompresi AGENT1 §3). Detail: AGENT7-MCP-SESSION-THREATMODEL §3.

## 6. Checklist conformance (gate A7-1, dijalankan saat implementasi)

MCP Inspector (atau setara): (1) initialize/discover → capability benar; (2) tools/list → 5 tool + skema valid JSON Schema 2020-12; (3) tools/call valid & invalid → isError benar, tanpa kebocoran; (4) resources/list + read tiap URI → format kemasan registri; (5) read URI tak dikenal → error standar; (6) prompts/get → prompt valid; (7) stdio: stdout bebas log (log hanya stderr); (8) keluarga 2025: `instructions` capsule ≤500 token (tokenizer #419) hadir di hasil initialize; (9) keluarga 2026: `server/discover` mengiklankan 2 keluarga versi.

## 7. Referensi

AGENT7-MCP-INTEGRATION-SPEC.md (§2, §4, §6); AGENT1-MCP-SKILL-SPEC.md; AGENT4-MCP-KNOWLEDGE-SPEC.md; changelog spec MCP (2026-09-09).
