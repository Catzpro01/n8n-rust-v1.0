//! FS-01..FS-12 + FS-15 — canonical data-plane gates for `FileSpillStore`.
//!
//! PORT of `/home/agent1/spill-run/tests/gates.rs` (T-SPILL-01..12, 12/12 green
//! vs agent2's `storage::spill` @ `5bf97497`) onto the Ruling-10 canonical port
//! (`crates/data-plane/src/spill.rs`). Transformations, all mechanical except
//! FS-11 (flagged):
//! * `#[tokio::test]` → sync `#[test]` + local `block_on` (Ruling 7: no tokio
//!   even as dev-dep; the lib does sync `std::fs` inside `async fn`).
//! * `spill_file(h)` resolves `dir.join(handle.path)` — canonical `SpillPath`
//!   is an OPAQUE STRING holding `{content_id}.spill` (Ruling 10.4).
//! * FS-11 REWORKED (was: distinct-paths): ContentId-derived filenames dedup
//!   by construction → same content MUST share one file; Ruling 21.4 adds a
//!   FILE-BACKED refcount sidecar (`{id}.spill.ref`): delete-one-of-pair keeps
//!   the file, the survivor still reads; the last delete removes it.
//! * FS-08 EXTENDED (Ruling 21.4): the identical-content pair lifecycle is
//!   tested EXPLICITLY (delete one, other still reads — closes audit A-21),
//!   plus empty-sentinel pins (NOTE-5).
//! * FS-15 NEW (Ruling 15, Jebakan 2): traversal names (`..`, `/`, absolute,
//!   NUL) are REJECTED at every API boundary (FS-13/FS-14 stay blocked: the
//!   kernel exposes no gc_execution API yet).
//! * FS-06 keeps in-process `umask(000)` (Ruling 9; `libc` dev-dep, test-only).
//!   Parallel-test umask racing is HARMLESS here: the lib enforces 0600/0700
//!   explicitly via `set_permissions`, independent of ambient umask.
//!
//! Evidence rule (#991§2): every run reports tree@sha + file sha256.

use data_plane::FileSpillStore;
use kernel::error::KernelError;
use kernel::id::SpillPath;
use kernel::item::{Item, SpillCodec, SpillStore, SpilledList};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

// Zero-dep async driver (the lib is sync-inside-async; no runtime needed).
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

/// Canonical handles carry an opaque `SpillPath` string (`{content_id}.spill`,
/// Ruling 10.4); resolve against the store's base dir.
fn spill_file(dir: &Path, h: &SpilledList) -> PathBuf {
    dir.join(h.path.as_str())
}

async fn new_store(dir: &Path) -> FileSpillStore {
    FileSpillStore::new(dir).await.unwrap()
}

/// Flip one bit at an absolute file offset. Returns bytes flipped (1).
fn flip_bit_at(p: &Path, off: u64) {
    let mut f = OpenOptions::new().read(true).write(true).open(p).unwrap();
    f.seek(SeekFrom::Start(off)).unwrap();
    let mut b = [0u8; 1];
    f.read_exact(&mut b).unwrap();
    b[0] ^= 0x01;
    f.seek(SeekFrom::Start(off)).unwrap();
    f.write_all(&b).unwrap();
}

/// T-SPILL-01 · SPEC: kernel `SpillStore` trait contract (A-03: spilling
/// invisible to node semantics). Write→read_at/range/all round-trips exactly;
/// delete removes the backing file.
#[test]
fn t01_roundtrip_identity() {
    block_on(t01_roundtrip_identity_async());
}

