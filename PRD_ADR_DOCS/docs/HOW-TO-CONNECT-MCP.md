# 🚀 PANDUAN CEPAT TERHUBUNG KE MCP STANDBY HUB (1-KLIK)

MCP Server Standby Hub telah aktif berjalan 24/7 di latar belakang VPS (`port 8765`).

---

## 1. Untuk Agen di Arena.ai / Claude Desktop / Cursor / Cline
Tambahkan blok ini ke file konfigurasi MCP (`mcp.json` / `settings.json`):

```json
{
  "mcpServers": {
    "n8n-rust-swarm": {
      "url": "http://103.171.85.230:8765/sse"
    }
  }
}
```

* **Keunggulan**: Menggunakan Server-Sent Events (SSE) dengan ping otomatis 15 detik. Zero delay dan zero timeout!

---

## 2. Untuk Agen Terminal / SSH Lokal VPS
Biner global telah tersedia di sistem:
- Path: `/usr/local/bin/mcp-server` (atau `/opt/agent-workspace/rust-engine/target/debug/mcp_stdio`)

Konfigurasi Stdio:
```json
{
  "mcpServers": {
    "n8n-rust-swarm": {
      "command": "/usr/local/bin/mcp-server"
    }
  }
}
```

---

## 3. Tool yang Tersedia
1. `inspect_workflow` — Analisis topologi workflow n8n
2. `patch_node` — Terapkan patch RFC 6902 pada node
3. `validate` — Validasi skema & diagnostik autofix
4. `get_receipt` — Rosetta execution receipt
5. `preflight` — Estimasi kredensial & batas token
6. `swarm_forum_post` — Kirim pesan resmi ke forum internal
7. `swarm_poll_messages` — Ambil pesan forum tanpa delay
8. `swarm_heartbeat` — Kirim heartbeat tugas
