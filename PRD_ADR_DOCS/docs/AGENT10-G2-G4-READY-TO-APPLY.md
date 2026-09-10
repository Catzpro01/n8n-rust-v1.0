# G-2 + G-4 — READY-TO-APPLY REFERENCE (untuk @agent2; teks resmi = Ruling 41 @matt #1345 §3/§4)

| | |
|---|---|
| **Penyusun** | agent10 (ROLE_COMPLIANCE) — kompilasi satu tempat dari keputusan @matt #1345 §3 dan §4 |
| **Status** | Keputusan RESMI (matt), otorisasi eksekusi @fern #1350; 004 BELUM pernah dijalankan di produksi → EDIT 004 LANGSUNG, JANGAN 005 (argumen #1134/#1210) |
| **File** | `crates/storage/migrations/004_item_lineage.sql` |

## 1. G-2 — CHECK 4-cabang (repr ↔ inputs_*)

Tambahkan ke `lineage_edge` (setelah kolom unknown_reason; CHECK tabel-level sebelum PRIMARY KEY):

```sql
    -- G-2 (Ruling 41 #1345§3 @matt): invariant komentar -> kendala DB. 4 cabang mutual-eksklusif + ekshaustif.
    CHECK (
      (repr = 0 AND inputs_exact   IS NOT NULL AND inputs_range  IS NULL AND inputs_digest IS NULL AND unknown_reason IS NULL) OR
      (repr = 1 AND inputs_range   IS NOT NULL AND inputs_exact  IS NULL AND inputs_digest IS NULL AND unknown_reason IS NULL) OR
      (repr = 2 AND inputs_digest  IS NOT NULL AND inputs_exact  IS NULL AND inputs_range  IS NULL AND unknown_reason IS NULL) OR
      (repr = 3 AND unknown_reason IS NOT NULL AND inputs_exact  IS NULL AND inputs_range  IS NULL AND inputs_digest IS NULL)
    )
```

Catatan matt: `CHECK (repr IN (0,1,2,3))` lama menjadi redundan (tercakup) — boleh dihapus atau ditinggal; keduanya benar. Yang WAJIB: 4 cabang di atas.

## 2. G-2 — test WAJIB (dengan kontrol positif, per #1345§3)

| # | INSERT | Hasil |
|---|---|---|
| T1 | repr=0 + inputs_exact NULL | DITOLAK |
| T2 | repr=3 + unknown_reason NULL | DITOLAK |
| T3 | repr=0 + inputs_exact DAN inputs_range keduanya terisi | DITOLAK (mutual-eksklusif) |
| T4 | repr=0..3 masing-masing dengan kolom yang benar | DITERIMA (KONTROL POSITIF — tanpa ini, CHECK yang menolak segalanya akan membuat T1-T3 "lulus" palsu) |

Mutan (Ruling 27): hapus CHECK → minimal satu dari T1-T3 harus GAGAL. Tempel baris FAIL + restore + hijau.

## 3. G-4 — indeks reverse

```sql
CREATE INDEX IF NOT EXISTS idx_lineage_edge_output_ref ON lineage_edge(output_ref);
```

Alasan matt #1345§4: biaya asimetris (gratis sekarang, rebuild penuh nanti); output_ref = BLOB 16B (BLAKE3-128) → indeks kecil. "Tunda-tertulis" = rekam jejak buruk (R39a).

SYARAT matt: UKUR, jangan asumsikan — sebelum vs sesudah, pada 100k baris:
```
laporan: ukuran berkas DB (du) + waktu INSERT 100k baris (time cargo test / skrip) — sebelum, sesudah
```
Karakter ini memutuskan apakah indeks diterima permanen.

## 4. Check list verifikasi (dijalankan @agent10 setelah mendarat — janji #1356)

1. `cargo test -p storage` → semua hijau (termasuk T1-T4 baru)
2. `cargo clippy -p storage` → 0 warning
3. Mutan G-2: hapus CHECK → T1-T3 GAGAL → restore → hijau
4. Mutan G-4: hapus idx_lineage_edge_output_ref → query reverse → SCAN → GAGAL (per #1195 M1d)
5. Angka 100k (ukuran DB + waktu INSERT) sebelum/sesudah — disitir sebagai bukti, bukan dugaan

— agent10 (sesi B), 2026-09-09 14:4x
