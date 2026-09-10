# AGENT7-TABLE-MAPPING-KERNELERROR-EXTERNAL — tawaran #1041, dieksekusi pasca merge RFC §3.4 (#1276, commit 89fdb3b)

- Status: REFERENCE untuk jembatan L4/MCP masa depan. Permukaan MCP hari ini authoring-only (crates/mcp TIDAK mengimpor kernel) — tabel ini kontrak saat tool eksekusi/telemetri disambungkan.
- Basis: crates/kernel/src/error.rs pasca-merge di kernel-asli-d3bcff0 (kanonik, #1247) — diverifikasi ke disk: `From<KernelError> for NodeError` (:197), `UNKNOWN_BYTES` konstanta, catatan B-3 (TooManyOpenFiles → Permanent, pinned test §6.1).

## Tabel: KernelError → kelas retry (D14/D57/D69) → kode eksternal stabil

| Baris | KernelError (final) | From §3.4 → NodeError | Retry otomatis (D14)? | Kode eksternal stabil (usul, token bukan Debug-string) | Catatan |
|---|---|---|---|---|---|
| 1 | Expression{expr,reason} | Expression 1:1 | TIDAK | `E-EXPR` | detail expr/reason di message |
| 2 | SpillIo{kind: Interrupted\|WouldBlock\|TimedOut} | Transient{retry_after:None} | YA (backoff node-layer; None = kernel tak invent) | `TRANSIENT` | ETIMEDOUT aman krn tulis spill atomik (agent10 #1185) |
| 3 | SpillIo{kind: StorageFull\|QuotaExceeded} | ResourceExhausted{Disk, 0, 0} | TIDAK (butuh aksi operator: bersihkan/remount) | `RESOURCE_EXHAUSTED_DISK` | requested/available = UNKNOWN_BYTES(0): sentinel bernama, greppable (B-1 agent10) |
| 4 | SpillIo{kind lain: NotFound, PermissionDenied, ReadOnlyFilesystem, TooManyOpenFiles(B-3), …} | Permanent{code: ErrorCode(kind Debug)} | TIDAK | `PERMANENT_IO` | fail toward non-retry (R17); B-3 pinned: EMFILE turun ke sini s/d std stabil |
| 5 | Transient{message,retry_after} | Transient 1:1 | YA | `TRANSIENT` | mirror-eksak |
| 6 | Timeout{elapsed} | Timeout 1:1 | Keputusan node-layer (D58) | `TIMEOUT` | mirror-eksak |
| 7 | ResourceExhausted{resource,requested,available} | ResourceExhausted 1:1 | TIDAK (governor D72) | `RESOURCE_EXHAUSTED_{resource}` | mirror-eksak |
| 8 | IndexOutOfBounds\|NodeNotFound\|Codec\|Invalid | Internal (bug-class) | TIDAK | `INTERNAL` | R17: tiap kemunculan wajib jadi test |

## Aturan kontrak (konsisten #1139 saya + agent10 #1185)
1. Kode eksternal = token enum stabil (TIDAK PERNAH turunan Debug string std::io::ErrorKind — rustc-version-dependent; N-2 #1139).
2. `retry_after` hanya dari lapisan yang punya informasi (node/D14) — kernel None; MCP TIDAK auto-retry (SEP-1303 authoring-only).
3. Tabel ini = referensi TUNGGAL klasifikasi retry lintas lapisan (usul #1139 N-3); jangan lahirkan tabel kedua di spec MCP.
4. Kode RESERVED (diag.rs: E-WCB-*) tetap tidak pernah di-serve via n8n://errors/ — di luar tabel ini (registri agent1).

## Kapan disurface-kan
- resources/read n8n://errors/{kode} (registry.rs): kode baru ditambahkan BERSAMA jembatan eksekusi — bukan hari ini (crate authoring-only; content registry statis).
- Dokumentasi: tabel ini direferensikan dari AGENT7-MCP-CONFORMANCE-TESTS.md bila TC-33..36 diperluas ke error eksternal.

— agent7, 2026-09-09
