#![forbid(unsafe_code)]
//! # W2-DETERM-ENFORCE — `RecordSet` v1 (Record-Replay Determinism Enforcement)
//!
//! Kontrak: `docs/AGENT10-W2-DETERM-ENFORCE-SPEC.md` v1.1-PREP (dinaikkan saat
//! implementasi; keputusan D-D1..D-D4 dinyatakan di README).
//!
//! Standar tipe merujuk tipe NYATA kernel (Ruling 3(c) @matt #1000):
//! - Hash BLAKE3 = `[u8; 32]` / `blake3_hex` (64-hex lowercase)
//! - Checksum berkas = SHA-256 (kontrak G-C7, #751)
//! - `RecordBody` = `Inline(Vec<u8>) | Spilled(SpillTag)`
//!
//! Domain separation (C-02): `CTX_RECORD` untuk digest RecordSet, `CTX_BODY`
//! untuk digest isi body (blob). Tidak pernah dicampur dengan digest Envelope
//! atau checksum spill.
//!
//! Perbatasan jujur (spec §1): record-replay membuktikan **reproduksibilitas**,
//! **bukan keaslian**. Bukti audit hanya Execution Envelope + anchor eksternal.

use std::collections::BTreeMap;

pub mod verify;

pub const CTX_RECORD: &[u8] = b"n8nrust/determinism/recordset/v1";
pub const CTX_BODY: &[u8] = b"n8nrust/determinism/recordset/body/v1";
pub const SCHEMA_VERSION: u8 = 1;
pub const SEED_SIZE: usize = 8;
pub const WORKFLOW_HASH_SIZE: usize = 32;

/// Tanda kelas nondeterminisme yang direkam (spec §4).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    HttpResponse = 0x01,
    ClockRead = 0x02,
    RngOutput = 0x03,
    IterOrder = 0x04,
    SysEnvRead = 0x05,
}

impl EntryKind {
    pub fn to_u8(self) -> u8 {
        self as u8
    }
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(EntryKind::HttpResponse),
            0x02 => Some(EntryKind::ClockRead),
            0x03 => Some(EntryKind::RngOutput),
            0x04 => Some(EntryKind::IterOrder),
            0x05 => Some(EntryKind::SysEnvRead),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            EntryKind::HttpResponse => "HTTP_RESPONSE",
            EntryKind::ClockRead => "CLOCK_READ",
            EntryKind::RngOutput => "RNG_OUTPUT",
            EntryKind::IterOrder => "ITER_ORDER",
            EntryKind::SysEnvRead => "SYS_ENV_READ",
        }
    }
}

/// Referensi payload zero-copy. Tipe kanonik per Ruling 3(c) @matt #1000;
/// §2 BAHASA-BERSAMA versi root = NON-BINDING (Ruling 3(a) #1000), jangan dikutip.
///
/// Catatan (erratum F-1, per review #956): kernel KANONIK TIDAK punya
/// `uuid::Uuid` (gate Cargo 4-dep; #924). Tipe NYATA: `ItemList::Spilled(
/// SpilledList{ path: SpillPath, len, total_bytes, codec })` + `ContentId(u64)`
/// (numeric_id!). Newtype `SpillTag(String)` di crate ini hanyalah penanda
/// "payload di spill" agar crate tetap bebas dependensi jaringan; kontrak
/// serialisasi `{"type":"SpillRef","id":"<id>","bytes":N}` mengikuti pola
/// SpillRef pada #940 T-2 (nilai kanon: "inline" | "spilled").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpillTag(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordBody {
    Inline(Vec<u8>),
    Spilled(SpillTag),
}

/// Satu rekaman nondeterminisme (spec §4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub kind: EntryKind,
    pub node_id: String,
    /// Urutan dalam eksekusi (disiplin seq Envelope; u64 BE).
    pub seq: u64,
    /// Payload kanonik per kind (lihat builder* di bawah).
    pub payload: Vec<u8>,
    /// Wall-clock capture — metadata saja, TIDAK pernah menjadi nilai replay.
    pub captured_at: u64,
}

/// RecordSet v1 (spec §4). Semua field BE; tidak ada `Display`/JSON bebas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordSet {
    pub exec_id: String,
    /// SHA-256 checksum berkas template workflow (determinism-record: template_sha256).
    pub workflow_sha256: [u8; 32],
    /// BLAKE3 digest konten workflow (T-11a/T-11b dual-hash interim).
    pub workflow_blake3: [u8; 32],
    /// Seed PRNG 8 byte; semua-nol = unseeded (mode non-deterministik).
    pub seed: [u8; SEED_SIZE],
    pub entries: Vec<Entry>,
}

// ---------------------------------------------------------------------------
// Canonical payload builders (spec §4.1)
// ---------------------------------------------------------------------------

fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}
fn put_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_be_bytes());
}
fn put_bytes(out: &mut Vec<u8>, b: &[u8]) {
    put_u32(out, b.len() as u32);
    out.extend_from_slice(b);
}

/// Header kanonik: key lowercase diurutkan + nilai di-redact bila terdeteksi
/// kredensial (spec §7.3; gate G-D7). Kredensial TIDAK pernah masuk RecordSet.
pub fn redact_headers(headers: &[(String, String)]) -> Vec<(String, String)> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    const SECRET_HDRS: &[&str] = &["authorization", "cookie", "x-api-key", "proxy-authorization"];
    for (k, v) in headers {
        let lk = k.to_lowercase();
        let lv = if SECRET_HDRS.iter().any(|s| *s == lk) {
            "[REDACTED-credential]"
        } else {
            v.as_str()
        };
        map.insert(lk, lv.to_string());
    }
    map.into_iter().collect()
}

/// Payload HTTP_RESPONSE: `{url, method, status, headers*, body_hash B32, body_len u64}`.
pub fn http_response_payload(
    url: &str,
    method: &str,
    status: u16,
    headers: &[(String, String)],
    body_hash: [u8; 32],
    body_len: u64,
) -> Vec<u8> {
    let mut out = Vec::new();
    put_bytes(&mut out, url.as_bytes());
    put_bytes(&mut out, method.as_bytes());
    out.extend_from_slice(&status.to_be_bytes());
    let hdrs = redact_headers(headers);
    put_u32(&mut out, hdrs.len() as u32);
    for (k, v) in hdrs {
        put_bytes(&mut out, k.as_bytes());
        put_bytes(&mut out, v.as_bytes());
    }
    out.extend_from_slice(&body_hash);
    out.extend_from_slice(&body_len.to_be_bytes());
    out
}

/// Payload CLOCK_READ: `{now_ms_utc u64, tz_scope}`.
pub fn clock_read_payload(now_ms_utc: u64, tz_scope: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&now_ms_utc.to_be_bytes());
    put_bytes(&mut out, tz_scope.as_bytes());
    out
}

/// Payload RNG_OUTPUT: hanya `{consumed u64}` — output RNG direkonstruksi dari
/// seed + counter (spec §4.1; tidak menyimpan output).
pub fn rng_output_payload(consumed: u64) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&consumed.to_be_bytes());
    out
}

/// Payload ITER_ORDER: urutan item yang dikunci.
pub fn iter_order_payload(item_ids: &[String]) -> Vec<u8> {
    let mut out = Vec::new();
    put_u32(&mut out, item_ids.len() as u32);
    for id in item_ids {
        put_bytes(&mut out, id.as_bytes());
    }
    out
}

/// Payload SYS_ENV_READ: `{key, value_hash B32}` — env = secret, tidak inline.
pub fn sys_env_read_payload(key: &str, value_hash: [u8; 32]) -> Vec<u8> {
    let mut out = Vec::new();
    put_bytes(&mut out, key.as_bytes());
    out.extend_from_slice(&value_hash);
    out
}

/// BLAKE3 digest body dengan domain separation CTX_BODY.
pub fn body_hash(body: &[u8]) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(CTX_BODY);
    h.update(body);
    *h.finalize().as_bytes()
}

/// `blake3_hex` (64-hex lowercase) — konvensi hex kernel; §2 kamus root NON-BINDING (Ruling 3(a) #1000).
pub fn blake3_hex(b: &[u8]) -> String {
    blake3::hash(b).to_hex().to_string()
}

/// SHA-256 checksum hex (kontrak G-C7 / #751) untuk checksum berkas blob.
pub fn sha256_checksum(b: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b);
    format!("{:x}", h.finalize())
}

// ---------------------------------------------------------------------------
// Encoding (spec §4 kanonik; u32 BE panjang, u64 BE)
// ---------------------------------------------------------------------------

fn encode_entry(out: &mut Vec<u8>, e: &Entry) {
    out.push(e.kind.to_u8());
    put_bytes(out, e.node_id.as_bytes());
    put_u64(out, e.seq);
    put_bytes(out, &e.payload);
    put_u64(out, e.captured_at);
}

impl RecordSet {
    /// Byte stream kanonik SEBELUM field hash (prefix).
    pub fn encode_prefix(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(256);
        out.push(SCHEMA_VERSION);
        put_bytes(&mut out, self.exec_id.as_bytes());
        out.extend_from_slice(&self.workflow_sha256);
        out.extend_from_slice(&self.workflow_blake3);
        out.extend_from_slice(&self.seed);
        put_u32(&mut out, self.entries.len() as u32);
        for e in &self.entries {
            encode_entry(&mut out, e);
        }
        out
    }

