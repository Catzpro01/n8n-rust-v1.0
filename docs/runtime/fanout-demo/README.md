# fanout-demo — bukti jalannya harness (2026-09-10)

Hasil `scripts/parallel-fanout.sh scripts/fanout/manifest.example --jobs 4 --timeout 300 --outdir docs/runtime/fanout-demo`:

**5 tugas · 5 OK · jumlah durasi worker 1.53 s · wall-clock 0.64 s (≈2.4× lebih cepat).**

Direktori ini regenerable; aman dihapus. File `.log` hanya ada di mesin yang menjalankannya
(`*.log` di-gitignore) — `SUMMARY.md`/`SUMMARY.tsv`/`*.status` yang di-commit di sini adalah
bukti ringkasannya.

Jalankan ulang:

```bash
scripts/parallel-fanout.sh scripts/fanout/manifest.example --jobs 4 --timeout 300
```
