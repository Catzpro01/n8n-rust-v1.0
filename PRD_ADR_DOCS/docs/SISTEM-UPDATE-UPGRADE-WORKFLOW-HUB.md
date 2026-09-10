# 🚀 SISTEM UPDATE & UPGRADE WORKFLOW HUB
## Panduan Kemudahan Pembaruan Template (Update) & Peningkatan Mesin (Upgrade)
**Status:** ✅ RESMI DISAHKAN (Mandat Pemilik Proyek - 2026-09-09)  
**Filosofi:** Kemudahan seperti App Store / Play Store — 1-Klik, Nol Kerumitan, Aman dengan Rollback O(1).

---

## 1. Prinsip Utama: Decoupled Architecture

Untuk menjamin pembaruan berjalan sangat mudah, arsitektur Workflow Hub memisahkan secara tegas antara **Lapisan Konten Template** dan **Lapisan Mesin Engine**:

```
+-------------------------------------------------------------------------+
|                  WORKFLOW HUB ECOSYSTEM ARCHITECTURE                    |
+-------------------------------------------------------------------------+

  [LAPISAN KONTEN: TEMPLATE & FEEDS]  <--- MUDAH DI-UPDATE (Tanpa Restart)
  - Katalog Template Aktif (Full-Canvas DAG)
  - Katalog Template Pasif (1-Node Capsule)
  - Feed Intelijen (FRED Makro, SEC EDGAR, FinBERT, Web Scraper)
  -> Update via: `hub update` atau 1-Klik di UI

  ------------------------------ [ISOLASI] ------------------------------

  [LAPISAN MESIN: RUST RUNTIME ENGINE] <--- MUDAH DI-UPGRADE (Atomic Switch)
  - Biner Kanonik Rust (<500MB RAM, Data-Plane FileSpillStore)
  - WASM Community Bridge (32MB Sandbox)
  - Rosetta Schema Migrator (Auto-Transform JSON versi lama)
  -> Upgrade via: `hub upgrade` dengan Garansi Rollback O(1)
```

---

## 2. Mekanisme Pembaruan Template (Easy Update)

### 2.1 Dua Jalur Pembaruan Cerdas (Smart Update Path)
Ketika pembuat template merilis versi baru (misal update dari v1.0.0 ke v1.1.0):

1. **Jalur 1: Silent-Swap (Pembaruan Halus Otomatis)**
   * **Kondisi**: Pembaruan bersifat perbaikan bug, penyesuaian selector CSS web scraping, atau peningkatan kecepatan tanpa mengubah parameter input/output (*Non-Breaking Schema*).
   * **Tindakan**: Hub secara otomatis memperbarui template di latar belakang (*hot-reload*). Pengguna langsung menikmati perbaikan tanpa perlu konfigurasi ulang.
2. **Jalur 2: Promote with Visual Diff (Pembaruan dengan Konfirmasi)**
   * **Kondisi**: Pembaruan mengubah parameter utama (misal ada field baru yang wajib diisi atau perubahan struktur output).
   * **Tindakan**: Muncul lencana (badge) *"Pembaruan Tersedia"* pada kanvas. Pengguna dapat melihat ringkasan perbedaan (diff view) dan mengklik tombol `[Terapkan Pembaruan]` kapan pun siap.

### 2.2 O(1) Instant Rollback
* Setiap template menyimpan riwayat versi sebelumnya di direktori arsip CASD: `/opt/agent-workspace/hub-templates/archive/<id>/`.
* Jika versi baru menghasilkan perilaku yang tidak diinginkan pada alur kerja Anda, pengguna cukup mengklik `[Rollback ke Versi Sebelumnya]` atau menjalankan perintah `hub rollback <template-id>`. Sistem langsung mengembalikan versi stabil dalam waktu <1 detik.

---

## 3. Mekanisme Peningkatan Mesin (Easy Upgrade)

### 3.1 Peningkatan Atomik Tanpa Downtime (Atomic Binary Swap)
* Peningkatan biner engine Rust menggunakan teknik symlink atomik:
  ```
  /usr/local/bin/n8n-rust-engine -> /opt/engine/releases/v0.2.0/bin
  ```
* Biner baru diunduh, diverifikasi hash kriptografisnya (BLAKE3), dan diuji kesehatannya (*self-health check*) sebelum symlink dialihkan.
* Jika biner baru gagal dalam 5 detik pasca peluncuran, symlink otomatis dikembalikan ke rilis sebelumnya (*fail-safe auto recovery*).

### 3.2 Rosetta Schema Migrator (Anti-Patah Format)
* Tidak ada risiko alur kerja lama menjadi rusak saat engine di-upgrade.
* Modul **Schema Rosetta** secara otomatis mengonversi sintaks node versi lama ke format terkini saat alur kerja dimuat ke kanvas.

---

## 4. Utilitas CLI Satu Pintu: `hub`

Telah disediakan perintah CLI tunggal yang terintegrasi di terminal VPS:

| Perintah | Fungsi Utama |
| :--- | :--- |
| `hub status` | Memeriksa versi engine, jumlah template terpasang, dan notifikasi update baru. |
| `hub update` | Memeriksa dan memperbarui seluruh template katalog ke rilis termutakhir. |
| `hub update <template-id>` | Memperbarui 1 template spesifik secara terarah. |
| `hub rollback <template-id>` | Mengembalikan template ke versi stabil sebelumnya secara instan. |
| `hub list` | Menampilkan seluruh katalog template (Aktif & Pasif) yang siap dipakai. |
| `hub upgrade` | Memeriksa dan meningkatkan biner engine Rust ke rilis stabil terbaru. |
