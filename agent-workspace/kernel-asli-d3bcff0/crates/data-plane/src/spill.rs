//! `FileSpillStore` — disk-backed `SpillStore` (canonical data-plane).
//!
//! PORT PROVENANCE (Ruling 10, #1073 — executed by agent1 per #1080):
//! source = `rust-engine/crates/storage/src/spill.rs` @ sha256 `5bf97497…`
//! (author: agent2), adapted to the canonical kernel API:
//!
//! * `tokio::fs` → `std::fs` inside `async fn` (Ruling 7/10.2; the trait is
//!   `async_trait`, signatures stay `async`). Precedent: `spill_bench.rs`.
//! * `tracing` → REMOVED (Ruling 10.3; kernel `Logger` D63/D93 is the only
//!   logging path — C-02 "no second path" applies to logging too).
//! * `uuid::Uuid::new_v4` filenames → `ContentId(u64)`-derived names
//!   (Ruling 10.4): `content_id = u64::from_le_bytes(sha256(payload)[..8])`,
//!   file = `{content_id}.spill`. Same content → same name: dedup by
//!   construction (see FS-11). Tmp files stay unique via pid+counter.
//! * Canonical `SpillPath` is an OPAQUE STRING (`id.rs:97`, deliberately not
//!   a `PathBuf`): the store keeps `{content_id}.spill` (relative file name)
//!   in the handle and resolves `base_dir.join(name)`. Empty-write sentinel:
//!   `SpillPath("")` + `len == 0` (reads guard on `len` first).
//! * `libc::umask` juggling → REMOVED (no new lib dep): files are created
//!   with mode `0600` + enforced via `set_permissions`; store-created dirs
//!   are enforced `0700`. Umask-independent by construction (FS-06).
//!
//! PORT-NOTEs (flagged, not silently changed):
//! * NOTE-1: `push()` still accumulates the full payload in RAM (as the
//!   source did); true streaming write-through is a follow-up (Governor D72).
//! * NOTE-2 (Ruling 21.4, applied): dedup-by-construction WITH a FILE-BACKED
//!   refcount sidecar (`{id}.spill.ref`, u64 LE, 0600). `write`/`finish`
//!   increment; `delete` decrements and removes the payload only at zero.
//!   Counts survive crashes on disk (no in-memory registry to lose); the last
//!   deleter always wins. FS-08 pins the two-handle lifecycle explicitly.
//! * NOTE-6: refcount read-modify-write is serialized by a process-wide mutex.
//!   The engine is one binary (`main.rs`); two processes sharing one spill
//!   dir is out of scope and documented as such.
//! * NOTE-3 (hardening, applied): `deserialize_item` uses CHECKED slicing
//!   (the source could panic on corrupt offsets — header is JSON, only the
//!   *payload* is checksummed), and `header_len`/`total_bytes` are capped at
//!   the physical file length before any allocation.
//! * NOTE-4 (parity fix, applied): writer-produced files are enforced `0600`
//!   like `write()`-produced ones (the source only enforced it on one path).
//! * NOTE-5 (behavior fix, applied): `read_range` with `len == 0` returns
//!   `Ok(vec![])` — the empty-write sentinel (`SpillPath("")`, no file) reads
//!   as empty instead of leaking a storage "no such file" error. `delete` on
//!   it is a silent `Ok` and `disk_footprint` is 0 — the empty string must
//!   NEVER reach `file_path` (`base_dir.join("")` is the base dir itself).
//! * NOTE-7 / FS-15 (Ruling 15, Jebakan 2): `SpillPath::new` does zero
//!   validation, so `file_path` allowlists `{digits}.spill` and rejects
//!   everything else (`..`, `/`, absolute paths, NUL bytes) with `Invalid`.
//!   Handles are untrusted input at every API boundary.

use kernel::error::KernelError;
use kernel::id::{ContentId, SpillPath};
use kernel::item::{Item, SpillCodec, SpilledList, SpillStore, SpillWriter};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

fn io_to_kernel(e: std::io::Error) -> KernelError {
    KernelError::SpillIo {
        message: e.to_string(),
        kind: e.kind(),
    }
}

