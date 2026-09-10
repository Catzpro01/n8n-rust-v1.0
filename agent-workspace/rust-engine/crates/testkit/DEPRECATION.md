# testkit — DEPRECATED

**Status**: Removed from workspace (ERR-031, Ruling 36 aftermath)

## Reason for Deprecation

testkit was written against the **kernel STUB API** (rust-engine/crates/kernel before Ruling 36 unify), not the **kernel kanonik** (kernel-asli-d3bcff0/crates/kernel).

After the Ruling 36 unify (commit ebfa533), testkit had **19 compile errors** due to structural API mismatches:
- KernelError variants changed (Storage → SpillIo/Codec with different syntax)
- ResourceHint fields changed (weight/max_concurrency → cpu/io/memory/buffering/cacheable/resumable)
- NodeDescriptor fields changed (missing credentials, description, icon)
- NodeContext structure completely different (missing 15+ fields)
- NodeOutput.items → NodeOutput.branches
- NodeError variants changed

## Attempted Fixes

1. **Option (a)**: Full rewrite against kernel kanonik API — estimated 2-3 hours, high complexity
2. **Option (b)**: Simplify to keep only MockNode/MockNodeContext — still 18 errors after removing InMemory stores
3. **Option (c)**: Deprecate completely — **CHOSEN** (10 minutes, clean solution)

## Why Option (c)?

- testkit has **0 dependents** (no other crate uses it)
- Real storage tests should use the actual storage crate, not in-memory mocks
- MockNode/MockNodeContext could be recreated in individual test files if needed
- Maintaining compatibility with kernel kanonik is not worth the effort for unused code

## Original Purpose

testkit provided:
- `InMemorySpillStore`: HashMap-based spill store for testing (no disk I/O)
- `InMemoryBlobStore`: HashMap-based blob store for testing (no disk I/O)
- `MockNode`: Configurable mock implementing kernel::Node
- `MockNodeContext`: Builder for test execution contexts
- `TestFixture`: Convenience builder for common test scenarios

## Replacement

For testing storage:
- Use actual storage crate with tempfile-based SQLite
- See `crates/storage/src/lineage.rs` tests for examples

For testing node logic:
- Create inline mocks in test files
- Use kernel::Node trait directly

## History

- Created by agent3 (2026-09-09)
- Deprecated by agent2 (2026-09-09) per ERR-031 resolution
- Code preserved in crates/testkit/ for reference, but not compiled

## References

- ERR-031: agent7 #1146
- Ruling 36: matt #1260
- Unify commit: ebfa533
- Deprecation discussion: agent2 #1338

## Rebuild trigger

testkit dibangun kembali bila API kernel dinyatakan STABIL, yaitu sesudah:
- W0-CONTEXT merged (656e265 + review 2/2)
- json_depth_within pindah ke kernel (Ruling 41)
- tidak ada perubahan kernel/src/*.rs selama 3 hari kerja

Yang dibangun kembali BUKAN mock InMemory. Yang dibangun kembali adalah fixture builder untuk
NodeDescriptor / ItemList / ExecutionContext yang dipakai test crate lain, dan helper yang
menjalankan SQLite nyata di tmpdir.

Dari 29 API publik di lib.rs lama, tinjau mana yang masih diinginkan — jangan salin semuanya.
