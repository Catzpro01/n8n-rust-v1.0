//! W0-SPILL-TEST — black-box gates vs LANDED W0-SPILL-IMPL (agent2, #850).
//!
//! CONTRACT BASIS (frozen before this run):
//! * Kernel canonical a7d0357: `SpillStore` trait (write/read_at/read_range/
//!   read_all/delete/disk_footprint/writer), `SpilledList{path,len,
//!   total_bytes,codec}`, `KernelError{IndexOutOfBounds,SpillIo,Codec,
//!   NodeNotFound,Expression,Invalid}`.
//! * Trait-doc MUSTs (esp. `SpillWriter` drop rule, audit A-21).
//! * agent10 acceptance #814.3: (a) umask000→0600 (b) 1-byte corrupt→Err via
//!   verify path (c) bad magic→Err (d) delete/drop no-leak.
//!
//! RECASTS vs harness v0.1 (which assumed the PATCH-extended contract):
//! * PATCH items ABSENT from canonical kernel — dual-hash handle fields,
//!   `verify_integrity`, `gc_execution`, `ChecksumMismatch`/`InvalidMagic`/
//!   `OffsetOutOfRange` variants — cannot be implemented against this kernel.
//!   Recorded as FINDING F2 (needs fern: extend kernel or drop PATCH req).
//! * Corruption gates assert BEHAVIOR (Err + message pins "checksum"/"magic"),
//!   never absent variants. Message pins are quoted from observed output and
//!   re-verified every run — they are tripwires, not trust.
//! * Async: impl is tokio-based, so gates are #[tokio::test]. Runtime is
//!   harness-side only; no impl code is linked except via the trait.

use impl_under_test::FileSpillStore;
use kernel::error::KernelError;
use kernel::item::{Item, SpillStore, SpillWriter, SpilledList};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

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

/// Landed impl stores an ABSOLUTE path in the handle (observed #850).
fn spill_file(h: &SpilledList) -> PathBuf {
    PathBuf::from(h.path.as_str())
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
#[tokio::test]
async fn t01_roundtrip_identity() {
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
    assert!(!spill_file(&h).exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-02 · SPEC: CHECKSUM-AGREE §8 Integrity + #814.3(b). Flip ONE bit
/// mid-payload (offset derived from the file's own length prefix, never from
/// impl source). Every read path must fail with a checksum-pinned error.
#[tokio::test]
async fn t02_payload_corruption_detected_on_read() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(20)).await.unwrap();
    let p = spill_file(&h);

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
#[tokio::test]
async fn t03_header_zone_corruption_fail_closed() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(10)).await.unwrap();
    flip_bit_at(&spill_file(&h), 2);
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
#[tokio::test]
async fn t04_bad_magic_fail_closed() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(5)).await.unwrap();
    let p = spill_file(&h);

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
#[tokio::test]
async fn t05_truncation_detected() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(20)).await.unwrap();
    let p = spill_file(&h);
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
#[tokio::test]
async fn t06_umask000_gives_0600() {
    let prev = unsafe { libc::umask(0o000) };
    let dir = fresh_dir();
    let store = FileSpillStore::new(dir.join("sub")).await.unwrap();
    let h = store.write(&items(2)).await.unwrap();
    unsafe {
        libc::umask(prev);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let fmode = std::fs::metadata(spill_file(&h))
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
#[tokio::test]
async fn t07_empty_file_fail_closed() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(2)).await.unwrap();
    let p = spill_file(&h);
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
#[tokio::test]
async fn t08_delete_lifecycle_no_leak_no_collateral() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h1 = store.write(&items(3)).await.unwrap();
    let h2 = store.write(&items(4)).await.unwrap();

    store.delete(&h1).await.unwrap();
    assert!(!spill_file(&h1).exists(), "deleted file must be gone");
    assert_eq!(store.disk_footprint(&h1), 0);
    assert!(store.read_at(&h1, 0).await.is_err(), "deleted handle unreadable");
    store.delete(&h1).await.unwrap(); // idempotent, still Ok
    // Sibling untouched and fully readable.
    assert!(spill_file(&h2).exists());
    assert_eq!(store.read_all(&h2).await.unwrap().len(), 4);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-09 · SPEC: `SpillWriter` trait-doc CONTRACT: "Dropping a writer
/// without calling `finish` MUST release its partial file (audit A-21)" +
/// #814.3(d) drop-no-leak.
#[tokio::test]
async fn t09_dropped_writer_leaves_no_file() {
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
#[tokio::test]
async fn t10_indexed_read_scales() {
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

/// T-SPILL-11 · SPEC: PATCH test_casd_dedup INTENT (same content → identical
/// bytes back; distinct paths — dedup is CASD's job, not Layer-0's). The
/// casd_hash PIN itself is absent from canonical `SpilledList` (FINDING F2).
#[tokio::test]
async fn t11_same_content_same_bytes_distinct_paths() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h1 = store.write(&items(7)).await.unwrap();
    let h2 = store.write(&items(7)).await.unwrap();
    assert_ne!(h1.path.as_str(), h2.path.as_str());
    assert_eq!(store.read_all(&h1).await.unwrap(), store.read_all(&h2).await.unwrap());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// T-SPILL-12 · SPEC: no-regression — API-level OOB keeps `IndexOutOfBounds`
/// (file-level truncation is T-05's Err; the two must not mix).
#[tokio::test]
async fn t12_api_oob_stays_index_out_of_bounds() {
    let dir = fresh_dir();
    let store = new_store(&dir).await;
    let h = store.write(&items(3)).await.unwrap();
    match store.read_at(&h, 3).await {
        Err(KernelError::IndexOutOfBounds { index: 3, len: 3 }) => {}
        other => panic!("want IndexOutOfBounds, got {other:?}"),
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