fn invalid(msg: impl Into<String>) -> KernelError {
    KernelError::Invalid {
        message: msg.into(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SpillFileHeader {
    magic: [u8; 6],
    version: u32,
    codec: SpillCodec,
    item_count: u32,
    total_bytes: u64,
    checksum: [u8; 32],
    offsets: Vec<u64>,
}

impl SpillFileHeader {
    const MAGIC: [u8; 6] = *b"SPILL\x00";
    const VERSION: u32 = 1;

    fn new(
        codec: SpillCodec,
        item_count: u32,
        total_bytes: u64,
        checksum: [u8; 32],
        offsets: Vec<u64>,
    ) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            codec,
            item_count,
            total_bytes,
            checksum,
            offsets,
        }
    }

    fn validate(&self) -> Result<(), KernelError> {
        if self.magic != Self::MAGIC {
            return Err(invalid("invalid magic"));
        }
        if self.version != Self::VERSION {
            return Err(invalid(format!("unsupported version: {}", self.version)));
        }
        Ok(())
    }
}

/// Tmp-file uniqueness within (and across) processes. Tmp names are never
/// audited — only final `{content_id}.spill` names are (Ruling 10.4b).
static NEXT_TMP: AtomicU64 = AtomicU64::new(0);

/// Process-wide lock serializing refcount read-modify-write (NOTE-6).
static REFCOUNT_LOCK: Mutex<()> = Mutex::new(());

/// Sidecar path for a payload file: `{id}.spill` -> `{id}.spill.ref`.
fn ref_path_for(spill_path: &Path) -> PathBuf {
    spill_path.with_extension("spill.ref")
}

/// Atomically (tmp + rename, 0600) persist a refcount value.
fn write_refcount(ref_path: &Path, count: u64) -> Result<(), KernelError> {
    let parent = ref_path
        .parent()
        .ok_or_else(|| invalid(format!("refcount path has no parent: {ref_path:?}")))?;
    let tmp = parent.join(format!(
        "ref-{}-{}.tmp",
        std::process::id(),
        NEXT_TMP.fetch_add(1, Ordering::SeqCst)
    ));
    fs::write(&tmp, count.to_le_bytes()).map_err(io_to_kernel)?;
    enforce_file_mode(&tmp)?;
    fs::rename(&tmp, ref_path).map_err(io_to_kernel)?;
    Ok(())
}

/// Read the live-handle count; a missing/corrupt sidecar reads as 0
/// (fail-closed toward "last handle": the file gets removed).
fn read_refcount(ref_path: &Path) -> u64 {
    fs::read(ref_path)
        .ok()
        .filter(|b| b.len() >= 8)
        .map(|b| u64::from_le_bytes(b[..8].try_into().expect("len>=8 checked")))
        .unwrap_or(0)
}

fn lock_refcounts() -> std::sync::MutexGuard<'static, ()> {
    REFCOUNT_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn tmp_path_for(base_dir: &Path) -> PathBuf {
    let n = NEXT_TMP.fetch_add(1, Ordering::SeqCst);
    base_dir.join(format!("tmp-{}-{}.spill.tmp", std::process::id(), n))
}

#[cfg(unix)]
fn enforce_file_mode(p: &Path) -> Result<(), KernelError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(p, fs::Permissions::from_mode(0o600)).map_err(io_to_kernel)
}

#[cfg(not(unix))]
fn enforce_file_mode(_p: &Path) -> Result<(), KernelError> {
    Ok(())
}

#[derive(Debug, Clone)]
pub struct FileSpillStore {
    base_dir: PathBuf,
}

