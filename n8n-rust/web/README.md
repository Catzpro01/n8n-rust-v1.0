# UI web n8n-rust v0.4.0

Editor workflow single-file (`crates/n8n-server/ui/app.html`, ±23 KB),
disajikan embedded oleh binary server di `GET /`. Tanpa toolchain JS,
tanpa bundler, tanpa dependensi CDN — buka browser, langsung pakai.

## Fitur

- Canvas SVG: 12 tipe node (termasuk Code/Function, Schedule, Webhook),
  drag untuk pindah, badge warna per tipe.
- Edge: drag dari lingkaran kanan node sumber ke node target; klik edge
  untuk menghapus. Aturan If: edge pertama = cabang true (abu-abu),
  edge kedua = false (kuning).
- Panel node: metadata + editor parameters JSON (disimpan per tombol,
  divalidasi sebagai object) + output run terakhir per cabang.
- Toolbar: Contoh (fixture), Buka JSON (impor file), **Export JSON**
  (unduh workflow yang sedang diedit — nama file dari nama workflow),
  Validate, Explain, Run.
- Hasil Run: badge urutan eksekusi di tiap node + durasi ms hijau;
  ringkasan run tercatat di server (`GET /api/runs`).

## API yang dipakai UI

`POST /api/validate`, `POST /api/explain`, `POST /api/run` — semuanya
menerima workflow JSON penuh sebagai body. UI tidak menyimpan state ke
server selain saat Run (riwayat adalah efek samping server).

## Batasan UI

- Canvas 1600×900 tetap (scroll), tanpa zoom/pan dan tanpa undo.
- Editor parameters mentah JSON (tanpa form per tipe node).
- Export hanya mengunduh file; tidak ada penyimpanan server-side.
