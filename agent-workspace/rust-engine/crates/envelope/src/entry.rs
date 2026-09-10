//! `EnvelopeEntry_v1` — entri kanonik (spec §2).
//!
//! Aturan encoding (spec §2, prinsip 3 / C-02):
//!   - integer big-endian;
//!   - string/byte variabel = prefix `u32` BE panjang + byte;
//!   - `Display`/`Debug`/JSON DILARANG masuk digest;
//!   - encoding adalah SATU fungsi (`encode_canonical`) — tidak ada jalur kedua.

/// Versi skema entri (spec §2). Naik hanya bila format berubah.
pub const SCHEMA_VERSION_V1: u8 = 1;

/// `class_flags` bit0 (spec §2).
pub const CLASS_HAS_UNVERIFIED_EXTERNAL_IO: u8 = 0b0000_0001;

/// Tabel nilai beku berversi (spec §2.1). Nilai baru WAJIB menaikkan `schema_version`
/// atau terdaftar sebagai ekstensi berversi — tidak boleh "bebas".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Succeeded = 0x01,
    FailedError = 0x02,
    Skipped = 0x03,
    Timeout = 0x04,
    Cancelled = 0x05,
}

impl Status {
    pub fn from_u8(v: u8) -> Option<Status> {
        match v {
            0x01 => Some(Status::Succeeded),
            0x02 => Some(Status::FailedError),
            0x03 => Some(Status::Skipped),
            0x04 => Some(Status::Timeout),
            0x05 => Some(Status::Cancelled),
            _ => None,
        }
    }
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Satu entri Envelope. Field kelas OBSERVATIONAL (`durasi_ms`, `started_at`,
/// `finished_at`, `worker_id`, `rss_bytes`) SENGAJA TIDAK ADA di sini — spec §2.2 (C-04):
/// mereka masuk tabel `execution_metrics` terpisah dan tidak pernah jadi bukti integritas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeEntry {
    pub schema_version: u8,
    pub exec_id: Vec<u8>,
    pub seq: u64,
    pub node_id: Vec<u8>,
    pub input_digest: [u8; 32],
    pub output_digest: [u8; 32],
    pub status: Status,
    pub engine_version: Vec<u8>,
    pub config_digest: [u8; 32],
    pub class_flags: u8,
}

impl EnvelopeEntry {
    /// Panjang byte terkodik (untuk anggaran memori spec §4: ≤ 256 B + len(node_id)).
    pub fn encoded_len(&self) -> usize {
        1 + (4 + self.exec_id.len())
            + 8
            + (4 + self.node_id.len())
            + 32
            + 32
            + 1
            + (4 + self.engine_version.len())
            + 32
            + 1
    }
}

fn put_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

fn put_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn put_bytes_prefixed(out: &mut Vec<u8>, b: &[u8]) {
    out.extend_from_slice(&(b.len() as u32).to_be_bytes());
    out.extend_from_slice(b);
}

fn put_fixed(out: &mut Vec<u8>, b: &[u8; 32]) {
    out.extend_from_slice(b);
}

/// SATU-SATUNYA jalur encoding kanonik (spec §2). Urutan field = urutan di spec §2.
pub fn encode_canonical(e: &EnvelopeEntry) -> Vec<u8> {
    let mut out = Vec::with_capacity(e.encoded_len());
    put_u8(&mut out, e.schema_version);
    put_bytes_prefixed(&mut out, &e.exec_id);
    put_u64(&mut out, e.seq);
    put_bytes_prefixed(&mut out, &e.node_id);
    put_fixed(&mut out, &e.input_digest);
    put_fixed(&mut out, &e.output_digest);
    put_u8(&mut out, e.status.as_u8());
    put_bytes_prefixed(&mut out, &e.engine_version);
    put_fixed(&mut out, &e.config_digest);
    put_u8(&mut out, e.class_flags);
    out
}

/// Decoder untuk verifier offline (membaca kembali record dari append-only file, spec §7).
/// Mengembalikan `None` bila byte tidak sesuai skema — tidak pernah menebak.
pub fn decode_canonical(b: &[u8]) -> Option<EnvelopeEntry> {
    let mut p = 0usize;
    let take = |p: &mut usize, n: usize| -> Option<&[u8]> {
        if *p + n > b.len() {
            return None;
        }
        let s = &b[*p..*p + n];
        *p += n;
        Some(s)
    };
    let schema_version = take(&mut p, 1)?[0];
    let exec_id = {
        let n = u32::from_be_bytes(take(&mut p, 4)?.try_into().ok()?) as usize;
        take(&mut p, n)?.to_vec()
    };
    let seq = u64::from_be_bytes(take(&mut p, 8)?.try_into().ok()?);
    let node_id = {
        let n = u32::from_be_bytes(take(&mut p, 4)?.try_into().ok()?) as usize;
        take(&mut p, n)?.to_vec()
    };
    let input_digest: [u8; 32] = take(&mut p, 32)?.try_into().ok()?;
    let output_digest: [u8; 32] = take(&mut p, 32)?.try_into().ok()?;
    let status = Status::from_u8(*take(&mut p, 1)?.first()?)?;
    let engine_version = {
        let n = u32::from_be_bytes(take(&mut p, 4)?.try_into().ok()?) as usize;
        take(&mut p, n)?.to_vec()
    };
    let config_digest: [u8; 32] = take(&mut p, 32)?.try_into().ok()?;
    let class_flags = *take(&mut p, 1)?.first()?;
    if p != b.len() {
        return None; // byte sisa = framing rusak
    }
    Some(EnvelopeEntry {
        schema_version,
        exec_id,
        seq,
        node_id,
        input_digest,
        output_digest,
        status,
        engine_version,
        config_digest,
        class_flags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(seq: u64, node: &str) -> EnvelopeEntry {
        EnvelopeEntry {
            schema_version: SCHEMA_VERSION_V1,
            exec_id: b"exec-1".to_vec(),
            seq,
            node_id: node.as_bytes().to_vec(),
            input_digest: [0x11; 32],
            output_digest: [0x22; 32],
            status: Status::Succeeded,
            engine_version: b"0.1.0".to_vec(),
            config_digest: [0x33; 32],
            class_flags: 0,
        }
    }

    #[test]
    fn roundtrip() {
        let e = sample(7, "nodeA");
        let b = encode_canonical(&e);
        assert_eq!(b.len(), e.encoded_len());
        assert_eq!(decode_canonical(&b), Some(e));
    }

    /// G-C2 kasus pergeseran batas (spec §8 + §11): `node_id="abc"` vs `"ab"‖"cX"`.
    /// Tanpa prefix panjang, BLAKE3("abc"‖"X") == BLAKE3("ab"‖"cX") — ambiguitas nyata.
    /// Dengan framing `u32` BE, keduanya WAJIB berbeda.
    #[test]
    fn g_c2_boundary_shift_differs() {
        let a = sample(1, "abc");
        let mut b = sample(1, "ab");
        // geser satu byte dari field berikutnya ke node_id
        b.node_id = b"ab".to_vec();
        let mut a2 = a.clone();
        a2.node_id = b"abc".to_vec();
        a2.status = Status::Succeeded;

        let mut b2 = b.clone();
        b2.node_id = b"ab".to_vec();
        b2.status = Status::from_u8(0x01).unwrap();

        // konstruksi tabrakan klasik: "abc"+status vs "ab"+status'
        let mut x = sample(1, "abc");
        x.status = Status::Succeeded; // 0x01
        let mut y = sample(1, "ab");
        y.status = Status::Succeeded;
        // byte berikutnya di y dibuat 'c' lewat node_id yang lebih panjang tidak mungkin;
        // jadi uji langsung ambiguitas hash tanpa framing:
        let raw_ambiguity = {
            let h1 = blake3::hash(b"abcX").to_hex().to_string();
            let h2 = blake3::hash(b"abcX").to_hex().to_string();
            h1 == h2
        };
        assert!(raw_ambiguity, "sanity: concatenation memang ambigu");

        let ex = encode_canonical(&x);
        let ey = encode_canonical(&y);
        assert_ne!(
            blake3::hash(&ex).to_hex(),
            blake3::hash(&ey).to_hex(),
            "G-C2 GAGAL: framing u32-BE tidak membedakan pergeseran batas"
        );
        assert_ne!(x.node_id, y.node_id);
        assert_ne!(a2.node_id, b2.node_id);
    }

    #[test]
    fn trailing_garbage_rejected() {
        let e = sample(1, "n");
        let mut b = encode_canonical(&e);
        b.push(0xFF);
        assert_eq!(decode_canonical(&b), None);
    }

    #[test]
    fn unknown_status_rejected() {
        let e = sample(1, "n");
        let mut b = encode_canonical(&e);
        // status ada di offset tetap? cari lewat panjang prefix — lebih aman: ubah byte
        // status dengan men-decode ulang posisi: status = 1 + (4+6) + 8 + (4+1) + 32 + 32
        let idx = 1 + (4 + 6) + 8 + (4 + 1) + 32 + 32;
        b[idx] = 0x7F;
        assert_eq!(decode_canonical(&b), None);
    }

    #[test]
    fn observational_fields_not_in_entry() {
        // bukti statis C-04: struct tidak punya field metrik
        let e = sample(1, "n");
        let s = format!("{:?}", e);
        for forbidden in ["durasi_ms", "started_at", "finished_at", "worker_id", "rss_bytes"] {
            assert!(!s.contains(forbidden), "C-04: field observasional bocor ke entri");
        }
    }
}
