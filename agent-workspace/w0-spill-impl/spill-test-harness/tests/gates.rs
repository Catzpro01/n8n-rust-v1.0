//! W0-SPILL-TEST — independent black-box gates for FileSpillStore.
//!
//! INDEPENDENCE RULES (separation of concerns per fern #786):
//! 1. Every assertion traces to SPEC text (cite in gate header), never to any
//!    implementation's source. The harness sees the impl ONLY via the kernel
//!    `SpillStore` trait + spec'd `KernelError` variants + spec'd handle fields.
//! 2. File-level surgery (corruption/magic/truncation) assumes ONLY the
//!    spec'd layout skeleton: magic first (#256: 4B magic + 1B version),
//!    records after header, whole-file SHA-256. No magic VALUE, no header
//!    LENGTH, no codec choice is pinned — those are implementer decisions.
//! 3. Inherent (non-trait) APIs (`for_execution`, `gc_execution`) are
//!    PROVISIONAL call forms: the GATE is spec'd (SINTESIS §6.1.3, A-21),
//!    the spelling adapts to agent2's landed API before final sign-off.
//! 4. Mutation runner (`mutants/`) proves every corruption gate goes RED on
//!    planted bugs — green here is never vacuous.

use impl_under_test::FileSpillStore;
use kernel::error::KernelError;
use kernel::item::{Item, SpillStore, SpilledList};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    use std::sync::atomic::AtomicBool;
    use std::task::{Context, Poll, Wake};

    struct Spin(AtomicBool);
    impl Wake for Spin {
        fn wake(self: std::sync::Arc<Self>) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    let waker = std::sync::Arc::new(Spin(AtomicBool::new(false))).into();
    let mut cx = Context::from_waker(&waker);
    let mut fut = Box::pin(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn fresh_dir() -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!(
        "spill-gate-{}-{}-{}",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn item(i: u64) -> Item {
    Item::new(json!({"id": i, "name": format!("row-{i}"), "pad": "x".repeat(64)}))
}

fn items(n: u64) -> Vec<Item> {
    (0..n).map(item).collect()
}

fn spill_file(dir: &Path, h: &SpilledList) -> PathBuf {
    dir.join(h.path.as_str())
}

/// T-SPILL-01 · SPEC: kernel `SpillStore` trait contract (A-03: spilling
/// invisible to node semantics). Write→read_at/range/all round-trips exactly.
#[test]
fn t01_roundtrip_identity() {
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    let items = items(50);
    let h = block_on(store.write(&items)).unwrap();
    assert_eq!(h.len, 50);
    assert!(h.total_bytes > 0);
    assert_eq!(block_on(store.read_at(&h, 0)).unwrap().json, items[0].json);
    assert_eq!(block_on(store.read_at(&h, 49)).unwrap().json, items[49].json);
    let r = block_on(store.read_range(&h, 10, 5)).unwrap();
    assert_eq!(r.len(), 5);
    assert_eq!(r[0].json, items[10].json);
    assert_eq!(block_on(store.read_all(&h)).unwrap().len(), 50);
    block_on(store.delete(&h)).unwrap();
    assert!(!spill_file(&dir, &h).exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-02 · SPEC: PATCH acceptance (SpilledList has both hashes) +
/// CHECKSUM-AGREE §8 Boundary (Layer0 ≠ CASD, different algorithms).
#[test]
fn t02_dual_hash_present_and_distinct() {
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    let h = block_on(store.write(&items(5))).unwrap();
    let (Some(sha), Some(b3)) = (h.layer0_checksum, h.casd_hash) else {
        panic!("PATCH acceptance: both hashes must be computed");
    };
    assert_ne!(sha, b3, "CHECKSUM-AGREE §8: boundary clarity");
    assert_eq!(block_on(store.verify_integrity(&h)).unwrap(), true);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-03 · SPEC: CHECKSUM-AGREE §8 Integrity (corrupt 1 byte → SHA-256
/// mismatch) + SINTESIS §6.1.4. Flips ONE byte mid-file; position-agnostic.
#[test]
fn t03_one_byte_corruption_detected() {
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    let h = block_on(store.write(&items(10))).unwrap();
    assert!(block_on(store.verify_integrity(&h)).unwrap());

    let p = spill_file(&dir, &h);
    let len = std::fs::metadata(&p).unwrap().len();
    let mut f = OpenOptions::new().read(true).write(true).open(&p).unwrap();
    f.seek(SeekFrom::Start(len / 2)).unwrap();
    let mut b = [0u8; 1];
    use std::io::Read;
    f.read_exact(&mut b).unwrap();
    b[0] ^= 0x01;
    f.seek(SeekFrom::Start(len / 2)).unwrap();
    f.write_all(&b).unwrap();
    drop(f);

    match block_on(store.verify_integrity(&h)) {
        Err(KernelError::ChecksumMismatch { .. }) => {}
        other => panic!("want ChecksumMismatch, got {other:?}"),
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-04 · SPEC: G-C7 bad-magic→InvalidMagic (fail-closed). Overwrites
/// the FIRST 4 bytes (#256: magic is first on disk).
#[test]
fn t04_bad_magic_fail_closed() {
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    let h = block_on(store.write(&items(5))).unwrap();

    let p = spill_file(&dir, &h);
    let mut f = OpenOptions::new().write(true).open(&p).unwrap();
    f.write_all(b"NOPE").unwrap();
    drop(f);

    match block_on(store.read_at(&h, 0)) {
        Err(KernelError::InvalidMagic { .. }) => {}
        other => panic!("want InvalidMagic, got {other:?}"),
    }
    match block_on(store.verify_integrity(&h)) {
        Err(KernelError::InvalidMagic { .. }) => {}
        other => panic!("want InvalidMagic on verify, got {other:?}"),
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-05 · SPEC: G-C7 truncation→OffsetOutOfRange (fail-closed).
/// Accepts InvalidMagic too: a 10-byte file may cut inside the header —
/// EITHER corruption signal is correct, silence is not.
#[test]
fn t05_truncation_detected() {
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    let h = block_on(store.write(&items(20))).unwrap();

    let p = spill_file(&dir, &h);
    let f = OpenOptions::new().write(true).open(&p).unwrap();
    f.set_len(10).unwrap();
    drop(f);

    let corrupt = |r: Result<Item, KernelError>| {
        matches!(
            r,
            Err(KernelError::OffsetOutOfRange { .. }) | Err(KernelError::InvalidMagic { .. })
        )
    };
    assert!(
        corrupt(block_on(store.read_at(&h, 19))),
        "truncated read_at must fail closed"
    );
    assert!(
        block_on(store.read_range(&h, 0, 20)).is_err(),
        "truncated read_range must fail closed"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-06 · SPEC: CHECKSUM-AGREE §8 UMask + SEC-07 (agent5): umask 000 →
/// spill files mode 0600 (constructor enforcement, not luck).
#[test]
fn t06_umask000_gives_0600() {
    let prev = unsafe { libc::umask(0o000) };
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    let h = block_on(store.write(&items(2))).unwrap();
    unsafe {
        libc::umask(prev);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(spill_file(&dir, &h))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "SEC-07: spill file must be 0600 under umask 000");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-07 · SPEC: PATCH backward-compat (legacy spills: hashes None;
/// verify returns Err, never a false Ok(true)).
#[test]
fn t07_legacy_handle_verify_fails_explicit() {
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    let h = block_on(store.write(&items(2))).unwrap();
    let legacy = SpilledList {
        path: h.path.clone(),
        len: h.len,
        total_bytes: h.total_bytes,
        codec: h.codec,
        layer0_checksum: None,
        casd_hash: None,
    };
    match block_on(store.verify_integrity(&legacy)) {
        Err(KernelError::Invalid { message }) => {
            assert!(message.contains("legacy"), "message: {message}");
        }
        Ok(v) => panic!("legacy verify must Err, got Ok({v})"),
        other => panic!("want Invalid(legacy), got {other:?}"),
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-08 · SPEC: SINTESIS §6.1.3 (absorb gc_execution) + A-21 (never GC
/// a live execution; GC removes OWN files only). CALL FORM PROVISIONAL.
#[test]
fn t08_gc_execution_removes_only_own_files() {
    let root = fresh_dir();
    let store = FileSpillStore::for_execution(&root, "exec-9").unwrap();
    let h1 = block_on(store.write(&items(3))).unwrap();
    let _h2 = block_on(store.write(&items(4))).unwrap();
    std::fs::write(root.join("exec-9").join("keep.txt"), b"no").unwrap();

    let n = store.gc_execution().unwrap();
    assert_eq!(n, 2);
    assert!(!spill_file(&root.join("exec-9"), &h1).exists());
    assert!(root.join("exec-9").join("keep.txt").exists());
    assert!(block_on(store.read_at(&h1, 0)).is_err());
    assert!(FileSpillStore::for_execution(&root, "../evil").is_err());
    std::fs::remove_dir_all(&root).unwrap();
}

/// T-SPILL-09 · SPEC: kernel trait contract (audit A-21): dropping a writer
/// without finish MUST release its partial file (no leak).
#[test]
fn t09_dropped_writer_leaves_no_file() {
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    {
        let mut w = block_on(store.writer()).unwrap();
        block_on(w.push(&items(5))).unwrap();
        assert_eq!(w.pushed(), 5);
    }
    let leftovers: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
    assert!(leftovers.is_empty(), "partial file leaked: {leftovers:?}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-10 · SPEC: kernel trait (`read_at` must be O(1)-ish indexed).
/// Correctness at 5k scale + full verify green.
#[test]
fn t10_indexed_read_scales() {
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    let h = block_on(store.write(&items(5000))).unwrap();
    assert_eq!(block_on(store.read_at(&h, 4999)).unwrap().json["id"], json!(4999));
    assert!(block_on(store.verify_integrity(&h)).unwrap());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-11 · SPEC: PATCH test_casd_dedup (same content → same casd_hash;
/// paths differ — dedup happens at CASD layer, not here).
#[test]
fn t11_same_content_same_casd_hash() {
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    let h1 = block_on(store.write(&items(7))).unwrap();
    let h2 = block_on(store.write(&items(7))).unwrap();
    assert_eq!(h1.casd_hash, h2.casd_hash);
    assert_eq!(h1.layer0_checksum, h2.layer0_checksum);
    assert_ne!(h1.path.as_str(), h2.path.as_str());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-12 · SPEC: no-regression — API-level OOB keeps `IndexOutOfBounds`
/// (file-level truncation is T-05's `OffsetOutOfRange`; the two must not mix).
#[test]
fn t12_api_oob_stays_index_out_of_bounds() {
    let dir = fresh_dir();
    let store = FileSpillStore::new(&dir).unwrap();
    let h = block_on(store.write(&items(3))).unwrap();
    match block_on(store.read_at(&h, 3)) {
        Err(KernelError::IndexOutOfBounds { index: 3, len: 3 }) => {}
        other => panic!("want IndexOutOfBounds, got {other:?}"),
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
