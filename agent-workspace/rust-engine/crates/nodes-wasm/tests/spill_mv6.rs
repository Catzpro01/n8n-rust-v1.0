//! MV-6 gate — body_window >256 KiB => spill + truncated, diuji MELAWAN
//! implementasi FileSpillStore HASIL PORT (crates/data-plane, Ruling 10/agent1),
//! BUKAN spill_bench (Ruling-11 / #1123). Kepatuhan tambahan: izin 0600/0700
//! (Ruling-10 butir 6), SpillPath opaque `{digits}.spill`.

use data_plane::FileSpillStore;
use kernel::item::{Item, SpillStore, SpilledList};
use serde_json::json;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Wake};

// Zero-dep async driver (lib sync-inside-async; no runtime — Ruling 7).
fn block_on<F: std::future::Future>(fut: F) -> F::Output {
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

static NEXT: AtomicU64 = AtomicU64::new(0);

/// Direktori BASE yang BELUM ada; FileSpillStore::new yg membuatnya (0700).
fn fresh_base() -> PathBuf {
    let n = NEXT.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("wcb-mv6-{}-{}", std::process::id(), n))
}

fn spill_file(dir: &Path, h: &SpilledList) -> PathBuf {
    dir.join(h.path.as_str())
}

const WINDOW: u64 = 262_144; // konsensus #922 §B — body_window 256 KiB

/// Body besar (>256 KiB) harus SPILL lewat FileSpillStore hasil port;
/// guest hanya melihat window (read_at/index), data penuh tetap utuh di disk.
#[test]
fn mv6_large_body_spills_via_ported_filestore_not_spill_bench() {
    block_on(async {
        let base = fresh_base();
        let store = FileSpillStore::new(&base).await.expect("store");

        // ~60 item x ~6 KiB JSON = ~360 KiB > 256 KiB window.
        let items: Vec<Item> = (0..60u64)
            .map(|i| Item::new(json!({ "id": i, "pad": "x".repeat(6_000) })))
            .collect();
        let serialized = serde_json::to_vec(&items).unwrap().len() as u64;

        let handle = store.write(&items).await.expect("write spill");
        assert!(handle.len == 60);
        assert!(handle.total_bytes >= serialized, "total_bytes must cover payload");
        assert!(
            handle.total_bytes > WINDOW,
            "payload ({}) must exceed 256KiB window",
            handle.total_bytes
        );

        // Keputusan windowing (RFC v0.3 MV-6) atas ukuran nyata hasil port.
        let plan = nodes_wasm::gates::plan_body_window(handle.total_bytes, WINDOW);
        assert!(plan.spill && plan.truncated);
        assert!(plan.bytes_out_of_window > 0);
        assert!(nodes_wasm::gates::kernel_item_list_shape(&plan).starts_with("ItemList::Spilled"));

        // Backing file: SpillPath opaque `{digits}.spill` + izin 0600/0700 (Ruling-10 butir 6).
        let p = spill_file(&base, &handle);
        assert!(p.exists(), "spill file exists");
        let name = handle.path.as_str();
        assert!(name.ends_with(".spill"));
        assert!(!name.contains('/') && !name.contains(".."), "SpillPath tak boleh traversal");
        assert_eq!(
            std::fs::metadata(&p).unwrap().permissions().mode() & 0o777,
            0o600,
            "spill file harus 0600"
        );
        assert_eq!(
            std::fs::metadata(&base).unwrap().permissions().mode() & 0o777,
            0o700,
            "store-created dir harus 0700"
        );

        // Guest "window": item pertama bisa dibaca (streaming); data penuh utuh di disk.
        let first = store.read_at(&handle, 0).await.unwrap();
        assert_eq!(first.json, items[0].json);
        let full = store.read_all(&handle).await.unwrap();
        assert_eq!(full.len(), items.len());

        store.delete(&handle).await.unwrap();
        assert!(!p.exists(), "delete menghapus berkas");
        let _ = std::fs::remove_dir_all(&base);
    });
}

/// Body kecil tetap Inline (tidak disentuh FileSpillStore).
#[test]
fn mv6_small_body_stays_inline() {
    let plan = nodes_wasm::gates::plan_body_window(100_000, WINDOW);
    assert!(!plan.spill && !plan.truncated && plan.bytes_out_of_window == 0);
    assert!(nodes_wasm::gates::kernel_item_list_shape(&plan).starts_with("ItemList::Inline"));
}
