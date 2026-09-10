# ADR-0002: Target skala — 100rb s/d 1jt node, bertahap & terukur

**Status:** DITERIMA (keputusan Pemilik, 2026-09-10, ronde grill #1–2)
**Hukum:** tanpa benchmark tidak ada klaim (D4 Resource Efficiency Absolut)

## Target (dua bacaan, bertahap)
| Fase | Target | Mesin | Kriteria lulus |
|---|---|---|---|
| **B0 baseline** | workflow panjang-berat harian (ribuan node, node berat) | **1 vCPU / 500 MB RAM** | jalan tanpa OOM/swap-thrash, budget guard aktif |
| **B1** | 100.000 node graph tunggal (muat + eksekusi) | 1 vCPU / 500 MB | peak RSS < 500 MB total mesin |
| **B2** | 500.000 node | 2 vCPU / 2 GB (VPS) | peak RSS terukur & dilaporkan |
| **B3** | 1.000.000 node | 2 vCPU / 2 GB + 4 GB swap | peak RSS terukur; swap hanya sebagai guard, bukan jalan tol |
| **B4** | konkurensi: total 100rb+ node aktif simultan | VPS | throughput + RSS terukur |

## Metode
- **Benchmark harness permanen** di repo (crate `bench`): generate graph deterministik (N node, pola linier/tree/DAG acak), ukur RSS via `/proc/<pid>/status` (VmHWM), waktu, item/s.
- Setiap fase baru = tiket dengan acceptance criteria angka, bukti output harness di commit.
- Strategi teknis (prinsip, detail di spec): graph statis = adjacency ringkas (arena, bukan `Box` per node); state per-node di luar memori (spill store, bukti ada: 753 MB payload @ RSS 50,4 MB); eksekusi streaming per-item, bukan materialisasi seluruh daftar item.