impl FileSpillStore {
    pub async fn new(base_dir: impl AsRef<Path>) -> Result<Self, KernelError> {
        let base_dir = base_dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).map_err(io_to_kernel)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&base_dir, fs::Permissions::from_mode(0o700))
                    .map_err(io_to_kernel)?;
            }
        }
        Ok(Self { base_dir })
    }

    /// Resolve a handle to its backing file. The opaque string inside
    /// `SpillPath` is the `{content_id}.spill` file name (Ruling 10.4).
    ///
    /// FS-15: the handle string is UNTRUSTED (`SpillPath::new` validates
    /// nothing) — allowlist `{digits}.spill`, reject everything else.
    fn file_path(&self, id: &SpillPath) -> Result<PathBuf, KernelError> {
        let name = id.as_str();
        let stem_ok = name
            .strip_suffix(".spill")
            .is_some_and(|stem| !stem.is_empty() && stem.bytes().all(|b| b.is_ascii_digit()));
        if !stem_ok {
            return Err(invalid(format!("rejected spill file name: {name:?}")));
        }
        Ok(self.base_dir.join(name))
    }

    fn serialize_items(items: &[Item]) -> Result<(Vec<u8>, Vec<u64>), KernelError> {
        let mut payload = Vec::new();
        let mut offsets = Vec::with_capacity(items.len());
        for item in items {
            offsets.push(payload.len() as u64);
            let item_bytes = serde_json::to_vec(item).map_err(|e| KernelError::Codec {
                codec: "json",
                reason: e.to_string(),
            })?;
            payload.extend_from_slice(&(item_bytes.len() as u32).to_le_bytes());
            payload.extend_from_slice(&item_bytes);
        }
        Ok((payload, offsets))
    }

    /// Content-derived identity (Ruling 10.4): first 8 bytes of the SAME
    /// SHA-256 that becomes `header.checksum` — one hash, two uses.
    /// 64-bit truncation: birthday aliasing is accepted W0 risk (documented,
    /// never silent: identical bytes are the only alias case by construction
    /// unless a 64-bit collision occurs).
    fn content_id_for(checksum: &[u8; 32]) -> ContentId {
        let mut b = [0u8; 8];
        b.copy_from_slice(&checksum[..8]);
        ContentId::new(u64::from_le_bytes(b))
    }

    /// NOTE-3: checked slicing — corrupt offsets must Err, never panic.
    /// 50c written-reason (R52b): target parse adalah struct tetap (Item), jadi
    /// kedalaman rekursi dibatasi oleh bentuk TIPE, bukan oleh input — berbeda dari
    /// situs yang menargetkan serde_json::Value.
    /// Dimensi panjang dijaga di sini (checked_add + payload.get bounds check).
    /// Provenance: berkas spill tulisan serialize_items sendiri; integritas SHA-256
    /// diverifikasi load_and_verify. Bukan trust.
    fn deserialize_item(payload: &[u8], offset: u64) -> Result<Item, KernelError> {
        let offset = offset as usize;
        let len_bytes: [u8; 4] = payload
            .get(offset..offset.checked_add(4).ok_or_else(|| invalid("offset overflow"))?)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| invalid("record length out of bounds"))?;
        let item_len = u32::from_le_bytes(len_bytes) as usize;
        let end = offset
            .checked_add(4)
            .and_then(|s| s.checked_add(item_len))
            .ok_or_else(|| invalid("record end overflow"))?;
        let item_bytes = payload
            .get(offset + 4..end)
            .ok_or_else(|| invalid("record bytes out of bounds"))?;
        serde_json::from_slice(item_bytes).map_err(|e| KernelError::Codec {
            codec: "json",
            reason: e.to_string(),
        })
    }

    /// Single implementation of the read+verify path (used by `read_at` and
    /// `read_range`): header parse → validate → payload read → SHA-256 check.
    /// Length fields are capped at the physical file length BEFORE any
    /// allocation (NOTE-3: fail-closed on truncation/corruption).
    /// 50c written-reason (R52b): target parse header adalah struct tetap
    /// (SpillFileHeader), jadi kedalaman rekursi dibatasi oleh bentuk TIPE, bukan
    /// oleh input. Dimensi panjang: header_len melebihi file_len ditolak SEBELUM
    /// header_bytes dialokasikan. Provenance + integritas: berkas spill tulisan
    /// serialize_items sendiri, checksum SHA-256 diverifikasi di bawah. Bukan trust.
    fn load_and_verify(&self, handle: &SpilledList) -> Result<(SpillFileHeader, Vec<u8>), KernelError> {
        let path = self.file_path(&handle.path)?;
        let file_len = fs::metadata(&path).map(|m| m.len()).map_err(io_to_kernel)?;
        let mut file = fs::File::open(&path).map_err(io_to_kernel)?;
        let mut header_len_bytes = [0u8; 4];
        file.read_exact(&mut header_len_bytes).map_err(io_to_kernel)?;
        let header_len = u32::from_le_bytes(header_len_bytes) as u64;
        if header_len > file_len {
            return Err(invalid("header length exceeds file length"));
        }
        let mut header_bytes = vec![0u8; header_len as usize];
        file.read_exact(&mut header_bytes).map_err(io_to_kernel)?;
        let header: SpillFileHeader =
            serde_json::from_slice(&header_bytes).map_err(|e| KernelError::Codec {
                codec: "json",
                reason: e.to_string(),
            })?;
        header.validate()?;
        if header.total_bytes > file_len {
            return Err(invalid("declared payload exceeds file length"));
        }
        let mut payload = vec![0u8; header.total_bytes as usize];
        file.read_exact(&mut payload).map_err(io_to_kernel)?;
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let computed: [u8; 32] = hasher.finalize().into();
        if computed != header.checksum {
            return Err(invalid("checksum mismatch — file corrupted"));
        }
        Ok((header, payload))
    }

    /// Write `header + payload` atomically (tmp + `0600` + rename).
    /// WARNING (agent3 #1254 4.1 / D6): ALL spill writes MUST stay atomic
    /// (tmp + rename). A non-atomic path invalidates D6 (ETIMEDOUT->Transient
    /// safety) — a torn write would be misclassified as retryable.
    fn commit(&self, header: &SpillFileHeader, payload: &[u8]) -> Result<SpillPath, KernelError> {
        let header_bytes =
            serde_json::to_vec(header).map_err(|e| KernelError::Codec {
                codec: "json",
                reason: e.to_string(),
            })?;
        let content_id = Self::content_id_for(&header.checksum);
        let spill_path = SpillPath::new(format!("{}.spill", content_id.get()));
        let final_path = self.file_path(&spill_path)?;
        let tmp = tmp_path_for(&self.base_dir);
        {
            let mut file = fs::File::create(&tmp).map_err(io_to_kernel)?;
            file.write_all(&(header_bytes.len() as u32).to_le_bytes())
                .map_err(io_to_kernel)?;
            file.write_all(&header_bytes).map_err(io_to_kernel)?;
            file.write_all(payload).map_err(io_to_kernel)?;
            file.flush().map_err(io_to_kernel)?;
        }
        enforce_file_mode(&tmp)?;
        fs::rename(&tmp, &final_path).map_err(io_to_kernel)?;
        {
            let _g = lock_refcounts();
            let ref_path = ref_path_for(&final_path);
            write_refcount(&ref_path, read_refcount(&ref_path).saturating_add(1))?;
        }
        Ok(spill_path)
    }
}

