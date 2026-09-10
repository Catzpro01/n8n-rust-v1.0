# AGENT7-MCP-HUB-FASE2-CONTRACT — Draf kontrak 4 tool comm.db (RFC-SWARM-MCP-HUB §3.3)

Status: DRAF v0.2 — KEPUTUSAN DE-FACTO terserap dari #1286 fern (RULING 37): socket directory resmi = /opt/agent-workspace/state/sockets/ (0770, agent-team) + AUTH TOKEN per-agent WAJIB (Ruling 38c/39, #1302/#1312). Sisa menunggu @matt #1256: (1) dep rusqlite feature [hub] vs crate terpisah; (2) inotify <5ms vs polling 250ms. Skema tool di bawah TIDAK bergantung keputusan itu — diturunkan dari kontrak CLI eksisting (`msg`, `swarm-task`) + skema comm.db (dibaca read-only 2026-09-09).

## Pemetaan target → tabel (aturan msg tool, diverifikasi dari baris nyata)
- `#channel` → messages.channel = nama, recipient = 'all'
- `@agent`  → messages.channel = 'dm', recipient = nama agent
- `all`     → messages.channel = ? (cek msg tool; umumnya 'general'/'all') — perlu keputusan: hub menerima {channel:"#x"|"@y"|"all"} dan menerjemah seperti msg.

## Tool 1 — mcp_forum_post
Params (JSON Schema):
```json
{
  "type": "object",
  "required": ["target", "message"],
  "properties": {
    "target": {"type": "string", "description": "#channel | @agent | all (sintaks msg)"},
    "message": {"type": "string", "maxLength": 65536},
    "reply_to": {"type": "integer", "description": "id pesan yang dibalas (opsional)"}
  },
  "additionalProperties": false
}
```
Semantik: INSERT messages(channel,sender,recipient,reply_to,content) — sender = agen pemanggil (dari sesi hub: header X-Agent / socket peer? LIHAT keputusan identitas, bawah). Result: {"status":"OK","id":<autoinc>}.
Kendala: tanpa bash mentah (spec §3.3.1); validasi target: maks 1 segmen, awalan #/@/all.

## Tool 2 — mcp_claim_task
Params:
```json
{"type":"object","required":["task_id"],
 "properties":{"task_id":{"type":"string"},"note":{"type":"string","description":"metadata opsional -> task_claim_log.metadata"}}}
```
Semantik (padanan `swarm-task claim <ID>`): UPDATE task_queue SET status='IN_PROGRESS', assigned_to=<self>, claimed_at=CURRENT_TIMESTAMP, heartbeat_at=CURRENT_TIMESTAMP WHERE id=? AND (status='UNCLAIMED' OR assigned_to=<self>) — ATOMIK via rusqlite transaction (BEGIN IMMEDIATE). + INSERT task_claim_log(task_id,agent,event='CLAIM'). Result: {claimed:true} / {claimed:false, reason:"not-unclaimed|not-found"}.

## Tool 3 — mcp_submit_review
Params:
```json
{"type":"object","required":["target","verdict","summary"],
 "properties":{"target":{"type":"string","description":"#channel utk post"},"task_id":{"type":"string"},"verdict":{"type":"string","enum":["APPROVE","APPROVE-WITH-CONDITIONS","REJECT","REVIEW-REQ"]},"summary":{"type":"string","maxLength":65536},"artifact_sha":{"type":"string"}}}
```
Semantik: mcp_forum_post dengan pesan terformat [REVIEW][<agent>] <task_id> — VERDICT ... + (bila task_id) catat di messages; TIDAK mengubah status task_queue (merge decision tetap manusia/matt). Result: {posted:true, message_id:...}.

## Tool 4 — mcp_heartbeat
Params: {"type":"object","required":["task_id"],"properties":{"task_id":{"type":"string"}}}
Semantik: UPDATE task_queue SET heartbeat_at=CURRENT_TIMESTAMP WHERE id=? AND assigned_to=<self> — mencegah RELEASE_TIMEOUT watchdog (RULING 24: heartbeat mengukur kerja). Result: {heartbeat:true} / {heartbeat:false, reason:"not-assigned"}.

## Identitas pemanggil — RESMI (RULING 37/38c/39, #1286)
- Socket: /opt/agent-workspace/state/sockets/mcp.sock (0770 dir; socket 0660 — konsisten Fase 1).
- AUTH WAJIB token per-agent: tiap koneksi membawa token; hub validasi thd registry salted-hash tokens.d/*.json (agent-token-register #1319/#1321: salt-32hex + hash-64hex b3sum). Alur: X-Agent-Token (HTTP header) / baris pertama handshake `{"jsonrpc":"2.0","method":"agent/authenticate","params":{"agent":"<nama>","token":"<raw>"}}` (socket/stdio) → hub verifikasi sha256/b3sum(token) vs tokens.d → sesi terautentikasi. Token TIDAK pernah di-log/disimpan; sender = identitas terautentikasi (bukan klaim klien).
- stdio (worker lokal): env AGENT_ID + AGENT_TOKEN saat spawn oleh supervisor; gagal-auth = koneksi ditolak.

## Batas & redaksi
- Sama dgn crate: redaksi di batas facade (A7-6) untuk isi pesan? TIDAK — pesan forum adalah data koordinasi (bukan log engine); redaksi hanya utk nilai yang memicu oracle (#394). Keputusan: redaksi TIDAK diterapkan ke content pesan (biarkan verbatim), tapi credential-looking tokens TETAP diredaksi (email/token/telepon) — konsisten crate.
- Rate/politeness: tidak ada sleep buatan; hub push <5ms (Fase 2, keputusan #1256-2).

— agent7, 2026-09-09 (draf)
