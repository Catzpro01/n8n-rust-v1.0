//! `mcp_hub` — daemon Swarm Hub multiplexed (RFC-SWARM-MCP-HUB, RATIFIED fern #1223).
//!
//! Satu proses Rust di atas `crates/mcp` dengan tiga transport:
//!   - `--stdio`          : loop stdio (padanan `mcp_stdio`; worker lokal/pipeline)
//!   - `--socket <path>`  : Unix Domain Socket; tiap koneksi = sesi stateless,
//!     keep-alive ping 15s saat koneksi diam
//!   - `--sse-port <n>`   : HTTP/1.1 minimal (std-only): GET /events = SSE stream
//!     (ping 15s), POST /rpc = JSON-RPC request-response
//!
//! Prinsip: deterministik, tanpa tokio/tracing/async runtime (Ruling 7 #1023),
//! deps hanya std + `mcp`. Ping & push = liveness TRANSPORT (domain koordinasi
//! swarm) — tidak pernah masuk digest deterministik engine mana pun (Ruling 31).
//! Fase 1 = transport; Fase 2 = adapter comm.db (rusqlite, feature-gated);
//! Fase 3 = uji beban RSS <10MB (RFC §4).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use std::time::Duration;

use mcp::server::ServerCtx;

const PING_INTERVAL: Duration = Duration::from_secs(15);
const SSE_EVENT_PING: &str = "ping";
const MAX_HEADER: usize = 8 * 1024;
const SERVER_TAG: &str = "mcp-hub/0.1 (crates/mcp kanonik; stdio+unix+sse)";

// ─────────────────────────── argumen (std-only) ───────────────────────────

struct Args {
    socket: Option<String>,
    sse_port: Option<u16>,
    stdio: bool,
    sse_bind: String,
}

fn parse_args() -> Args {
    let mut a = Args { socket: None, sse_port: None, stdio: false, sse_bind: "127.0.0.1".into() };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--socket" => a.socket = it.next(),
            "--sse-port" => a.sse_port = it.next().and_then(|v| v.parse().ok()),
            "--sse-bind" => a.sse_bind = it.next().unwrap_or_else(|| "127.0.0.1".into()),
            "--stdio" => a.stdio = true,
            "--help" | "-h" => {
                eprintln!("usage: mcp_hub [--socket PATH] [--sse-port N] [--sse-bind IP] [--stdio]");
                std::process::exit(0);
            }
            other => eprintln!("[mcp-hub] argumen tak dikenal: {other}"),
        }
    }
    a
}

fn main() {
    eprintln!("[{SERVER_TAG}] start");
    let args = parse_args();
    let mut handles = Vec::new();

    if let Some(path) = args.socket.clone() {
        let h = thread::spawn(move || {
            if let Err(e) = serve_unix(&path) {
                eprintln!("[mcp-hub] unix {path}: {e}");
            }
        });
        handles.push(h);
    }
    if let Some(port) = args.sse_port {
        let bind = args.sse_bind.clone();
        let h = thread::spawn(move || {
            if let Err(e) = serve_sse(&bind, port) {
                eprintln!("[mcp-hub] sse {bind}:{port}: {e}");
            }
        });
        handles.push(h);
    }
    if args.stdio || (args.socket.is_none() && args.sse_port.is_none()) {
        serve_stdio();
    }
    for h in handles {
        let _ = h.join();
    }
    eprintln!("[mcp-hub] exit");
}

// ─────────────────────────── stdio ───────────────────────────

fn serve_stdio() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut ctx = ServerCtx::new();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => { eprintln!("[mcp-hub] stdin error: {e}"); break; }
        };
        if let Some(resp) = ctx.handle_line(&line) {
            if let Ok(s) = serde_json::to_string(&resp) {
                if writeln!(out, "{s}").is_err() { break; }
                let _ = out.flush();
            }
        }
    }
}

// ─────────────────────── Unix domain socket ───────────────────────

fn serve_unix(path: &str) -> std::io::Result<()> {
    // izin eksplisit (Ruling 10/13): socket 0660 — pembersihan path lama bila ada
    let _ = std::fs::remove_file(path);
    let listener = UnixListener::bind(path)?;
    // 0o660: owner+group rw. set_permissions eksplisit — jangan bergantung umask.
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o660))?;
    eprintln!("[mcp-hub] unix listening {path}");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => { thread::spawn(|| serve_unix_conn(s)); }
            Err(e) => eprintln!("[mcp-hub] accept: {e}"),
        }
    }
    Ok(())
}

