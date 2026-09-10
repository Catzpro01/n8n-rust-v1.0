//! W0-ANCHOR-IMPL — RFC 3161 TSA anchor client (writer + verifier).
//!
//! Implements AGENT10-W0-ANCHOR-SPEC.md §4 (record), §5 (protocol incl. V1–V8),
//! §6 (cadence/window), §7 (quorum, MVP = M-of-M explicit), §9 (DDL verbatim).
//!
//! # Trust-relevant design notes (read before audit)
//! - ZERO new crypto code: ASN.1 / signing / verify done by `openssl ts` CLI,
//!   transport by `curl` CLI. Rust only orchestrates, parses text output, and
//!   enforces policy (notably ANC-5: refuse to run without explicit `--pin`).
//! - Canonical payload (C-02-safe, length-prefixed):
//!   `u32BE(len)||chain_id || head(32) || prev(32) || u64BE(from) || u64BE(to)
//!    || u64BE(count) || u32BE(len)||local_ts`.
//! - `local_ts` == `created_utc` row value (single clock read). This makes the
//!   payload reconstructible from `anchor_log` alone, which is what §5.3
//!   (offline audit re-verify from artifacts) requires. Without this identity
//!   V1 re-verification would be impossible — spec §4 lists `local_ts` in the
//!   payload but §9 stores no separate `local_ts` column.
//! - D-A1 default (owner undecided): fail-OPEN ops / fail-CLOSED claims — TSA
//!   outage yields a FAILED row + alarm on stderr, exit status 0 (non-blocking).
//! - D-A2 resolved per agent1 #730: dual trigger (N entries OR 60 s); N counts
//!   ENVELOPE ENTRIES, not items. Cadence scheduling itself is the caller's job
//!   (cron/engine); this binary anchors exactly one window per invocation.

#![forbid(unsafe_code)]

use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const DDL: &str = r#"
CREATE TABLE IF NOT EXISTS anchor_log (
  anchor_id        INTEGER PRIMARY KEY,
  chain_id         TEXT    NOT NULL,
  chain_head       BLOB    NOT NULL CHECK (length(chain_head) = 32),
  prev_anchor_head BLOB    NOT NULL CHECK (length(prev_anchor_head) = 32),
  entry_from       INTEGER NOT NULL,
  entry_to         INTEGER NOT NULL,
  entry_count      INTEGER NOT NULL,
  imprint_algo     TEXT    NOT NULL DEFAULT 'sha256',
  nonce            BLOB    NOT NULL CHECK (length(nonce) >= 8),
  tsa_id           TEXT    NOT NULL,
  tsa_cert_fp      TEXT    NOT NULL,
  tsa_serial       TEXT    NOT NULL,
  tsa_time         TEXT    NOT NULL,
  tsr_sha256       BLOB    NOT NULL CHECK (length(tsr_sha256) = 32),
  tsr_path         TEXT    NOT NULL,
  quorum           TEXT    NOT NULL,
  status           TEXT    NOT NULL CHECK (status IN ('ANCHORED','PARTIAL','FAILED')),
  created_utc      TEXT    NOT NULL
);
CREATE TABLE IF NOT EXISTS pin_change_log (
  change_id     INTEGER PRIMARY KEY,
  tsa_id        TEXT NOT NULL,
  old_fp        TEXT,
  new_fp        TEXT NOT NULL,
  reason        TEXT NOT NULL,
  authorized_by TEXT NOT NULL,
  changed_utc   TEXT NOT NULL
);
"#;

const DEFAULT_DB: &str = "/mnt/extra-storage/anchor/anchor_log.sqlite";
const DEFAULT_TSR_DIR: &str = "/mnt/extra-storage/anchor/tsr";
const CLOCK_TOL_SECS: i64 = 600; // V7 ±10 min
const TSA_TIMEOUT_SECS: u64 = 25; // §5.1 step 4

/// One anchor_log row as read back for audit (field order = SELECT order).
type AnchorRow = (
    i64, String, Vec<u8>, Vec<u8>, i64, i64, i64, Vec<u8>, String, String, Vec<u8>, String,
    String, String,
);

// ---------- small utilities (no extra deps, auditable) ----------

fn hex_encode(b: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(H[(x >> 4) as usize] as char);
        s.push(H[(x & 15) as usize] as char);
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) {
        return Err(format!("hex length {} is odd", s.len()));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn hex_val(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("bad hex char '{c}'")),
    }
}

/// Normalize a SHA-256 fingerprint: strip colons/spaces/`0x`, lowercase.
fn norm_fp(s: &str) -> String {
    s.trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .chars()
        .filter(|c| *c != ':' && !c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
}

/// Normalize a nonce hex string for comparison (strip 0x + leading zeros).
fn norm_nonce(s: &str) -> String {
    let t = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    let t = t.trim_start_matches('0');
    if t.is_empty() {
        "0".to_string()
    } else {
        t.to_lowercase()
    }
}

// (nonce is generated inside `openssl ts -query`; no local RNG needed.)
fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Days from civil (Howard Hinnant). Valid for full i64 range of sane dates.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn epoch_to_iso_utc(epoch: i64) -> String {
    let days = epoch.div_euclid(86400);
    let secs = epoch.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

fn month_num(s: &str) -> Option<i64> {
    match s {
        "Jan" => Some(1),
        "Feb" => Some(2),
        "Mar" => Some(3),
        "Apr" => Some(4),
        "May" => Some(5),
        "Jun" => Some(6),
        "Jul" => Some(7),
        "Aug" => Some(8),
        "Sep" => Some(9),
        "Oct" => Some(10),
        "Nov" => Some(11),
        "Dec" => Some(12),
        _ => None,
    }
}

/// Parse `openssl ts -reply -text` "Time stamp: Sep  9 07:44:01 2026 GMT".
fn parse_tsa_time(s: &str) -> Result<i64, String> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(format!("bad TSA time (want 5 fields): {s:?}"));
    }
    let mon = month_num(parts[0]).ok_or_else(|| format!("bad month: {}", parts[0]))?;
    let day: i64 = parts[1].parse().map_err(|_| format!("bad day: {}", parts[1]))?;
    let t: Vec<&str> = parts[2].split(':').collect();
    if t.len() != 3 {
        return Err(format!("bad clock: {}", parts[2]));
    }
    let (hh, mm, ss): (i64, i64, i64) = (
        t[0].parse().map_err(|_| "bad hh")?,
        t[1].parse().map_err(|_| "bad mm")?,
        t[2].parse().map_err(|_| "bad ss")?,
    );
    let year: i64 = parts[3].parse().map_err(|_| format!("bad year: {}", parts[3]))?;
    if parts[4] != "GMT" && parts[4] != "UTC" {
        return Err(format!("bad tz (want GMT/UTC): {}", parts[4]));
    }
    Ok(days_from_civil(year, mon, day) * 86400 + hh * 3600 + mm * 60 + ss)
}

