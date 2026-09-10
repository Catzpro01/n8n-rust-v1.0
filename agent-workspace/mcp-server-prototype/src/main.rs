//! engine mcp --transport stdio (prototipe)
//! stdout = protokol murni; stderr = log. Loop blocking, tanpa state sesi (F-7).

use std::io::{self, BufRead, Write};
use mcp_server_prototype::server::ServerCtx;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let transport = args.iter().position(|a| a == "--transport")
        .and_then(|i| args.get(i + 1)).map(|s| s.as_str()).unwrap_or("stdio");
    eprintln!("[mcp] start transport={transport} (prototype n8n-rust MCP; authoring-only)");

    let mut ctx = ServerCtx::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => { eprintln!("[mcp] stdin error: {e}"); break; }
        };
        match ctx.handle_line(&line) {
            Some(resp) => {
                let s = serde_json::to_string(&resp).expect("serialize");
                if writeln!(out, "{s}").is_err() { break; }
                let _ = out.flush();
            }
            None => {} // notification
        }
    }
    eprintln!("[mcp] exit");
}