#[async_trait::async_trait]
impl SpillStore for FileSpillStore {
    async fn write(&self, items: &[Item]) -> Result<SpilledList, KernelError> {
        if items.is_empty() {
            // Empty sentinel: no file, `len == 0` guards every read path.
            return Ok(SpilledList {
                path: SpillPath::new(""),
                len: 0,
                total_bytes: 0,
                codec: SpillCodec::Json,
            });
        }
        let (payload, offsets) = Self::serialize_items(items)?;
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let checksum: [u8; 32] = hasher.finalize().into();
        let header = SpillFileHeader::new(
            SpillCodec::Json,
            items.len() as u32,
            payload.len() as u64,
            checksum,
            offsets,
        );
        let spill_path = self.commit(&header, &payload)?;
        Ok(SpilledList {
            path: spill_path,
            len: items.len() as u32,
            total_bytes: payload.len() as u64,
            codec: SpillCodec::Json,
        })
    }

    async fn read_at(&self, handle: &SpilledList, index: usize) -> Result<Item, KernelError> {
        if index >= handle.len as usize {
            return Err(KernelError::IndexOutOfBounds {
                index,
                len: handle.len as usize,
            });
        }
        let (header, payload) = self.load_and_verify(handle)?;
        let offset = *header
            .offsets
            .get(index)
            .ok_or_else(|| invalid("offset index out of bounds"))?;
        Self::deserialize_item(&payload, offset)
    }

    async fn read_range(
        &self,
        handle: &SpilledList,
        start: usize,
        len: usize,
    ) -> Result<Vec<Item>, KernelError> {
        if len == 0 {
            return Ok(Vec::new());
        }
        if start.saturating_add(len) > handle.len as usize {
            return Err(KernelError::IndexOutOfBounds {
                index: start.saturating_add(len).saturating_sub(1),
                len: handle.len as usize,
            });
        }
        let (header, payload) = self.load_and_verify(handle)?;
        let mut items = Vec::with_capacity(len);
        for i in start..start + len {
            let offset = *header
                .offsets
                .get(i)
                .ok_or_else(|| invalid("offset index out of bounds"))?;
            items.push(Self::deserialize_item(&payload, offset)?);
        }
        Ok(items)
    }