fn run_cmd(prog: &str, args: &[&str]) -> (bool, String, String) {
    match Command::new(prog).args(args).output() {
        Ok(o) => (
            o.status.success(),
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
        ),
        Err(e) => (false, String::new(), format!("spawn {prog}: {e}")),
    }
}

// ---------- canonical payload ----------

fn build_payload(
    chain_id: &str,
    head: &[u8; 32],
    prev: &[u8; 32],
    from: u64,
    to: u64,
    local_ts: &str,
) -> Vec<u8> {
    let mut p = Vec::with_capacity(128 + chain_id.len() + local_ts.len());
    p.extend_from_slice(&(chain_id.len() as u32).to_be_bytes());
    p.extend_from_slice(chain_id.as_bytes());
    p.extend_from_slice(head);
    p.extend_from_slice(prev);
    p.extend_from_slice(&from.to_be_bytes());
    p.extend_from_slice(&to.to_be_bytes());
    p.extend_from_slice(&(to.saturating_sub(from).saturating_add(1)).to_be_bytes());
    p.extend_from_slice(&(local_ts.len() as u32).to_be_bytes());
    p.extend_from_slice(local_ts.as_bytes());
    p
}

// ---------- openssl text parsers ----------

#[derive(Debug, Default)]
struct TsrInfo {
    status_granted: bool,
    hash_algo: String,
    serial: String,
    time_str: String,
    ordering_yes: bool,
    nonce: String,
}

fn parse_tsr_text(t: &str) -> TsrInfo {
    let mut info = TsrInfo::default();
    for line in t.lines() {
        let l = line.trim();
        if let Some(v) = l.strip_prefix("Status:") {
            info.status_granted = v.trim().starts_with("Granted");
        } else if let Some(v) = l.strip_prefix("Hash Algorithm:") {
            info.hash_algo = v.trim().to_lowercase();
        } else if let Some(v) = l.strip_prefix("Serial number:") {
            info.serial = v.trim().to_string();
        } else if let Some(v) = l.strip_prefix("Time stamp:") {
            info.time_str = v.trim().to_string();
        } else if let Some(v) = l.strip_prefix("Ordering:") {
            info.ordering_yes = v.trim() == "yes";
        } else if let Some(v) = l.strip_prefix("Nonce:") {
            let nv = v.trim();
            info.nonce = if nv.eq_ignore_ascii_case("unspecified") {
                String::new()
            } else {
                nv.to_string()
            };
        }
    }
    info
}

fn parse_req_nonce(t: &str) -> String {
    for line in t.lines() {
        let l = line.trim();
        if let Some(v) = l.strip_prefix("Nonce:") {
            return v.trim().to_string();
        }
    }
    String::new()
}

// ---------- CLI ----------

fn usage() -> ! {
    eprintln!(
        "anchor 0.1.1 (W0-ANCHOR-IMPL v1.1 C)\n\
         usage:\n  \
         anchor create --chain-id ID --chain-head HEX32 --entry-from N --entry-to M \\\n    \
         --tsa URL --pin FP [--tsa URL --pin FP]... [--db PATH] [--tsr-dir DIR] \\\n    \
         [--tsa-timeout SEC] [--quorum K] [--authorized-by NAME] [--pin-change-reason TXT]\n  \
         anchor verify --db PATH --tsr-dir DIR --pin FP [--chain-id ID] [--expected-fp HEX]\n\
         notes: --pin is MANDATORY (ANC-5: verifier refuses to run without it).\n  \
         --quorum K = min TSA successes for ANCHORED (default = all listed, M-of-M).\n  \
         A4 (v1.1): first pin use per TSA seeds pin_change_log (needs --authorized-by);\n  \
         pin rotation needs --authorized-by + --pin-change-reason; verify enforces\n  \
         audit-pin == last pin_change_log row (ANC-8/ANC-9) and --expected-fp if given."
    );
    std::process::exit(2);
}

fn get_arg(args: &[String], i: &mut usize, name: &str) -> String {
    *i += 1;
    if *i >= args.len() {
        eprintln!("missing value for {name}");
        usage();
    }
    args[*i].clone()
}

fn open_db(path: &str) -> Result<Connection, String> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir db dir: {e}"))?;
    }
    let conn = Connection::open(path).map_err(|e| format!("open db {path}: {e}"))?;
    conn.execute_batch(DDL)
        .map_err(|e| format!("DDL: {e}"))?;
    Ok(conn)
}

/// Last ANCHORED/PARTIAL row for chain: (head, entry_to). None = first anchor.
fn last_anchored(conn: &Connection, chain: &str) -> Result<Option<(Vec<u8>, u64)>, String> {
    let mut st = conn
        .prepare(
            "SELECT chain_head, entry_to FROM anchor_log WHERE chain_id=?1 \
             AND status IN ('ANCHORED','PARTIAL') ORDER BY anchor_id DESC LIMIT 1",
        )
        .map_err(|e| format!("prepare: {e}"))?;
    let mut rows = st.query([chain]).map_err(|e| format!("query: {e}"))?;
    if let Some(r) = rows.next().map_err(|e| format!("row: {e}"))? {
        let h: Vec<u8> = r.get(0).map_err(|e| format!("col: {e}"))?;
        let to: u64 = r.get(1).map_err(|e| format!("col: {e}"))?;
        Ok(Some((h, to)))
    } else {
        Ok(None)
    }
}

