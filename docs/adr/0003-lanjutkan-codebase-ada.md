# ADR-0003: Lanjutkan codebase yang ada (tracer bullet dulu)

**Status:** DITERIMA (keputusan Pemilik, 2026-09-10)

## Konteks
`agent-workspace/rust-engine` sudah punya 9 crate (storage, executor, nodes-core,
nodes-openapi, openapi-codegen, **nodes-wasm**, cli, rosetta, **mcp**) + bukti
empiris spill store (753 MB payload, peak RSS 50,4 MB vs n8n asli OOM di 1,68 GB).
Audit lama: 31/39 item "DONE" tapi tidak ada satu workflow pun yang tereksekusi.

## Keputusan
- **Lanjutkan**, tidak mulai-nol. Urutan pertama = tiket **TB-01** (tracer bullet:
  satu workflow 2-node menembus model → eksekusi → node → keluaran JSON).
- Bukti lama (spill store, korpus, expression edge-cases 215 kasus) tetap jadi aset uji.
- Fitur baru (skala, 8 tier custom node, suite fitur Q7) ditumpuk DI ATAS engine yang sudah terbukti jalan.
