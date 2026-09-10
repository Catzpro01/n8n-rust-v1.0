//! Konstruksi chain (spec §3) + checkpoint (§5).
//!
//! ```text
//! CTX     = "n8nrust/envelope/v1"
//! ckey    = blake3::derive_key(CTX, chain_key)          ; DUA argumen (KOREKSI-1)
//! h_0     = keyed_hash(ckey, 0x00 || 0x00 || entry_0)   ; genesis ber-flag
//! h_i     = keyed_hash(ckey, h_{i-1} || entry_i)
//! head    = h_{n-1}                                     ; BUKAN "Merkle root" (C-03)
//! checkpoint_k = h_{(k+1)*K - 1},  K = 1.000
//! ```
//!
//! Disiplin memori (spec §1 prinsip 4, §4, SEC-TRANSIENT): SATU `blake3::Hasher` dipakai
//! berurutan lewat `reset()`. Persistent state hanya `h_prev` (32 B) + `ckey` (32 B) +
//! checkpoint (32 B per K entri). `heads` TIDAK disimpan di sini — itu tugas storage (§7).

use crate::entry::{encode_canonical, EnvelopeEntry};

pub const CTX: &str = "n8nrust/envelope/v1";
pub const CHECKPOINT_INTERVAL: u64 = 1_000;

/// Flag genesis (spec §3): dua byte 0x00 di depan entri pertama, supaya rantai kosong
/// dan rantai berisi entri yang di-hash dari string kosong tidak pernah bertabrakan.
pub const GENESIS_FLAG: [u8; 2] = [0x00, 0x00];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    /// indeks checkpoint (0-based)
    pub k: u64,
    /// `seq` entri terakhir yang dicakup
    pub seq: u64,
    /// `h_{(k+1)*K - 1}`
    pub head: [u8; 32],
    /// jumlah entri saat checkpoint dibuat
    pub n: u64,
    /// kid kunci yang dipakai segmen ini (spec §6, A1.5a)
    pub chain_key_id: String,
}

pub struct Chain {
    ckey: [u8; 32],
    pub chain_key_id: String,
    /// SATU hasher, dipakai ulang (SEC-TRANSIENT).
    hasher: blake3::Hasher,
    h_prev: [u8; 32],
    genesis_done: bool,
    pub n: u64,
    pub checkpoints: Vec<Checkpoint>,
}

impl Chain {
    /// `chain_key` = rahasia 32 B dari secret-manager (spec §6; BUKAN env/file — A1.2).
    /// Konteks saja bukan rahasia: tanpa `chain_key`, "kunci" = fungsi dari string publik.
    pub fn new(chain_key: &[u8; 32], chain_key_id: &str) -> Chain {
        let ckey = blake3::derive_key(CTX, chain_key);
        let hasher = blake3::Hasher::new_keyed(&ckey);
        Chain {
            ckey,
            chain_key_id: chain_key_id.to_string(),
            hasher,
            h_prev: [0u8; 32],
            genesis_done: false,
            n: 0,
            checkpoints: Vec::new(),
        }
    }

    /// Kunci turunan (diekspos untuk uji KAT/domain-separation, bukan untuk produksi).
    pub fn derived_key(&self) -> &[u8; 32] {
        &self.ckey
    }

    fn step(&mut self, prefix: &[u8], entry_bytes: &[u8]) -> [u8; 32] {
        self.hasher.reset();
        self.hasher.update(prefix);
        self.hasher.update(entry_bytes);
        *self.hasher.finalize().as_bytes()
    }

    /// Tambahkan satu entri; kembalikan `h_i`.
    pub fn append(&mut self, e: &EnvelopeEntry) -> [u8; 32] {
        let bytes = encode_canonical(e);
        let h = if !self.genesis_done {
            self.genesis_done = true;
            self.step(&GENESIS_FLAG, &bytes)
        } else {
            let prev = self.h_prev;
            self.step(&prev, &bytes)
        };
        self.h_prev = h;
        self.n += 1;
        if self.n.is_multiple_of(CHECKPOINT_INTERVAL) {
            self.checkpoints.push(Checkpoint {
                k: self.n / CHECKPOINT_INTERVAL - 1,
                seq: e.seq,
                head: h,
                n: self.n,
                chain_key_id: self.chain_key_id.clone(),
            });
        }
        h
    }