fn pin_fingerprint(pin_pem_path: &str) -> Result<String, String> {
    // Resolve --pin value: it may be a hex fingerprint OR a path to a PEM file
    // containing the pinned CA cert. If path exists, extract its fingerprint.
    if Path::new(pin_pem_path).exists()
        && fs::read(pin_pem_path)
            .map(|b| b.starts_with(b"-----BEGIN"))
            .unwrap_or(false)
    {
        let (ok, out, err) = run_cmd(
            "openssl",
            &["x509", "-in", pin_pem_path, "-noout", "-fingerprint", "-sha256"],
        );
        if !ok {
            return Err(format!("openssl x509 fingerprint failed: {err}"));
        }
        // "sha256 Fingerprint=AA:BB:..."
        let fp = out
            .split('=')
            .nth(1)
            .ok_or_else(|| format!("bad fingerprint output: {out:?}"))?;
        Ok(norm_fp(fp))
    } else {
        Ok(norm_fp(pin_pem_path))
    }
}

/// Last pin_change_log row for a TSA: (change_id, new_fp). None = never trusted.
fn last_pin_row(conn: &Connection, tsa_id: &str) -> Result<Option<(i64, String)>, String> {
    let mut st = conn
        .prepare("SELECT change_id, new_fp FROM pin_change_log WHERE tsa_id=?1 ORDER BY change_id DESC LIMIT 1")
        .map_err(|e| format!("prepare pin log: {e}"))?;
    let mut rows = st.query([tsa_id]).map_err(|e| format!("query pin log: {e}"))?;
    if let Some(r) = rows.next().map_err(|e| format!("row pin log: {e}"))? {
        Ok(Some((
            r.get(0).map_err(|e| format!("col: {e}"))?,
            r.get(1).map_err(|e| format!("col: {e}"))?,
        )))
    } else {
        Ok(None)
    }
}

fn insert_pin_row(
    conn: &Connection,
    tsa_id: &str,
    old_fp: Option<&str>,
    new_fp: &str,
    reason: &str,
    authorized_by: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO pin_change_log (tsa_id,old_fp,new_fp,reason,authorized_by,changed_utc) VALUES (?1,?2,?3,?4,?5,?6)",
        params![tsa_id, old_fp, new_fp, reason, authorized_by, epoch_to_iso_utc(now_epoch())],
    )
    .map_err(|e| format!("insert pin log: {e}"))?;
    Ok(())
}

/// Verify one TSR against payload bytes + expected pin. Returns TsrInfo or reason.
fn verify_tsr(
    payload_path: &str,
    tsr_path: &str,
    ca_pin_pem: &str,
    expect_fp: &str,
    expect_nonce_norm: &str,
    now: i64,
    verbose: bool,
) -> Result<TsrInfo, String> {
    // V1: openssl verify with explicit CAfile pin.
    let (ok, out, err) = run_cmd(
        "openssl",
        &[
            "ts", "-verify", "-data", payload_path, "-in", tsr_path, "-CAfile", ca_pin_pem,
        ],
    );
    let combined = format!("{out}\n{err}");
    if !ok || !combined.contains("Verification: OK") {
        return Err(format!("V1 openssl verify failed: {}", combined.trim()));
    }
    // Parse TSR text for V2..V7.
    let (ok, tout, terr) = run_cmd("openssl", &["ts", "-reply", "-in", tsr_path, "-text"]);
    if !ok {
        return Err(format!("V2 tsr -text failed: {terr}"));
    }
    let info = parse_tsr_text(&tout);
    if verbose {
        eprintln!("tsr: granted={} algo={} serial={} time={} ordering={} nonce={}",
            info.status_granted, info.hash_algo, info.serial, info.time_str,
            info.ordering_yes, info.nonce);
    }
    // V2 status granted.
    if !info.status_granted {
        return Err("V2 status != Granted".to_string());
    }
    // V3 hash algo sha256 (anti-downgrade).
    if info.hash_algo != "sha256" {
        return Err(format!("V3 hash algo is {:?}, want sha256", info.hash_algo));
    }
    // V4 pin fingerprint: fingerprint of the CAfile must equal expected.
    let (ok, fout, ferr) = run_cmd(
        "openssl",
        &["x509", "-in", ca_pin_pem, "-noout", "-fingerprint", "-sha256"],
    );
    if !ok {
        return Err(format!("V4 pin fingerprint failed: {ferr}"));
    }
    let actual_fp = norm_fp(fout.split('=').nth(1).unwrap_or(""));
    if actual_fp != expect_fp || actual_fp.is_empty() {
        return Err("V4 pin fingerprint != expected (A3/A4)".to_string());
    }
    // V5 nonce match.
    if info.nonce.is_empty() {
        return Err("V5 TSR has no nonce".to_string());
    }
    if norm_nonce(&info.nonce) != expect_nonce_norm {
        return Err("V5 nonce mismatch (replay/precomputed TSR)".to_string());
    }
    // V6 ordering.
    if !info.ordering_yes {
        return Err("V6 Ordering != yes".to_string());
    }
    // V7 clock tolerance.
    let tsa_epoch = parse_tsa_time(&info.time_str)?;
    if (tsa_epoch - now).abs() > CLOCK_TOL_SECS {
        return Err(format!(
            "V7 tsa_time outside ±{CLOCK_TOL_SECS}s (tsa={tsa_epoch} local={now})"
        ));
    }
    Ok(info)
}