    async fn read_all(&self, handle: &SpilledList) -> Result<Vec<Item>, KernelError> {
        self.read_range(handle, 0, handle.len as usize).await
    }

    async fn delete(&self, handle: &SpilledList) -> Result<(), KernelError> {
        // NOTE-5: the empty sentinel owns no file; never resolve it.
        if handle.path.as_str().is_empty() {
            return Ok(());
        }
        // FS-15: a malicious handle is REJECTED here, not executed.
        let path = self.file_path(&handle.path)?;
        let _g = lock_refcounts();
        let ref_path = ref_path_for(&path);
        let count = read_refcount(&ref_path);
        if count > 1 {
            // Other live handles still reference this payload: keep the file.
            write_refcount(&ref_path, count - 1)?;
            return Ok(());
        }
        // Last handle (or no sidecar at all): remove payload + sidecar.
        let _ = fs::remove_file(&ref_path);
        if path.exists() {
            fs::remove_file(&path).map_err(io_to_kernel)?;
        }
        Ok(())
    }

    fn disk_footprint(&self, handle: &SpilledList) -> u64 {
        // NOTE-5: the empty sentinel owns no file (and `join("")` would
        // resolve to the base dir itself — never let it reach `file_path`).
        if handle.path.as_str().is_empty() {
            return 0;
        }
        // FS-15: untrusted names fail closed to 0 instead of being stated.
        // Payload file only; the 8-byte refcount sidecar is metadata
        // overhead and deliberately excluded (pinned by FS-08).
        self.file_path(&handle.path)
            .ok()
            .and_then(|path| fs::metadata(&path).map(|m| m.len()).ok())
            .unwrap_or(0)
    }

    async fn writer(&self) -> Result<Box<dyn SpillWriter>, KernelError> {
        Ok(Box::new(FileSpillWriter::new(self.base_dir.clone())?))
    }
}

struct FileSpillWriter {
    base_dir: PathBuf,
    tmp_path: PathBuf,
    /// `None` after `finish` takes it: the tmp is renamed, Drop disarms.
    file: Option<fs::File>,
    payload: Vec<u8>,
    offsets: Vec<u64>,
    pushed_count: u32,
}

impl FileSpillWriter {
    fn new(base_dir: PathBuf) -> Result<Self, KernelError> {
        let tmp_path = tmp_path_for(&base_dir);
        let file = fs::File::create(&tmp_path).map_err(io_to_kernel)?;
        Ok(Self {
            base_dir,
            tmp_path,
            file: Some(file),
            payload: Vec::new(),
            offsets: Vec::new(),
            pushed_count: 0,
        })
    }
}

#[async_trait::async_trait]
impl SpillWriter for FileSpillWriter {
    async fn push(&mut self, items: &[Item]) -> Result<(), KernelError> {
        // NOTE-1: accumulates in RAM (source behavior preserved); streaming
        // write-through is follow-up work (Governor D72).
        for item in items {
            self.offsets.push(self.payload.len() as u64);
            let item_bytes =
                serde_json::to_vec(item).map_err(|e| KernelError::Codec {
                    codec: "json",
                    reason: e.to_string(),
                })?;
            self.payload
                .extend_from_slice(&(item_bytes.len() as u32).to_le_bytes());
            self.payload.extend_from_slice(&item_bytes);
            self.pushed_count += 1;
        }
        Ok(())
    }