fn serve_unix_conn(stream: UnixStream) {
    let _ = stream.set_read_timeout(Some(PING_INTERVAL));
    let mut writer = stream.try_clone().expect("clone unix stream");
    let mut reader = BufReader::new(stream);
    let mut ctx = ServerCtx::new();
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let t = line.trim();
                if t.is_empty() { continue; }
                if let Some(resp) = ctx.handle_line(t) {
                    if let Ok(s) = serde_json::to_string(&resp) {
                        if writeln!(writer, "{s}").is_err() { break; }
                        let _ = writer.flush();
                    }
                }
            }
            // koneksi diam selama PING_INTERVAL → kirim keep-alive (zero-timeout)
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock
                  || e.kind() == std::io::ErrorKind::TimedOut => {
                if writeln!(writer, "{{\"jsonrpc\":\"2.0\",\"method\":\"ping\"}}").is_err() { break; }
                let _ = writer.flush();
            }
            Err(_) => break,
        }
    }
    let _ = writer.flush();
}

// ─────────────────── SSE / HTTP streaming (std-only) ───────────────────

fn serve_sse(bind: &str, port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind((bind, port))?;
    eprintln!("[mcp-hub] sse listening http://{bind}:{port} (GET /events, POST /rpc)");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => { thread::spawn(|| handle_http(s)); }
            Err(e) => eprintln!("[mcp-hub] sse accept: {e}"),
        }
    }
    Ok(())
}

fn handle_http(mut stream: TcpStream) {
    let mut buf = Vec::new();
    // baca header sampai CRLFCRLF (batas MAX_HEADER)
    let mut tmp = [0u8; 512];
    let header_end = loop {
        if buf.len() > MAX_HEADER {
            let _ = stream.write_all(b"HTTP/1.1 431 Request Header Fields Too Large\r\n\r\n");
            return;
        }
        match stream.read(&mut tmp) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                buf.extend_from_slice(&tmp[..n]);
                if let Some(pos) = find_crlfcrlf(&buf) {
                    break pos;
                }
            }
        }
    };
    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut lines = head.lines();
    let req_line = lines.next().unwrap_or("");
    let mut path = req_line.split_whitespace().nth(1).unwrap_or("/").to_string();
    // potong query
    if let Some(q) = path.find('?') { path.truncate(q); }
    let body = &buf[header_end + 4..];

    match path.as_str() {
        "/events" => sse_stream(stream),
        "/rpc" => {
            // content-length → body
            let len = parse_content_length(&head).unwrap_or(body.len());
            let payload: &[u8] = if body.len() >= len { &body[..len] } else { body };
            let text = String::from_utf8_lossy(payload).trim().to_string();
            let mut ctx = ServerCtx::new();
            let resp = if text.is_empty() {
                "{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32600,\"message\":\"body kosong\",\"data\":null}}".to_string()
            } else {
                ctx.handle_line(&text)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "{\"jsonrpc\":\"2.0\",\"id\":null,\"result\":{}}".to_string())
            };
            let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{resp}", resp.len()).as_bytes());
            let _ = stream.flush();
        }
        _ => {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        }
    }
}

fn sse_stream(mut stream: TcpStream) {
    let _ = stream.set_write_timeout(Some(PING_INTERVAL * 3));
    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n");
    let _ = stream.flush();
    // keep-alive event tiap PING_INTERVAL; klien putus → write error → selesai
    loop {
        let ev = format!("event: {SSE_EVENT_PING}\ndata: {{\"ts_note\":\"transport keep-alive — bukan digest deterministik\"}}\n\n");
        if stream.write_all(ev.as_bytes()).is_err() { break; }
        if stream.flush().is_err() { break; }
        thread::sleep(PING_INTERVAL);
    }
}

fn find_crlfcrlf(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

fn parse_content_length(head: &str) -> Option<usize> {
    for l in head.lines() {
        let low = l.to_lowercase();
        if let Some(v) = low.strip_prefix("content-length:") {
            return v.trim().parse().ok();
        }
    }
    None
}
