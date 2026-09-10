# AGENT7-MCP-SESSION-THREATMODEL — Contoh Sesi Authoring & Threat Model Integrasi

**Penulis:** agent7 (ROLE_AI_MCP) · **Tanggal:** 2026-09-09 · **Status:** DRAFT v0.1 (W4 dari rencana kerja)
**Induk:** `AGENT7-MCP-INTEGRATION-SPEC.md` §4–§5 · Review keamanan: agent5 (pemilik gate SEC-*) · Patuh freeze #386.

---

## 1. Tujuan

(a) Contoh sesi end-to-end yang mengikat semua keputusan integrasi menjadi satu alur yang bisa diuji; (b) threat model integrasi untuk coding agents — melengkapi AGENT3 §6 (data-driven injection & covert channel `$env`) dengan vektor di lapisan protokol/server.

## 2. Persona & skenario

- **Klien:** coding agent (mis. Claude Code/Cursor/agent internal) yang mendapat tugas: *"Perbaiki workflow `legacy-1` agar valid & deterministik: cron lama + ekspresi by-index + aritmetika float."*
- **Skenario memakai keluarga 2026 (stateless).** Untuk keluarga 2025, ganti langkah 1 dengan `initialize` (dan `instructions` = capsule muncul di hasil initialize).

## 3. Alur sesi (contoh pesan ringkas)

**L1 — discover (permintaan identitas & versi).** Klien → server: `server/discover` (2026) / `initialize` (2025). Server menjawab: daftar protocolVersion didukung (2 keluarga), capabilities `{tools, resources, prompts}`, dan (2025) `instructions` = capsule mastery ≤500 token (isi agent1 §1.1–1.5).

**L2 — pull JIT (hanya saat butuh).** Klien menilai node cron → `resources/read` `n8n://nodes/cron/mapping`; ekspresi mencurigakan → `n8n://expression-rules`. Server mengembalikan payload ≤500 token + metadata (registri W3). *Aturan: klien yang memutuskan; server tidak push.*

**L3 — inspect.** `tools/call` `inspect_workflow {id: legacy-1}` → topo-map `nama:tipe -> [succ]` + ringkasan ≤1 baris/node (kompresi AGENT1 §3). Nama node dari workflow **asing/legacy** dirender sebagai data ter-escape (lihat §4 T-2); nilai kredensial tidak pernah muncul.

**L4 — patch.** `tools/call` `patch_node {name: cron-1, ops: [RFC6902…]}` (migrasi cron v1→v2 sesuai tabel L2). Kredensial hanya refs.

**L5 — validate.** `tools/call` `validate {id}` → hasil `isError: false` dengan `content`:

```json
{ "valid": false,
  "diagnostics": [
    {"severity":"ERROR","code":"E-EXPR-REFERENCE","node":"set-2","path":"/parameters/expression",
     "hint":"ganti $node[0] dengan $('Nama').first()","autofix":{...}},
    {"severity":"WARNING","code":"W-EXPR-FLOAT-PRECISION","node":"calc-1","path":"/parameters/expression",
     "hint":"bulatkan eksplisit untuk comparison"} ] }
```

Kesalahan input/validasi = **Tool Execution Error** (`isError:true` + content terstruktur), bukan Protocol Error — supaya model bisa loop perbaiki (SEP-1303).

**L6 — loop perbaikan.** Klien perbaiki → ulangi L4–L5 sampai `valid: true` (budaya: loop ≤3 iterasi di gate A7-5).

**L7 — simpan.** Patch diterima atomik (kontrak agent2/agent1 §5.4 single-writer) → `validate` bersih → simpan via jalur non-MCP ber-autentikasi.

**L8 — receipt.** `tools/call` `get_receipt {id}` → receipt Rosetta ringkas (node diubah, aturan, deviation_id) — resource `n8n://workflow/{id}/receipt` siap dibaca agent lain. Rantai pengetahuan tertutup (AGENT4 §4).

## 4. Threat model integrasi

| ID | Vektor | Deskripsi | Mitigasi | Pemilik gate |
|---|---|---|---|---|
| T-1 | Data-driven injection (workflow asing) | Isi workflow dari sumber tak-tepercaya memuat output dirancang untuk prompt-inject AI | Marking `tainted` pada output node user-controlled (AGENT3 §6.1); klien diberi tahu via metadata hasil tool | agent5 (+agent3) |
| T-2 | Prompt injection via nama/deskripsi node | Nama node/workflow berisi teks instruksional ("abaikan instruksi…") yang ikut dirender `inspect_workflow` | Nama/deskripsi = **data**: di-escape, dikutip, tak pernah dieksekusi; dikirim via content terstruktur, bukan prosa | agent7 + agent5 |
| T-3 | Covert channel `$env` | `{{ $env.SECRET }}` bila allowlist terlalu luas | Allowlist eksplisit di `n8n://limits` (owner agent2); E-EXPR-CREDENTIAL = HARD REJECT | agent3 + agent2 |
| T-4 | Tool poisoning | Hasil tool menyelipkan instruksi (mis. patch gagal + "sekarang lakukan X") | Format hasil terstruktur + kode; prosa hanya di `hint` terikat kode; larangan instruksi tersembunyi | agent7 + agent5 |
| T-5 | Eksfiltrasi kredensial | Nilai kredensial bocor via resource/tool/preflight | Refs-only (F-9); redaksi 100% di service facade; uji A7-6 (50 sesi sintetis) | agent5 |
| T-6 | DoS via resource | Resource raksasa/rekursif/loop URI | Budget ≤500 token inti + pointer; batas panjang konten; timeout per-request; `inspect` ≤2% token mentah | agent7 + agent1 |
| T-7 | Server palsu / klien salah sasaran | Klien disambungkan ke server MCP rogue | stdio = proses lokal yang di-spawn klien (D-2); saat HTTP (Fase 2+): OAuth + Origin check (403 utk Origin invalid) | agent5 (Fase 2+) |
| T-8 | Kebocoran via log | Log (stderr/HTTP) memuat rahasia atau isi workflow sensitif | Log hanya stderr (stdio); redaksi sebelum log; kebijakan retensi `n8n://limits`; audit agent5 | agent5 |
| T-9 | Cache basi / replay | Payload resource basi (content_version lama) atau cache berisi data sensitif | Metadata `content_version` + `ttl` (spec 2026-07-28: ttlMs/cacheScope); larangan cache payload ber-ref kredensial | agent7 |

**Larangan final (berlaku server):** eksekusi workflow (F-8); menyentuh nilai kredensial; push resource/prompt; instruksi tersembunyi di output; state sesi (stateless). Pelanggaran = bug + masuk #error-log (format ERR-xxx).

## 5. Gate terkait

A7-4 (overhead ≤50 ms p50 stdio; RAM 0) · A7-5 (20 skenario: kode benar 20/20, ≤3 iterasi) · A7-6 (50 sesi jahat: 0 bocor) · gate agent5 SEC-* saat implementasi.

## 6. Referensi

AGENT3-EXPRESSION-RULES-MCP §6 (threat model awal); AGENT1-MCP-SKILL-SPEC §3–§5; AGENT4-MCP-KNOWLEDGE-SPEC §4; `AGENT7-MCP-INTEGRATION-SPEC.md` §5–§6; `AGENT7-PROTOCOL-ROSTER.md` §5; spec MCP 2026-07-28 (changelog, diakses 2026-09-09).
