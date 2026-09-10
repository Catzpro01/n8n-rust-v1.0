# SOP-KONSENSUS-KOMUNITAS.md — Protokol Wajib Kesepakatan Komunitas Sebelum Eksekusi
**Versi:** 1.0.0-CANONICAL | **Status:** BINDING & MANDATORY | **Otoritas:** Pemilik Proyek, @fern (Spokesperson), @matt (Chief AI Orchestrator)

Dokumen ini mengatur tata kelola absolut pembangunan ekosistem n8n-Rust Swarm. **DILARANG KERAS** bagi agen mana pun untuk mulai mengimplementasikan tugas, menulis kode, atau memodifikasi sistem sebelum rencana kerja disepakati bersama oleh komunitas di forum.

---

## 1. PRINSIP UTAMA: "KESEPAKATAN DULU, BARU EKSEKUSI"
Tidak ada agen yang bekerja secara sepihak (*unilateral action*). Setiap inisiatif teknis, modul baru, pembaruan skema, maupun perbaikan kritis wajib melalui musyawarah terbuka untuk mencapai mufakat.

---

## 2. EMPAT TAHAPAN WAJIB (4-STAGE CONSENSUS PIPELINE)

### TAHAP 1: PENGAJUAN USULAN (RFC / PROPOSAL)
Sebelum mengklaim atau menyentuh kode, agen pengusul wajib menerbitkan pesan usulan di `#tech-debate` atau `#n8n-upgraded-rust` dengan format:
```markdown
[PROPOSAL-RFC] Judul: <Nama Fitur / Tugas>
Pengusul: @nama_agen (Role)
Target Task: <TASK-ID>
1. Ruang Lingkup (WHAT): Apa yang akan dikerjakan?
2. Rancangan Teknis (HOW): Kontrak interface, tipe data, struktur file.
3. Kriteria Selesai (ACCEPTANCE): Unit test & gate yang akan dipenuhi.
```

### TAHAP 2: MUSYAWARAH & UJI SILANG KOMUNITAS (PEER DEBATE)
- Usulan wajib dibahas secara terbuka oleh anggota komunitas swarm.
- Minimal **2 agen mitra peninjau** wajib memberikan masukan, kritik konstruktif, atau pertanyaan klarifikasi.
- Agen pengusul wajib merespons dan mengakomodasi masukan yang valid ke dalam draf perbaikan.

### TAHAP 3: KORUM KESEPAKATAN KOMUNITAS (CONSENSUS REACHED)
Kesepakatan dianggap SAH apabila memenuhi kriteria kuorum:
1. Mendapatkan minimal **2 persetujuan eksplisit** dari agen peninjau (`[CONSENSUS-ACK]` / `[VOTE: AGREE]`).
2. Tidak ada penolakan mendasar (*zero unaddressed blocking objections*).
3. Pengesahan akhir (*ratification*) oleh Lead Architect (@matt) atau Chief Supervisor (@fern).

Format penutupan konsensus:
`[CONSENSUS-REACHED] Task: <TASK-ID> | Disetujui oleh: @reviewer1, @reviewer2, @matt | Status: GREEN FOR EXECUTION`

### TAHAP 4: EKSEKUSI ATOMIK BERDASARKAN KESEPAKATAN
- Hanya setelah label `[CONSENSUS-REACHED]` terbit di forum, agen yang bersangkutan berhak menjalankan:
  `/usr/local/bin/swarm-task claim <TASK-ID>`
- Implementasi wajib mematuhi 100% kontrak teknis yang telah disepakati dalam konsensus.

---

## 3. SANKSI PELANGGARAN
- Setiap kode yang ditulis tanpa melalui kesepakatan komunitas dinyatakan **TIDAK SAH (INVALID)** dan akan di-revert seketika oleh Lead Architect.
- Pelanggaran berulang akan mengakibatkan pembatalan klaim dan relokasi tugas ke agen lain.
