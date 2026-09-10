# PROTOKOL-RAPAT-KILAT-KONSENSUS.md — Cara Paling Efisien Musyawarah Swarm AI
**Versi:** 1.0.0-CANONICAL | **Otoritas:** Pemilik Proyek, @fern (Spokesperson), @matt (Chief AI Orchestrator)

Dokumen ini mendefinisikan mekanisme rapat paling efisien (*high-efficiency swarm plenary*) untuk agen AI otonom agar mencapai **bahasa yang sama, kesepakatan antarmuka yang mutlak, dan kerja kompak tanpa pemborosan token**.

---

## 1. MENGAPA METODE INI PALING EFISIEN?
- **Masalah Rapat Tradisional AI**: Chat bebas tanpa struktur menyebabkan looping ucapan sopan, pembahasan melebar, dan context drift yang membakar ribuan token sia-sia.
- **Solusi Protokol 3-Babak (Structured 3-Round Synod)**:
  1. **Babak 1 (Orchestrator)**: Peletakan Draf Piagam (5 Klausul Kanonik).
  2. **Babak 2 (Roll-Call Suara Seluruh Agen)**: Tepat 1 respons per agen menggunakan template data terstruktur (Ballot Payload).
  3. **Babak 3 (Ratifikasi Bersama)**: Penandatanganan Piagam Konsensus (Treaty of Consensus) dengan verifikasi hash BLAKE3.

---

## 2. LIMA KLAUSUL KESEPAKATAN BERSAMA
1. **Klausul Bahasa Kanonik**: Tunduk pada `BAHASA-BERSAMA.md` (BLAKE3 32-byte digest, SHA-256 berkas, `PayloadRef` zero-copy, single-tenant <500MB RAM).
2. **Klausul Kedaulatan Domain**: Tunduk pada `ROSTER-DAN-DIREKTORI-PERAN-SWARM.md` (hormat hak kedaulatan crate agen lain, tidak menyentuh kode di luar domainnya).
3. **Klausul Wajib Konsensus Komunitas**: Tunduk pada `SOP-KONSENSUS-KOMUNITAS.md` (Proposal RFC $	o$ 2 Reviewer $	o$ Kuorum $	o$ Baru Eksekusi).
4. **Klausul Orkestrasi & Bebas Pekerjaan Kasar**: Menghormati wewenang delegasi Lead Architect (@matt); pekerjaan kotor/teknis diselesaikan oleh agen pelaksana.
5. **Klausul Budaya Kompak**: Saling mereview, terbuka terhadap masukan, zero-ego, dan transparan dalam pengujian.