async fn t01_roundtrip_identity_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let items = items(50);
    let h = store.write(&items).await.unwrap();
    assert_eq!(h.len, 50);
    assert!(h.total_bytes > 0);
    assert_eq!(store.read_at(&h, 0).await.unwrap().json, items[0].json);
    assert_eq!(store.read_at(&h, 49).await.unwrap().json, items[49].json);
    let r = store.read_range(&h, 10, 5).await.unwrap();
    assert_eq!(r.len(), 5);
    assert_eq!(r[0].json, items[10].json);
    assert_eq!(store.read_all(&h).await.unwrap().len(), 50);
    assert!(store.disk_footprint(&h) > 0);
    store.delete(&h).await.unwrap();
    assert!(!spill_file(&dir, &h).exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-02 · SPEC: CHECKSUM-AGREE §8 Integrity + #814.3(b). Flip ONE bit
/// mid-payload (offset derived from the file's own length prefix, never from
/// impl source). Every read path must fail with a checksum-pinned error.
#[test]
fn t02_payload_corruption_detected_on_read() {
    block_on(t02_payload_corruption_detected_on_read_async());
}

async fn t02_payload_corruption_detected_on_read_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(20)).await.unwrap();
    let p = spill_file(&dir, &h);

    let raw = std::fs::read(&p).unwrap();
    let hlen = u32::from_le_bytes(raw[0..4].try_into().unwrap()) as u64;
    let total = std::fs::metadata(&p).unwrap().len();
    assert!(4 + hlen < total, "payload must exist");
    flip_bit_at(&p, 4 + hlen + (total - 4 - hlen) / 2);

    match store.read_at(&h, 0).await {
        Err(KernelError::Invalid { message }) => {
            assert!(message.contains("checksum"), "unpinned msg: {message}")
        }
        other => panic!("want checksum-pinned Err, got {other:?}"),
    }
    match store.read_all(&h).await {
        Err(KernelError::Invalid { message }) => {
            assert!(message.contains("checksum"), "unpinned msg: {message}")
        }
        other => panic!("want checksum-pinned Err on read_all, got {other:?}"),
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-03 · SPEC: G-C7 fail-closed (header-zone corruption). Flip one bit
/// inside the length-prefix zone: reads must Err (any variant) — never Ok,
/// never panic.
#[test]
fn t03_header_zone_corruption_fail_closed() {
    block_on(t03_header_zone_corruption_fail_closed_async());
}

async fn t03_header_zone_corruption_fail_closed_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(10)).await.unwrap();
    flip_bit_at(&spill_file(&dir, &h), 2);
    assert!(
        store.read_at(&h, 0).await.is_err(),
        "header-corrupt read must fail closed"
    );
    assert!(
        store.read_range(&h, 0, 10).await.is_err(),
        "header-corrupt range must fail closed"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-04 · SPEC: G-C7 bad-magic→Err + #814.3(c). Locate the serialized
/// magic byte-array in the file (pattern search, same-length digit flip — no
/// layout offsets pinned). Read must fail with a magic-pinned error.
#[test]
fn t04_bad_magic_fail_closed() {
    block_on(t04_bad_magic_fail_closed_async());
}

async fn t04_bad_magic_fail_closed_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(5)).await.unwrap();
    let p = spill_file(&dir, &h);

    let mut raw = std::fs::read(&p).unwrap();
    // Serialized u8-array form; length-preserving single-digit flip.
    let needle = b"[83,80,73,76,76,0]";
    let pos = raw
        .windows(needle.len())
        .position(|w| w == needle)
        .expect("magic array must be observable on disk");
    raw[pos + 1] = b'9'; // 83 -> 93: same length, wrong magic
    std::fs::write(&p, &raw).unwrap();

    match store.read_at(&h, 0).await {
        Err(KernelError::Invalid { message }) => {
            assert!(message.contains("magic"), "unpinned msg: {message}")
        }
        other => panic!("want magic-pinned Err, got {other:?}"),
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-05 · SPEC: G-C7 truncation fail-closed. A 10-byte file must not
/// read Ok and must not panic on any read path.
#[test]
fn t05_truncation_detected() {
    block_on(t05_truncation_detected_async());
}

async fn t05_truncation_detected_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(20)).await.unwrap();
    let p = spill_file(&dir, &h);
    OpenOptions::new()
        .write(true)
        .open(&p)
        .unwrap()
        .set_len(10)
        .unwrap();
    assert!(
        store.read_at(&h, 19).await.is_err(),
        "truncated read_at must fail closed"
    );
    assert!(
        store.read_range(&h, 0, 20).await.is_err(),
        "truncated read_range must fail closed"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-06 · SPEC: CHECKSUM-AGREE §8 UMask + SEC-07 + #814.3(a): umask 000
/// → spill files 0600, store-created dirs 0700 (constructor enforcement).
#[test]
fn t06_umask000_gives_0600() {
    block_on(t06_umask000_gives_0600_async());
}

async fn t06_umask000_gives_0600_async() {
    let prev = unsafe { libc::umask(0o000) };
    let dir = fresh_dir();
    let sub = dir.join("sub");
    let store = FileSpillStore::new(&sub).await.unwrap();
    let h = store.write(&items(2)).await.unwrap();
    unsafe {
        libc::umask(prev);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let fmode = std::fs::metadata(spill_file(&sub, &h))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(fmode, 0o600, "SEC-07: spill file must be 0600 under umask 000");
        let dmode = std::fs::metadata(dir.join("sub"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(dmode, 0o700, "store-created dir must be 0700");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-07 · SPEC: G-C7 fail-closed (degenerate file). A 0-byte backing
/// file must Err on read — never Ok, never panic.
#[test]
fn t07_empty_file_fail_closed() {
    block_on(t07_empty_file_fail_closed_async());
}

async fn t07_empty_file_fail_closed_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(2)).await.unwrap();
    let p = spill_file(&dir, &h);
    OpenOptions::new()
        .write(true)
        .open(&p)
        .unwrap()
        .set_len(0)
        .unwrap();
    assert!(
        store.read_at(&h, 0).await.is_err(),
        "empty-file read must fail closed"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-08 · SPEC: trait `delete` doc (D39 lifecycle/GC) + #814.3(d).
/// delete removes ONLY its own file, is idempotent, leaves no residue, and
/// sibling handles keep reading.
#[test]
fn t08_delete_lifecycle_no_leak_no_collateral() {
    block_on(t08_delete_lifecycle_no_leak_no_collateral_async());
}

async fn t08_delete_lifecycle_no_leak_no_collateral_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h1 = store.write(&items(3)).await.unwrap();
    let h2 = store.write(&items(4)).await.unwrap();

    store.delete(&h1).await.unwrap();
    assert!(!spill_file(&dir, &h1).exists(), "deleted file must be gone");
    assert_eq!(store.disk_footprint(&h1), 0);
    assert!(store.read_at(&h1, 0).await.is_err(), "deleted handle unreadable");
    store.delete(&h1).await.unwrap(); // idempotent, still Ok
    // Sibling untouched and fully readable.
    assert!(spill_file(&dir, &h2).exists());
    assert_eq!(store.read_all(&h2).await.unwrap().len(), 4);
    // Ruling 21.4: IDENTICAL-content pair lifecycle, tested EXPLICITLY.
    // delete one — the other must still read fully (closes audit A-21).
    let ha = store.write(&items(5)).await.unwrap();
    let hb = store.write(&items(5)).await.unwrap();
    assert_eq!(ha.path.as_str(), hb.path.as_str());
    let pair_file = spill_file(&dir, &ha);
    store.delete(&ha).await.unwrap();
    assert!(pair_file.exists(), "refcounted file must survive first delete");
    assert!(store.disk_footprint(&hb) > 0, "footprint counts payload only");
    assert_eq!(store.read_all(&hb).await.unwrap().len(), 5);
    store.delete(&hb).await.unwrap();
    assert!(!pair_file.exists(), "last delete removes the file");
    assert_eq!(store.disk_footprint(&hb), 0);
    assert!(store.read_at(&hb, 0).await.is_err());
    // NOTE-5: empty-sentinel pins — delete is silent Ok, footprint is 0.
    let he = store.write(&[]).await.unwrap();
    store.delete(&he).await.unwrap();
    assert_eq!(store.disk_footprint(&he), 0);
    assert_eq!(store.read_all(&he).await.unwrap().len(), 0);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-09 · SPEC: `SpillWriter` trait-doc CONTRACT: "Dropping a writer
/// without calling `finish` MUST release its partial file (audit A-21)" +
/// #814.3(d) drop-no-leak.
#[test]
fn t09_dropped_writer_leaves_no_file() {
    block_on(t09_dropped_writer_leaves_no_file_async());
}

async fn t09_dropped_writer_leaves_no_file_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    {
        let mut w = store.writer().await.unwrap();
        w.push(&items(5)).await.unwrap();
        assert_eq!(w.pushed(), 5);
    }
    let leftovers: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
    assert!(leftovers.is_empty(), "partial file leaked: {leftovers:?}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-10 · SPEC: kernel trait (`read_at` O(1)-ish indexed). Correctness
/// at 5k scale: spot reads + full ordered content equality.
#[test]
fn t10_indexed_read_scales() {
    block_on(t10_indexed_read_scales_async());
}

async fn t10_indexed_read_scales_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let items = items(5000);
    let h = store.write(&items).await.unwrap();
    assert_eq!(store.read_at(&h, 4999).await.unwrap().json["id"], json!(4999));
    assert_eq!(store.read_at(&h, 0).await.unwrap().json["id"], json!(0));
    let all = store.read_all(&h).await.unwrap();
    assert_eq!(all.len(), 5000);
    assert_eq!(all[1234].json, items[1234].json);
    assert_eq!(all[4999].json, items[4999].json);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// FS-11 · SPEC: Ruling 10.4 (ContentId-derived filenames). Same content →
/// SAME file (dedup by construction); aliasing caveat pinned (NOTE-2).
#[test]
fn t11_same_content_shares_one_file() {
    block_on(t11_same_content_shares_one_file_async());
}

async fn t11_same_content_shares_one_file_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h1 = store.write(&items(7)).await.unwrap();
    let h2 = store.write(&items(7)).await.unwrap();
    // Ruling 10.4: ContentId-derived names dedup BY CONSTRUCTION.
    assert_eq!(h1.path.as_str(), h2.path.as_str(), "same content must share one file");
    assert_eq!(spill_file(&dir, &h1), spill_file(&dir, &h2));
    assert_eq!(
        store.read_all(&h1).await.unwrap(),
        store.read_all(&h2).await.unwrap()
    );
    // Ruling 21.4 refcount semantics: one file, two handles; delete one —
    // the file SURVIVES and the aliased handle still reads fully.
    store.delete(&h1).await.unwrap();
    assert!(spill_file(&dir, &h2).exists(), "shared file must survive first delete");
    assert_eq!(store.read_all(&h2).await.unwrap().len(), 7);
    // Last delete removes payload AND sidecar (no residue).
    store.delete(&h2).await.unwrap();
    assert!(!spill_file(&dir, &h2).exists(), "last delete removes the file");
    assert_eq!(store.disk_footprint(&h2), 0);
    assert!(store.read_at(&h2, 0).await.is_err(), "deleted handle unreadable");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// FS-15 · SPEC: Ruling 15, Jebakan 2 (`SpillPath::new` validates nothing).
/// Traversal names are REJECTED at every API boundary — `..`, `/`, absolute
/// paths, NUL bytes. A valid-but-absent name is a storage error, not Invalid.
#[test]
fn t15_traversal_names_rejected() {
    block_on(t15_traversal_names_rejected_async());
}

fn evil_handle(name: &str) -> SpilledList {
    SpilledList {
        path: SpillPath::new(name),
        len: 1,
        total_bytes: 1,
        codec: SpillCodec::Json,
    }
}

async fn t15_traversal_names_rejected_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    for evil in [
        "../evil.spill",
        "../../etc/passwd",
        "/tmp/x.spill",
        "7.spill\0",
        "sub/7.spill",
        "C:\\win\\7.spill",
    ] {
        let h = evil_handle(evil);
        assert!(store.read_all(&h).await.is_err(), "read must reject {evil:?}");
        assert!(store.read_at(&h, 0).await.is_err(), "read_at must reject {evil:?}");
        assert!(store.delete(&h).await.is_err(), "delete must reject {evil:?}");
        assert_eq!(store.disk_footprint(&h), 0, "footprint must fail closed for {evil:?}");
    }
    // Rejection is a validation error, and NOTHING escaped the store dir.
    let h = evil_handle("../escape.spill");
    assert!(matches!(
        store.read_all(&h).await,
        Err(KernelError::Invalid { .. })
    ));
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0, "store dir must stay empty");
    assert!(!dir.join("escape.spill").exists());
    // Valid-but-absent name: storage error (file missing), NOT a rejection.
    let ghost = evil_handle("9999999999999999999.spill");
    assert!(matches!(
        store.read_all(&ghost).await,
        Err(KernelError::SpillIo { .. })
    ));
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-12 · SPEC: no-regression — API-level OOB keeps `IndexOutOfBounds`
/// (file-level truncation is T-05's Err; the two must not mix).
#[test]
fn t12_api_oob_stays_index_out_of_bounds() {
    block_on(t12_api_oob_stays_index_out_of_bounds_async());
}

async fn t12_api_oob_stays_index_out_of_bounds_async() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(3)).await.unwrap();
    match store.read_at(&h, 3).await {
        Err(KernelError::IndexOutOfBounds { index: 3, len: 3 }) => {}
        other => panic!("want IndexOutOfBounds, got {other:?}"),
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