fn cmd_create(args: &[String]) -> i32 {
    let mut chain_id = String::new();
    let mut head_hex = String::new();
    let mut from: u64 = 0;
    let mut to: u64 = 0;
    let mut from_set = false;
    let mut to_set = false;
    let mut tsas: Vec<(String, String)> = vec![]; // (url, pin-value-or-pem)
    let mut db = DEFAULT_DB.to_string();
    let mut tsr_dir = DEFAULT_TSR_DIR.to_string();
    let mut timeout = TSA_TIMEOUT_SECS;
    let mut quorum_min: Option<usize> = None;
    let mut authorized_by: Option<String> = None;
    let mut pin_change_reason: Option<String> = None;
    let mut verbose = false;
    let mut i = 1;
    let mut pending_url: Option<String> = None;
    while i < args.len() {
        match args[i].as_str() {
            "--chain-id" => chain_id = get_arg(args, &mut i, "--chain-id"),
            "--chain-head" => head_hex = get_arg(args, &mut i, "--chain-head"),
            "--entry-from" => {
                from = get_arg(args, &mut i, "--entry-from").parse().unwrap_or_else(|_| usage());
                from_set = true;
            }
            "--entry-to" => {
                to = get_arg(args, &mut i, "--entry-to").parse().unwrap_or_else(|_| usage());
                to_set = true;
            }
            "--tsa" => {
                if pending_url.is_some() {
                    eprintln!("each --tsa needs a following --pin");
                    usage();
                }
                pending_url = Some(get_arg(args, &mut i, "--tsa"));
            }
            "--pin" => {
                let pin = get_arg(args, &mut i, "--pin");
                match pending_url.take() {
                    Some(u) => tsas.push((u, pin)),
                    None => {
                        eprintln!("--pin without preceding --tsa");
                        usage();
                    }
                }
            }
            "--db" => db = get_arg(args, &mut i, "--db"),
            "--tsr-dir" => tsr_dir = get_arg(args, &mut i, "--tsr-dir"),
            "--tsa-timeout" => {
                timeout = get_arg(args, &mut i, "--tsa-timeout").parse().unwrap_or_else(|_| usage());
            }
            "--quorum" => {
                quorum_min = Some(get_arg(args, &mut i, "--quorum").parse().unwrap_or_else(|_| usage()));
            }
            "--authorized-by" => authorized_by = Some(get_arg(args, &mut i, "--authorized-by")),
            "--pin-change-reason" => {
                pin_change_reason = Some(get_arg(args, &mut i, "--pin-change-reason"))
            }
            "--verbose" => verbose = true,
            _ => {
                eprintln!("unknown arg: {}", args[i]);
                usage();
            }
        }
        i += 1;
    }
    if chain_id.is_empty() || head_hex.is_empty() || !from_set || !to_set || tsas.is_empty() {
        eprintln!("create needs --chain-id --chain-head --entry-from --entry-to --tsa --pin");
        usage();
    }
    if to < from {
        eprintln!("entry-to < entry-from");
        std::process::exit(2);
    }
    let head = match hex_decode(&head_hex) {
        Ok(h) if h.len() == 32 => {
            let mut a = [0u8; 32];
            a.copy_from_slice(&h);
            a
        }
        _ => {
            eprintln!("--chain-head must be 64 hex chars (32 bytes)");
            std::process::exit(2);
        }
    };
    let quorum_need = quorum_min.unwrap_or(tsas.len()); // MVP default: M-of-M
    if quorum_need == 0 || quorum_need > tsas.len() {
        eprintln!("--quorum must be within 1..{}", tsas.len());
        std::process::exit(2);
    }

    // DB + continuity (V8).
    let conn = match open_db(&db) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("db: {e}");
            return 1;
        }
    };
    let (prev, v8_ok) = match last_anchored(&conn, &chain_id) {
        Ok(Some((h, prev_to))) => {
            if h.len() != 32 {
                eprintln!("stored chain_head corrupt (len {})", h.len());
                return 1;
            }
            let mut a = [0u8; 32];
            a.copy_from_slice(&h);
            (a, from == prev_to + 1)
        }
        Ok(None) => ([0u8; 32], true), // first anchor: prev = zeros
        Err(e) => {
            eprintln!("db: {e}");
            return 1;
        }
    };
    if !v8_ok {
        eprintln!("V8 range gap: entry_from={from} does not continue previous anchor (fail-closed)");
        std::process::exit(2);
    }

    if let Err(e) = fs::create_dir_all(&tsr_dir) {
        eprintln!("mkdir tsr-dir: {e}");
        return 1;
    }
    // Work dir for temp files.
    let work = PathBuf::from(&tsr_dir).join(format!("work-{}-{}", std::process::id(), now_epoch()));
    if let Err(e) = fs::create_dir_all(&work) {
        eprintln!("mkdir work: {e}");
        return 1;
    }

    // local_ts == created_utc (single clock read; see module docs).
    let now = now_epoch();
    let local_ts = epoch_to_iso_utc(now);
    let payload = build_payload(&chain_id, &head, &prev, from, to, &local_ts);
    let payload_path = work.join("payload.bin");
    if let Err(e) = fs::write(&payload_path, &payload) {
        eprintln!("write payload: {e}");
        return 1;
    }
    let imprint = Sha256::digest(&payload);
    if verbose {
        eprintln!("payload {} bytes imprint={} local_ts={local_ts}", payload.len(), hex_encode(&imprint));
    }
    let payload_s = payload_path.to_string_lossy().into_owned();

    // Per-TSA: query -> POST -> verify-before-record (§5.1 steps 3-5).
    struct Hit {
        tsa_id: String,
        fp: String,
        info: TsrInfo,
        tsr_bytes: Vec<u8>,
        nonce: Vec<u8>,
    }
    let mut hits: Vec<Hit> = vec![];
    let mut fails: Vec<String> = vec![];
    for (idx, (url, pin_val)) in tsas.iter().enumerate() {
        // ANC-5 analogue at creation: pin must be present (CLI guarantees) and
        // must resolve to a 64-hex fingerprint; refuse on empty.
        let fp = match pin_fingerprint(pin_val) {
            Ok(f) if f.len() == 64 => f,
            Ok(f) => {
                fails.push(format!("{url}: bad pin length {}", f.len()));
                continue;
            }
            Err(e) => {
                fails.push(format!("{url}: pin: {e}"));
                continue;
            }
        };
        // A4/D-A5 (spec v1.1 C): pin trust is a RECORDED event, not operator
        // discipline. Seed on first use; rotate only with human attribution +
        // reason. Refusal exits 2 BEFORE any anchor row is written.
        match last_pin_row(&conn, url) {
            Ok(None) => {
                let by = authorized_by.as_deref().unwrap_or("").trim();
                if by.is_empty() {
                    eprintln!(
                        "POLICY REFUSAL (A4): first use of pin for {url} needs --authorized-by                          <human> to seed pin_change_log."
                    );
                    std::process::exit(2);
                }
                if let Err(e) = insert_pin_row(&conn, url, None, &fp, "bootstrap", by) {
                    eprintln!("pin_change_log seed: {e}");
                    return 1;
                }
            }
            Ok(Some((_id, last_fp))) if norm_fp(&last_fp) == fp => {}
            Ok(Some((_id, last_fp))) => {
                let by = authorized_by.as_deref().unwrap_or("").trim();
                let reason = pin_change_reason.as_deref().unwrap_or("").trim();
                if by.is_empty() || reason.is_empty() {
                    eprintln!(
                        "POLICY REFUSAL (A4): pin for {url} differs from recorded last                          pin_change_log row. Rotation needs --authorized-by <human>                          --pin-change-reason <why>."
                    );
                    std::process::exit(2);
                }
                if let Err(e) =
                    insert_pin_row(&conn, url, Some(&norm_fp(&last_fp)), &fp, reason, by)
                {
                    eprintln!("pin_change_log rotation: {e}");
                    return 1;
                }
            }
            Err(e) => {
                eprintln!("pin_change_log: {e}");
                return 1;
            }
        }
        // Resolve CAfile path for openssl: if pin was a PEM path use it,
        // else we need a PEM file — require <tsr-dir>/pins/<n>.pem? NO:
        // honest rule: --pin accepts a PEM path OR hex, but openssl -CAfile
        // needs a file. If hex given, look for pins/<fp>.pem in tsr_dir.
        let cafile: String = if Path::new(pin_val).exists()
            && fs::read(pin_val).map(|b| b.starts_with(b"-----BEGIN")).unwrap_or(false)
        {
            pin_val.clone()
        } else {
            let p = PathBuf::from(&tsr_dir).join("pins").join(format!("{fp}.pem"));
            if !p.exists() {
                fails.push(format!(
                    "{url}: pin hex given but no PEM at {} (place pinned CA cert there)",
                    p.display()
                ));
                continue;
            }
            p.to_string_lossy().into_owned()
        };
        // Step 3: tsquery with nonce.
        let req = work.join(format!("req{idx}.tsq"));
        let req_s = req.to_string_lossy().into_owned();
        let (ok, _o, e) = run_cmd(
            "openssl",
            &["ts", "-query", "-data", &payload_s, "-sha256", "-cert", "-out", &req_s],
        );
        if !ok {
            fails.push(format!("{url}: ts -query failed: {e}"));
            continue;
        }
        let (ok, qtext, _) = run_cmd("openssl", &["ts", "-query", "-in", &req_s, "-text"]);
        if !ok {
            fails.push(format!("{url}: req -text failed"));
            continue;
        }
        let sent_nonce = norm_nonce(&parse_req_nonce(&qtext));
        if sent_nonce.is_empty() || sent_nonce == "0" {
            fails.push(format!("{url}: request has no nonce"));
            continue;
        }
        // Step 4: POST.
        let tsr = work.join(format!("resp{idx}.tsr"));
        let tsr_s = tsr.to_string_lossy().into_owned();
        let timeout_s = timeout.to_string();
        let (ok, code, cerr) = run_cmd(
            "curl",
            &[
                "-s", "-S", "--max-time", &timeout_s, "-H",
                "Content-Type: application/timestamp-query", "--data-binary",
                &format!("@{req_s}"), url, "-o", &tsr_s, "-w", "%{http_code}",
            ],
        );
        if !ok || code.trim() != "200" {
            fails.push(format!("{url}: POST failed (curl_ok={ok} http={}) {}", code.trim(), cerr.trim()));
            continue;
        }
        let tsr_bytes = match fs::read(&tsr) {
            Ok(b) if !b.is_empty() => b,
            _ => {
                fails.push(format!("{url}: empty TSR body"));
                continue;
            }
        };
        // Step 5: verify-before-record (V1..V7; V8 already enforced).
        match verify_tsr(&payload_s, &tsr_s, &cafile, &fp, &sent_nonce, now, verbose) {
            Ok(info) => {
                let nonce_raw = hex_decode(&sent_nonce).unwrap_or_default();
                hits.push(Hit { tsa_id: url.clone(), fp, info, tsr_bytes, nonce: nonce_raw });
            }
            Err(reason) => fails.push(format!("{url}: {reason}")),
        }
    }

    // Step 6: record (one row per TSA attempt; ANCHORED/PARTIAL/FAILED).
    let status = if hits.len() >= quorum_need {
        "ANCHORED"
    } else if !hits.is_empty() {
        "PARTIAL"
    } else {
        "FAILED"
    };
    let quorum_s = format!("{}-of-{}", hits.len(), tsas.len());
    // Store one row per TSA: success rows carry TSR; failed rows carry zeros +
    // empty strings (documented: zeros = no TSR received).
    let mut anchor_ids: Vec<i64> = vec![];
    for h in &hits {
        let tsr_hash: [u8; 32] = Sha256::digest(&h.tsr_bytes).into();
        // tsr filename content-addressed.
        let fname = format!("tsr-{}.der", hex_encode(&tsr_hash));
        let fpath = PathBuf::from(&tsr_dir).join(&fname);
        if let Err(e) = fs::write(&fpath, &h.tsr_bytes) {
            eprintln!("write tsr: {e}");
            return 1;
        }
        let nonce_stored = if h.nonce.len() >= 8 { h.nonce.clone() } else { vec![0u8; 8] };
        match conn.execute(
            "INSERT INTO anchor_log (chain_id,chain_head,prev_anchor_head,entry_from,\
             entry_to,entry_count,imprint_algo,nonce,tsa_id,tsa_cert_fp,tsa_serial,\
             tsa_time,tsr_sha256,tsr_path,quorum,status,created_utc) \
             VALUES (?1,?2,?3,?4,?5,?6,'sha256',?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                chain_id, head.to_vec(), prev.to_vec(), from as i64, to as i64,
                (to - from + 1) as i64, nonce_stored, h.tsa_id, h.fp,
                h.info.serial, h.info.time_str, tsr_hash.to_vec(),
                fpath.to_string_lossy().into_owned(), quorum_s, status, local_ts,
            ],
        ) {
            Ok(_) => anchor_ids.push(conn.last_insert_rowid()),
            Err(e) => {
                eprintln!("insert: {e}");
                return 1;
            }
        }
    }
    for f in &fails {
        // Parse TSA url prefix "url: reason".
        let tsa_id = f.split_once(": ").map(|(u, _)| u).unwrap_or("?").to_string();
        let zeros = vec![0u8; 32];
        if let Err(e) = conn.execute(
            "INSERT INTO anchor_log (chain_id,chain_head,prev_anchor_head,entry_from,\
             entry_to,entry_count,imprint_algo,nonce,tsa_id,tsa_cert_fp,tsa_serial,\
             tsa_time,tsr_sha256,tsr_path,quorum,status,created_utc) \
             VALUES (?1,?2,?3,?4,?5,?6,'sha256',?7,?8,'','','',?9,'',?10,'FAILED',?11)",
            params![
                chain_id, head.to_vec(), prev.to_vec(), from as i64, to as i64,
                (to - from + 1) as i64, vec![0u8; 8], tsa_id, zeros, quorum_s, local_ts,
            ],
        ) {
            eprintln!("insert FAILED row: {e}");
            return 1;
        }
        anchor_ids.push(conn.last_insert_rowid());
        eprintln!("ALARM anchor FAILED [{tsa_id}]: {f}");
    }
    let _ = fs::remove_dir_all(&work); // best-effort cleanup, no secrets inside
    println!("status={status} quorum={quorum_s} anchors={anchor_ids:?} chain={chain_id} [{from}..{to}]");
    for f in &fails {
        println!("fail: {f}");
    }
    if status == "FAILED" {
        // D-A1 default: fail-open ops — machine continues; claim downgraded.
        eprintln!("ALARM: window [{from}..{to}] UNANCHORED (claim downgraded, execution continues)");
    }
    0 // exit 0 even on FAILED: non-blocking by design (ANC-7)
}