    /// `recordset_hash = BLAKE3(CTX_RECORD || prefix)` (spec §4).
    pub fn recordset_hash(&self) -> [u8; 32] {
        let prefix = self.encode_prefix();
        let mut h = blake3::Hasher::new();
        h.update(CTX_RECORD);
        h.update(&prefix);
        *h.finalize().as_bytes()
    }

    /// Berkas lengkap: prefix + 32-byte digest (self-verifying).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.encode_prefix();
        out.extend_from_slice(&self.recordset_hash());
        out
    }

    /// `recordset_sha256` = SHA-256 checksum berkas (metadata determinism-record).
    pub fn recordset_sha256(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(self.encode());
        h.finalize().into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RsError {
    Truncated,
    BadVersion(u8),
    BadKind(u8),
    BadLength { reason: String },
    HashMismatch,
    Utf8,
}

impl std::fmt::Display for RsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RsError::Truncated => write!(f, "RecordSet truncated"),
            RsError::BadVersion(v) => write!(f, "unsupported schema_version {v}"),
            RsError::BadKind(k) => write!(f, "bad entry kind 0x{k:02x}"),
            RsError::BadLength { reason } => write!(f, "bad length: {reason}"),
            RsError::HashMismatch => write!(f, "recordset_hash mismatch"),
            RsError::Utf8 => write!(f, "invalid utf-8"),
        }
    }
}
impl std::error::Error for RsError {}

fn get_u32(b: &[u8], pos: &mut usize) -> Result<u32, RsError> {
    if *pos + 4 > b.len() {
        return Err(RsError::Truncated);
    }
    let v = u32::from_be_bytes([b[*pos], b[*pos + 1], b[*pos + 2], b[*pos + 3]]);
    *pos += 4;
    Ok(v)
}
fn get_u64(b: &[u8], pos: &mut usize) -> Result<u64, RsError> {
    if *pos + 8 > b.len() {
        return Err(RsError::Truncated);
    }
    let v = u64::from_be_bytes(b[*pos..*pos + 8].try_into().unwrap());
    *pos += 8;
    Ok(v)
}
fn get_bytes(b: &[u8], pos: &mut usize) -> Result<Vec<u8>, RsError> {
    let len = get_u32(b, pos)? as usize;
    if *pos + len > b.len() {
        return Err(RsError::BadLength {
            reason: format!("need {len} bytes at {pos}"),
        });
    }
    let v = b[*pos..*pos + len].to_vec();
    *pos += len;
    Ok(v)
}

impl RecordSet {
    /// Decode + verifikasi hash (fail-closed: hash salah => Err(HashMismatch)).
    pub fn decode(b: &[u8]) -> Result<(RecordSet, [u8; 32]), RsError> {
        if b.len() < 1 + 4 + 32 + 32 + 8 + 4 + 32 {
            return Err(RsError::Truncated);
        }
        let split = b.len() - 32;
        let (prefix, tail_hash) = b.split_at(split);
        let mut h = blake3::Hasher::new();
        h.update(CTX_RECORD);
        h.update(prefix);
        let calc: [u8; 32] = *h.finalize().as_bytes();
        if calc != tail_hash {
            return Err(RsError::HashMismatch);
        }
        let mut pos = 0usize;
        let ver = prefix[pos];
        pos += 1;
        if ver != SCHEMA_VERSION {
            return Err(RsError::BadVersion(ver));
        }
        let exec_id = String::from_utf8(get_bytes(prefix, &mut pos)?).map_err(|_| RsError::Utf8)?;
        let mut wf_sha = [0u8; 32];
        wf_sha.copy_from_slice(&prefix[pos..pos + 32]);
        pos += 32;
        let mut wf_b3 = [0u8; 32];
        wf_b3.copy_from_slice(&prefix[pos..pos + 32]);
        pos += 32;
        let mut seed = [0u8; SEED_SIZE];
        seed.copy_from_slice(&prefix[pos..pos + SEED_SIZE]);
        pos += SEED_SIZE;
        let n_entries = get_u32(prefix, &mut pos)? as usize;
        let mut entries = Vec::with_capacity(n_entries.min(1_000_000));
        for _ in 0..n_entries {
            let kind = EntryKind::from_u8(prefix[pos]).ok_or(RsError::BadKind(prefix[pos]))?;
            pos += 1;
            let node_id = String::from_utf8(get_bytes(prefix, &mut pos)?).map_err(|_| RsError::Utf8)?;
            let seq = get_u64(prefix, &mut pos)?;
            let payload = get_bytes(prefix, &mut pos)?;
            let captured_at = get_u64(prefix, &mut pos)?;
            entries.push(Entry { kind, node_id, seq, payload, captured_at });
        }
        if pos != prefix.len() {
            return Err(RsError::BadLength {
                reason: format!("trailing {} bytes", prefix.len() - pos),
            });
        }
        Ok((RecordSet { exec_id, workflow_sha256: wf_sha, workflow_blake3: wf_b3, seed, entries }, calc))
    }
}