    async fn finish(mut self: Box<Self>) -> Result<SpilledList, KernelError> {
        if self.pushed_count == 0 {
            self.file.take();
            let _ = fs::remove_file(&self.tmp_path);
            self.tmp_path = tmp_path_for(&self.base_dir); // disarm Drop
            return Ok(SpilledList {
                path: SpillPath::new(""),
                len: 0,
                total_bytes: 0,
                codec: SpillCodec::Json,
            });
        }
        let mut hasher = Sha256::new();
        hasher.update(&self.payload);
        let checksum: [u8; 32] = hasher.finalize().into();
        let header = SpillFileHeader::new(
            SpillCodec::Json,
            self.pushed_count,
            self.payload.len() as u64,
            checksum,
            self.offsets.clone(),
        );
        let header_bytes =
            serde_json::to_vec(&header).map_err(|e| KernelError::Codec {
                codec: "json",
                reason: e.to_string(),
            })?;
        {
            let file = self.file.as_mut().expect("writer file taken before finish");
            file.write_all(&(header_bytes.len() as u32).to_le_bytes())
                .map_err(io_to_kernel)?;
            file.write_all(&header_bytes).map_err(io_to_kernel)?;
            file.write_all(&self.payload).map_err(io_to_kernel)?;
            file.flush().map_err(io_to_kernel)?;
        }
        self.file.take(); // close before chmod+rename
        // NOTE-4: enforce 0600 on the writer path too (source missed this).
        enforce_file_mode(&self.tmp_path)?;
        let content_id = FileSpillStore::content_id_for(&checksum);
        let spill_path = SpillPath::new(format!("{}.spill", content_id.get()));
        let final_path = self.base_dir.join(spill_path.as_str());
        fs::rename(&self.tmp_path, &final_path).map_err(io_to_kernel)?;
        {
            let _g = lock_refcounts();
            let ref_path = ref_path_for(&final_path);
            write_refcount(&ref_path, read_refcount(&ref_path).saturating_add(1))?;
        }
        // Disarm Drop: the tmp no longer exists after rename.
        self.tmp_path = tmp_path_for(&self.base_dir);
        Ok(SpilledList {
            path: spill_path,
            len: self.pushed_count,
            total_bytes: self.payload.len() as u64,
            codec: SpillCodec::Json,
        })
    }

    fn pushed(&self) -> u32 {
        self.pushed_count
    }
}

// Audit A-21: dropping a writer without `finish` MUST release its partial
// file. Best-effort, silent (no logging framework in this crate — Ruling 7).
impl Drop for FileSpillWriter {
    fn drop(&mut self) {
        if self.tmp_path.exists() {
            let _ = fs::remove_file(&self.tmp_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::{Seek, SeekFrom};

    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        use std::sync::Arc;
        use std::task::{Context, Poll, Wake};
        struct Spin;
        impl Wake for Spin {
            fn wake(self: Arc<Self>) {}
        }
        let waker = Arc::new(Spin).into();
        let mut cx = Context::from_waker(&waker);
        let mut fut = Box::pin(fut);
        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(v) => return v,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    fn unique_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "dataplane-unit-{}-{}-{}",
            std::process::id(),
            tag,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn make_test_item(id: u32) -> Item {
        Item {
            json: json!({"id": id}),
            binary: None,
            paired_item: None,
        }
    }

    #[test]
    fn test_write_and_read() {
        block_on(async {
            let dir = unique_dir("rw");
            let store = FileSpillStore::new(&dir).await.unwrap();
            let items = vec![make_test_item(1), make_test_item(2), make_test_item(3)];
            let handle = store.write(&items).await.unwrap();
            assert_eq!(handle.len, 3);
            let item = store.read_at(&handle, 1).await.unwrap();
            assert_eq!(item.json["id"], 2);
            store.delete(&handle).await.unwrap();
            fs::remove_dir_all(&dir).unwrap();
        });
    }

    #[test]
    fn file_path_allowlists_digits_dot_spill_only() {
        let dir = std::env::temp_dir().join(format!("spill-unit-{}", std::process::id()));
        let store = block_on(FileSpillStore::new(&dir)).unwrap();
        assert!(store.file_path(&SpillPath::new("7.spill")).is_ok());
        for bad in [
            "",
            "../evil.spill",
            "../../etc/passwd",
            "/tmp/x.spill",
            "7.spill\0",
            "abc.spill",
            ".spill",
            "7.spil",
            "7.SPILL",
            "sub/7.spill",
        ] {
            assert!(
                store.file_path(&SpillPath::new(bad)).is_err(),
                "must reject {bad:?}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_checksum_verification() {
        block_on(async {
            let dir = unique_dir("ck");
            let store = FileSpillStore::new(&dir).await.unwrap();
            let items = vec![make_test_item(1)];
            let handle = store.write(&items).await.unwrap();

            let path = dir.join(handle.path.as_str());
            let mut file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .unwrap();
            file.seek(SeekFrom::Start(200)).unwrap();
            let mut byte = [0u8; 1];
            file.read_exact(&mut byte).unwrap();
            byte[0] ^= 0xFF;
            file.seek(SeekFrom::Start(200)).unwrap();
            file.write_all(&byte).unwrap();
            drop(file);

            let result = store.read_at(&handle, 0).await;
            assert!(result.is_err(), "expected error from corrupted file");
            fs::remove_dir_all(&dir).unwrap();
        });
    }
}