fn cmd_verify(args: &[String]) -> i32 {
    let mut db = String::new();
    let mut tsr_dir = String::new();
    let mut pin_val: Option<String> = None;
    let mut chain_filter: Option<String> = None;
    let mut expected_fp_arg: Option<String> = None;
    let mut verbose = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--db" => db = get_arg(args, &mut i, "--db"),
            "--tsr-dir" => tsr_dir = get_arg(args, &mut i, "--tsr-dir"),
            "--pin" => pin_val = Some(get_arg(args, &mut i, "--pin")),
            "--chain-id" => chain_filter = Some(get_arg(args, &mut i, "--chain-id")),
            "--expected-fp" => expected_fp_arg = Some(get_arg(args, &mut i, "--expected-fp")),
            "--verbose" => verbose = true,
            _ => {
                eprintln!("unknown arg: {}", args[i]);
                usage();
            }
        }
        i += 1;
    }
    // ANC-5: refuse to run without explicit pin. This is POLICY, not crypto.
    let pin_val = match pin_val {
        Some(p) if !p.trim().is_empty() => p,
        _ => {
            eprintln!("POLICY REFUSAL (ANC-5): --pin <fingerprint-or-PEM> is mandatory. \
                The verifier MUST NOT run against the system trust store.");
            std::process::exit(2);
        }
    };
    if db.is_empty() || tsr_dir.is_empty() {
        eprintln!("verify needs --db --tsr-dir --pin");
        usage();
    }
    let expect_fp = match pin_fingerprint(&pin_val) {
        Ok(f) if f.len() == 64 => f,
        _ => {
            eprintln!("--pin must resolve to a 64-hex SHA-256 fingerprint");
            std::process::exit(2);
        }
    };
    // D-A5 (spec v1.1 C): the out-of-band owner constant beats host state.
    if let Some(ef) = expected_fp_arg.as_deref() {
        let ef = norm_fp(ef);
        if ef.len() != 64 {
            eprintln!("--expected-fp must be 64 hex chars");
            std::process::exit(2);
        }
        if expect_fp != ef {
            eprintln!("ALARM PIN-MISMATCH: loaded pin != --expected-fp (A4)");
            std::process::exit(2);
        }
    }
    // CAfile for openssl: PEM path required.
    let cafile: String = if Path::new(&pin_val).exists()
        && fs::read(&pin_val).map(|b| b.starts_with(b"-----BEGIN")).unwrap_or(false)
    {
        pin_val.clone()
    } else {
        let p = PathBuf::from(&tsr_dir).join("pins").join(format!("{expect_fp}.pem"));
        if !p.exists() {
            eprintln!("pin hex given but no PEM at {} (ANC-5: explicit pin required)", p.display());
            std::process::exit(2);
        }
        p.to_string_lossy().into_owned()
    };
    let conn = match open_db(&db) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("db: {e}");
            return 1;
        }
    };
    let sql = if chain_filter.is_some() {
        "SELECT anchor_id,chain_id,chain_head,prev_anchor_head,entry_from,entry_to,\
         entry_count,nonce,tsa_id,tsa_cert_fp,tsr_sha256,tsr_path,status,created_utc \
         FROM anchor_log WHERE chain_id=?1 ORDER BY anchor_id"
    } else {
        "SELECT anchor_id,chain_id,chain_head,prev_anchor_head,entry_from,entry_to,\
         entry_count,nonce,tsa_id,tsa_cert_fp,tsr_sha256,tsr_path,status,created_utc \
         FROM anchor_log ORDER BY anchor_id"
    };
    let mut st = match conn.prepare(sql) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("prepare: {e}");
            return 1;
        }
    };
    let rows: Vec<AnchorRow> = (|| -> Result<_, String> {
        let mut out = vec![];
        if let Some(cf) = chain_filter.as_deref() {
            let mut r = st.query([cf]).map_err(|e| format!("query: {e}"))?;
            while let Some(row) = r.next().map_err(|e| format!("row: {e}"))? {
                out.push(read_row(row)?);
            }
        } else {
            let mut r = st
                .query(rusqlite::params![])
                .map_err(|e| format!("query: {e}"))?;
            while let Some(row) = r.next().map_err(|e| format!("row: {e}"))? {
                out.push(read_row(row)?);
            }
        }
        Ok(out)
    })()
    .unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    fn read_row(r: &rusqlite::Row) -> Result<AnchorRow, String> {
        Ok((
            r.get(0).map_err(|e| format!("c0: {e}"))?,
            r.get(1).map_err(|e| format!("c1: {e}"))?,
            r.get(2).map_err(|e| format!("c2: {e}"))?,
            r.get(3).map_err(|e| format!("c3: {e}"))?,
            r.get(4).map_err(|e| format!("c4: {e}"))?,
            r.get(5).map_err(|e| format!("c5: {e}"))?,
            r.get(6).map_err(|e| format!("c6: {e}"))?,
            r.get(7).map_err(|e| format!("c7: {e}"))?,
            r.get(8).map_err(|e| format!("c8: {e}"))?,
            r.get(9).map_err(|e| format!("c9: {e}"))?,
            r.get(10).map_err(|e| format!("c10: {e}"))?,
            r.get(11).map_err(|e| format!("c11: {e}"))?,
            r.get(12).map_err(|e| format!("c12: {e}"))?,
            r.get(13).map_err(|e| format!("c13: {e}"))?,
        ))
    }

    // A4 enforcement (spec v1.1 C): every TSA with a live (non-FAILED) row
    // must have a recorded pin row, and the audit pin must equal its LAST
    // row. An empty log REFUSES: deleting history must never widen trust.
    // Verify stays read-only (SELECT only).
    {
        use std::collections::BTreeSet;
        let mut tsas = BTreeSet::new();
        for r in &rows {
            if r.12 != "FAILED" {
                tsas.insert(r.8.clone());
            }
        }
        for tsa in &tsas {
            match last_pin_row(&conn, tsa) {
                Ok(Some((_id, last_fp))) if norm_fp(&last_fp) == expect_fp => {}
                Ok(Some((_id, _))) => {
                    eprintln!(
                        "ALARM PIN-MISMATCH: audit pin != last pin_change_log row for {tsa} (A4)"
                    );
                    std::process::exit(2);
                }
                Ok(None) => {
                    eprintln!(
                        "ALARM PIN-LOG-EMPTY: no pin_change_log row for {tsa}; refusing to verify                          unrecorded trust (A4)"
                    );
                    std::process::exit(2);
                }
                Err(e) => {
                    eprintln!("pin_change_log: {e}");
                    return 1;
                }
            }
        }
    }
    let now = now_epoch();
    let work = PathBuf::from(&tsr_dir).join(format!("vwork-{}-{now}", std::process::id()));
    if let Err(e) = fs::create_dir_all(&work) {
        eprintln!("mkdir: {e}");
        return 1;
    }
    let mut bad = 0;
    let mut prev_ok_to: Option<(String, u64)> = None; // (chain, entry_to) of last good row
    for (id, chain, head, prev, from, to, count, nonce, tsa, row_fp, tsr_hash, tsr_path, status, created)
        in &rows
    {
        // FAILED rows: report, don't re-verify crypto (nothing to verify).
        if status == "FAILED" {
            println!("anchor {id} [{chain} {from}..{to}] {tsa}: UNANCHORED (recorded FAILED)");
            continue;
        }
        let mut why = String::new();
        // Rebuild payload from row (local_ts == created_utc identity).
        let ok = head.len() == 32 && prev.len() == 32;
        if !ok {
            why = "stored head/prev not 32B".into();
        }
        let payload = if ok {
            let (mut h, mut p) = ([0u8; 32], [0u8; 32]);
            h.copy_from_slice(head);
            p.copy_from_slice(prev);
            build_payload(chain, &h, &p, *from as u64, *to as u64, created)
        } else {
            vec![]
        };
        if *count as u64 != (*to as u64).saturating_sub(*from as u64).saturating_add(1) {
            why = "entry_count mismatch".into();
        }
        // TSR file present + hash matches?
        let tsr_bytes = fs::read(tsr_path).unwrap_or_default();
        if tsr_bytes.is_empty() {
            why = format!("missing TSR file {tsr_path}");
        } else {
            let h: [u8; 32] = Sha256::digest(&tsr_bytes).into();
            if h.to_vec() != *tsr_hash {
                why = "tsr_sha256 mismatch (TSR file altered)".into();
            }
        }
        // Row pin must equal the pin we were told to trust (V4 at audit).
        if norm_fp(row_fp) != expect_fp {
            why = format!("row pin {row_fp} != audit pin (refuses cross-pin trust)");
        }
        // V1..V7 crypto re-verification.
        if why.is_empty() {
            let pp = work.join(format!("p{id}.bin"));
            if fs::write(&pp, &payload).is_err() {
                why = "workdir write failed".into();
            } else {
                let expect_nonce = norm_nonce(&hex_encode(nonce));
                match verify_tsr(
                    &pp.to_string_lossy(), tsr_path, &cafile, &expect_fp, &expect_nonce, now,
                    verbose,
                ) {
                    Ok(_) => {}
                    Err(e) => why = e,
                }
            }
        }
        // V8 continuity vs previous GOOD row of same chain.
        if why.is_empty() {
            if let Some((pc, pto)) = &prev_ok_to {
                if pc == chain && *from as u64 != pto + 1 {
                    why = format!("V8 gap: from={from} prev_to={pto}");
                }
            }
            // prev_anchor_head must equal previous good row's head (chain of log).
            if why.is_empty() {
                if let Some((pc, _)) = &prev_ok_to {
                    if pc == chain {
                        // fetch previous good head
                        let mut st2 = conn.prepare(
                            "SELECT chain_head FROM anchor_log WHERE chain_id=?1 \
                             AND status IN ('ANCHORED','PARTIAL') AND anchor_id < ?2 \
                             ORDER BY anchor_id DESC LIMIT 1",
                        ).unwrap();
                        let ph: Vec<u8> = st2.query_row([chain, &id.to_string()], |r| r.get(0)).unwrap_or_default();
                        if !ph.is_empty() && ph != *prev {
                            why = "prev_anchor_head != previous row head (log chain broken)".into();
                        }
                    }
                } else if prev.iter().any(|&b| b != 0) {
                    why = "first row prev must be zeros".into();
                }
            }
        }
        if why.is_empty() {
            println!("anchor {id} [{chain} {from}..{to}] {tsa}: ANCHORED (re-verified)");
            prev_ok_to = Some((chain.clone(), *to as u64));
        } else {
            println!("anchor {id} [{chain} {from}..{to}] {tsa}: ANCHOR-MISMATCH: {why}");
            bad += 1;
        }
    }
    let _ = fs::remove_dir_all(&work);
    // UNANCHORED tail: report highest anchored entry per chain.
    if verbose {
        eprintln!("note: entries beyond last ANCHORED row per chain = UNANCHORED (never 'verified')");
    }
    if bad > 0 {
        eprintln!("{bad} mismatching anchor(s)");
        1
    } else {
        0
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }
    // ANC-5 global analogue: `verify` without --pin refuses (handled inside).
    let code = match args[1].as_str() {
        "create" => cmd_create(&args[1..]),
        "verify" => cmd_verify(&args[1..]),
        _ => usage(),
    };
    // Flush stdout before exit (println! may panic on broken pipe otherwise).
    let _ = std::io::stdout().flush();
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let b = vec![0u8, 1, 15, 16, 171, 255];
        assert_eq!(hex_decode(&hex_encode(&b)).unwrap(), b);
        assert!(hex_decode("abc").is_err());
        assert!(hex_decode("zz").is_err());
    }

    #[test]
    fn fp_nonce_norm() {
        assert_eq!(norm_fp("AA:bb 01"), "aabb01");
        assert_eq!(norm_fp("0xAABB"), "aabb");
        assert_eq!(norm_nonce("0x00D6BF"), "d6bf");
        assert_eq!(norm_nonce("0"), "0");
    }

    #[test]
    fn epoch_known_values() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 1, 1) * 86400, 946684800);
        assert_eq!(epoch_to_iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // roundtrip a recent date
        let e: i64 = 1788382245; // ~2026-09-09
        let days = e.div_euclid(86400);
        let (y, m, d) = civil_from_days(days);
        assert_eq!(days_from_civil(y, m as i64, d as i64), days);
    }

    #[test]
    fn tsa_time_parse() {
        let e = parse_tsa_time("Sep  9 07:44:01 2026 GMT").unwrap();
        assert_eq!(epoch_to_iso_utc(e), "2026-09-09T07:44:01Z");
        assert!(parse_tsa_time("bogus").is_err());
    }

    #[test]
    fn payload_deterministic_and_sized() {
        let p1 = build_payload("c", &[1u8; 32], &[2u8; 32], 0, 999, "2026-09-09T00:00:00Z");
        let p2 = build_payload("c", &[1u8; 32], &[2u8; 32], 0, 999, "2026-09-09T00:00:00Z");
        assert_eq!(p1, p2);
        // 4+1 +32+32 +8+8+8 +4+20 = 117
        assert_eq!(p1.len(), 117);
        let p3 = build_payload("c", &[1u8; 32], &[2u8; 32], 0, 1000, "2026-09-09T00:00:00Z");
        assert_ne!(p1, p3);
    }

    #[test]
    fn pin_log_seed_last_rotate() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(DDL).unwrap();
        assert!(last_pin_row(&conn, "https://t/tsr").unwrap().is_none());
        insert_pin_row(&conn, "https://t/tsr", None, &"a".repeat(64), "bootstrap", "gates").unwrap();
        let (id, fp) = last_pin_row(&conn, "https://t/tsr").unwrap().unwrap();
        assert_eq!(id, 1);
        assert_eq!(fp, "a".repeat(64));
        insert_pin_row(
            &conn, "https://t/tsr", Some(&"a".repeat(64)), &"b".repeat(64), "rotation", "gates",
        )
        .unwrap();
        let (id2, fp2) = last_pin_row(&conn, "https://t/tsr").unwrap().unwrap();
        assert_eq!(id2, 2);
        assert_eq!(fp2, "b".repeat(64));
        // Other TSA unaffected.
        assert!(last_pin_row(&conn, "https://x/tsr").unwrap().is_none());
    }

    #[test]
    fn tsr_text_parse() {
        let t = "Status info:\n    Status: Granted.\nTST info:\n    Hash Algorithm: sha256\n    \
                 Serial number: 0x07C1\n    Time stamp: Sep  9 07:44:01 2026 GMT\n    \
                 Ordering: yes\n    Nonce: 0xD6BF\n";
        let i = parse_tsr_text(t);
        assert!(i.status_granted);
        assert_eq!(i.hash_algo, "sha256");
        assert_eq!(i.serial, "0x07C1");
        assert!(i.ordering_yes);
        assert_eq!(norm_nonce(&i.nonce), "d6bf");
    }
}