    /// `head = h_{n-1}` (spec §3). Untuk rantai kosong: nol — dan verifier harus menolak
    /// mengklaim apa pun atas rantai kosong.
    pub fn head(&self) -> [u8; 32] {
        self.h_prev
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
}

/// Hitung ulang seluruh rantai dari entri (dipakai verifier offline, spec §5.3).
/// Mengembalikan `(heads, checkpoints)`. Ini O(n) memori di sisi verifier — sadar biaya,
/// dan hanya dipakai alat audit, bukan jalur eksekusi.
pub fn recompute(
    entries: &[EnvelopeEntry],
    chain_key: &[u8; 32],
    chain_key_id: &str,
) -> (Vec<[u8; 32]>, Vec<Checkpoint>) {
    let mut c = Chain::new(chain_key, chain_key_id);
    let mut heads = Vec::with_capacity(entries.len());
    for e in entries {
        heads.push(c.append(e));
    }
    (heads, c.checkpoints)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{Status, SCHEMA_VERSION_V1};

    fn sample(seq: u64, node: &str) -> EnvelopeEntry {
        EnvelopeEntry {
            schema_version: SCHEMA_VERSION_V1,
            exec_id: b"exec-1".to_vec(),
            seq,
            node_id: node.as_bytes().to_vec(),
            input_digest: [seq as u8; 32],
            output_digest: [0x22; 32],
            status: Status::Succeeded,
            engine_version: b"0.1.0".to_vec(),
            config_digest: [0x33; 32],
            class_flags: 0,
        }
    }

    #[test]
    fn genesis_flag_matters() {
        let key = [7u8; 32];
        let mut c = Chain::new(&key, "kid-1");
        let h = c.append(&sample(0, "n0"));
        // tanpa flag genesis, hash akan sama dengan keyed_hash atas entri polos
        let bytes = encode_canonical(&sample(0, "n0"));
        let ckey = blake3::derive_key(CTX, &key);
        let no_flag = blake3::keyed_hash(&ckey, &bytes);
        assert_ne!(h, *no_flag.as_bytes(), "genesis flag tidak dipakai");
        let with_flag = {
            let mut v = GENESIS_FLAG.to_vec();
            v.extend_from_slice(&bytes);
            blake3::keyed_hash(&ckey, &v)
        };
        assert_eq!(h, *with_flag.as_bytes());
    }

    #[test]
    fn single_hasher_reuse_is_deterministic() {
        let key = [9u8; 32];
        let entries: Vec<_> = (0..5).map(|i| sample(i, &format!("node{i}"))).collect();
        let (h1, _) = recompute(&entries, &key, "kid-1");
        let (h2, _) = recompute(&entries, &key, "kid-1");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 5);
        // G-C4: replay deterministik -> head identik
        assert_eq!(h1[4], h2[4]);
    }

    #[test]
    fn different_key_different_head() {
        let entries: Vec<_> = (0..3).map(|i| sample(i, "n")).collect();
        let (h1, _) = recompute(&entries, &[1u8; 32], "kid-1");
        let (h2, _) = recompute(&entries, &[2u8; 32], "kid-1");
        assert_ne!(h1[2], h2[2], "keyed chain tidak bergantung kunci");
    }

    #[test]
    fn checkpoint_every_k() {
        let key = [3u8; 32];
        let mut c = Chain::new(&key, "kid-1");
        for i in 0..(CHECKPOINT_INTERVAL * 2) {
            c.append(&sample(i, "n"));
        }
        assert_eq!(c.checkpoints.len(), 2);
        assert_eq!(c.checkpoints[0].n, CHECKPOINT_INTERVAL);
        assert_eq!(c.checkpoints[0].k, 0);
        assert_eq!(c.checkpoints[1].k, 1);
        assert_eq!(c.checkpoints[1].head, c.head());
    }

    #[test]
    fn transient_budget_one_hasher() {
        // SEC-TRANSIENT: 1 hasher = 1.920 B <= 4.096 B (47% budget)
        assert!(crate::hasher_size_bytes() <= crate::TRANSIENT_BUDGET_BYTES);
        assert!(crate::sha256_size_bytes() <= crate::TRANSIENT_BUDGET_BYTES);
    }
}
