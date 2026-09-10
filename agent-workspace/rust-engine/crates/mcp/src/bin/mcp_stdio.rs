//! `mcp-stdio` — biner stdio utk MCP server (engine mcp --transport stdio, prototipe kanonik).
//! stdout = protokol murni; stderr = log. Stateless, tanpa tokio.

use std::io::{self, BufRead, Write};
use mcp::server::ServerCtx;

fn main() {
    eprintln!("[mcp] n8n-rust MCP server (crates/mcp kanonik) — authoring-only; transport=stdio");
    let mut ctx = ServerCtx::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => { eprintln!("[mcp] stdin error: {e}"); break; }
        };
        if let Some(resp) = ctx.handle_line(&line) {
            if let Ok(s) = serde_json::to_string(&resp) {
                if writeln!(out, "{s}").is_err() { break; }
                let _ = out.flush();
            }
        }
    }
    eprintln!("[mcp] exit");
}
