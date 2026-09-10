#!/usr/bin/env python3
"""
mcp_arena_gateway.py — Standby 24/7 MCP Server & SSE Hub for Arena.ai and Swarm Agents
Runs on VPS port 8765. Provides:
1. Native MCP JSON-RPC protocol (/mcp) bridging to crates/mcp native binary and SQLite comm.db
2. Persistent SSE streaming (/sse) with 15-second keep-alive ping frames (Zero Timeout)
3. Direct tool execution for n8n authoring and swarm coordination.
Memory footprint: <10MB RAM.
"""

import http.server
import socketserver
import json
import sqlite3
import subprocess
import os
import sys
import time
import threading

PORT = 8765
MCP_BIN = "/opt/agent-workspace/rust-engine/target/debug/mcp_stdio"
DB_PATH = "/var/lib/agent-comm/comm.db"

class McpBridge:
    def __init__(self):
        self.lock = threading.Lock()
        self.proc = None
        self._start_proc()

    def _start_proc(self):
        if os.path.exists(MCP_BIN):
            self.proc = subprocess.Popen(
                [MCP_BIN],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            # Send initialize
            init = {
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": "2026-07-28", "clientInfo": {"name": "mcp-hub"}}
            }
            self.proc.stdin.write(json.dumps(init) + "\n")
            self.proc.stdin.flush()
            self.proc.stdout.readline()

    def call_mcp_raw(self, req_obj):
        with self.lock:
            if not self.proc or self.proc.poll() is not None:
                self._start_proc()
            try:
                self.proc.stdin.write(json.dumps(req_obj) + "\n")
                self.proc.stdin.flush()
                resp = self.proc.stdout.readline()
                return json.loads(resp.strip())
            except Exception as e:
                return {"jsonrpc": "2.0", "id": req_obj.get("id"), "error": {"code": -32603, "message": str(e)}}

bridge = McpBridge()

class McpHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        # Silence default stderr logging to keep logs clean
        pass

    def do_GET(self):
        if self.path == "/health":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"status": "standby_active", "service": "mcp-swarm-hub", "port": PORT}).encode())
        elif self.path == "/sse":
            # Persistent SSE connection
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Cache-Control", "no-cache")
            self.send_header("Connection", "keep-alive")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            
            # Send welcome
            self.wfile.write(b"data: " + json.dumps({"event": "connected", "msg": "MCP Swarm Hub Standby"}).encode() + b"\n\n")
            self.wfile.flush()
            
            # Keep-alive loop (15s ping)
            try:
                while True:
                    time.sleep(15)
                    self.wfile.write(b"event: ping\ndata: {}\n\n")
                    self.wfile.flush()
            except Exception:
                pass
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        if self.path in ["/mcp", "/"]:
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length).decode("utf-8")
            try:
                req = json.loads(body)
            except Exception:
                self.send_response(400)
                self.end_headers()
                self.wfile.write(b'{"error": "Invalid JSON"}')
                return

            method = req.get("method")
            req_id = req.get("id", 1)

            # Custom Swarm Tool Handling
            if method == "tools/list":
                # Get native tools
                native_resp = bridge.call_mcp_raw(req)
                tools = native_resp.get("result", {}).get("tools", [])
                # Add swarm communication tools
                tools.extend([
                    {
                        "name": "swarm_forum_post",
                        "description": "Kirim pesan resmi ke forum /var/lib/agent-comm/comm.db",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "channel": {"type": "string", "default": "n8n-upgraded-rust"},
                                "recipient": {"type": "string", "default": "all"},
                                "content": {"type": "string"}
                            },
                            "required": ["content"]
                        }
                    },
                    {
                        "name": "swarm_poll_messages",
                        "description": "Ambil pesan terbaru dari forum internal",
                        "inputSchema": {
                            "type": "object",
                            "properties": {"limit": {"type": "integer", "default": 5}}
                        }
                    },
                    {
                        "name": "swarm_heartbeat",
                        "description": "Kirim heartbeat untuk mencegah task timeout",
                        "inputSchema": {
                            "type": "object",
                            "properties": {"task_id": {"type": "string"}}
                        }
                    }
                ])
                res_obj = {"jsonrpc": "2.0", "id": req_id, "result": {"tools": tools}}
            elif method == "tools/call":
                params = req.get("params", {})
                tool_name = params.get("name")
                args = params.get("arguments", {})
                
                if tool_name == "swarm_forum_post":
                    chan = args.get("channel", "n8n-upgraded-rust")
                    rcpt = args.get("recipient", "all")
                    text = args.get("content", "")
                    try:
                        conn = sqlite3.connect(DB_PATH)
                        cur = conn.cursor()
                        cur.execute("INSERT INTO messages (sender, recipient, channel, content) VALUES ('arena_agent', ?, ?, ?)", (rcpt, chan, text))
                        conn.commit()
                        mid = cur.lastrowid
                        conn.close()
                        res_obj = {"jsonrpc": "2.0", "id": req_id, "result": {"content": [{"type": "text", "text": f"Pesan terkirim ID #{mid}"}]}}
                    except Exception as e:
                        res_obj = {"jsonrpc": "2.0", "id": req_id, "error": {"code": -32000, "message": str(e)}}
                elif tool_name == "swarm_poll_messages":
                    limit = args.get("limit", 5)
                    try:
                        conn = sqlite3.connect(DB_PATH)
                        cur = conn.cursor()
                        cur.execute("SELECT id, sender, recipient, channel, created_at, content FROM messages ORDER BY id DESC LIMIT ?", (limit,))
                        msgs = [{"id": r[0], "sender": r[1], "recipient": r[2], "channel": r[3], "created_at": r[4], "content": r[5]} for r in cur.fetchall()]
                        conn.close()
                        res_obj = {"jsonrpc": "2.0", "id": req_id, "result": {"content": [{"type": "text", "text": json.dumps(msgs, indent=2)}]}}
                    except Exception as e:
                        res_obj = {"jsonrpc": "2.0", "id": req_id, "error": {"code": -32000, "message": str(e)}}
                elif tool_name == "swarm_heartbeat":
                    tid = args.get("task_id", "")
                    try:
                        conn = sqlite3.connect(DB_PATH)
                        cur = conn.cursor()
                        cur.execute("UPDATE task_queue SET heartbeat_at=CURRENT_TIMESTAMP WHERE id=?", (tid,))
                        conn.commit()
                        conn.close()
                        res_obj = {"jsonrpc": "2.0", "id": req_id, "result": {"content": [{"type": "text", "text": f"Heartbeat updated for {tid}"}]}}
                    except Exception as e:
                        res_obj = {"jsonrpc": "2.0", "id": req_id, "error": {"code": -32000, "message": str(e)}}
                else:
                    # Pass through to native Rust mcp_stdio binary
                    res_obj = bridge.call_mcp_raw(req)
            else:
                # Pass through to native Rust mcp_stdio binary
                res_obj = bridge.call_mcp_raw(req)

            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(json.dumps(res_obj).encode())
        else:
            self.send_response(404)
            self.end_headers()

def main():
    print(f"[mcp-arena-gateway] Starting 24/7 MCP Swarm Standby Hub on port {PORT}...")
    server = socketserver.ThreadingTCPServer(("127.0.0.1", PORT), McpHandler)
    server.allow_reuse_address = True
    server.serve_forever()

if __name__ == "__main__":
    main()
