# 🧠 PANDUAN SISTEM MEMORI BERJENJANG 4-LAYER (`mem` / `agent-mem`)

Sistem memori berjenjang (*Hierarchical 4-Layer Memory*) telah terpasang secara global di `/usr/local/bin/mem` (alias: `agent-mem`) pada VPS `103.171.85.230`. Sistem ini dirancang untuk mencegah **context rot**, menjaga penggunaan token tetap hemat, dan memungkinkan pewarisan wawasan teknis antar-agen.

Database bersama tersimpan di `/var/lib/agent-memory/memory.db` menggunakan engine **SQLite WAL + FTS5 (Full-Text Search)**.

---

## 🏛️ Arsitektur 4-Lapisan Memori

```
┌─────────────────────────────────────────────────────────────┐
│ LAYER 0: CORE MEMORY (RAM / Identitas & Aturan Utama)       │
│ • Ukuran: Sangat kecil (~100 token, selalu aktif)           │
│ • Isi   : Persona peran, aturan kantor, direktif mutlak    │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│ LAYER 1: WORKING MEMORY (Scratchpad / Task State)           │
│ • Ukuran: Menengah (hanya ada selama task aktif)            │
│ • Isi   : Milestone saat ini, variabel kerja, status step   │
│ • Siklus: Direset tiap task selesai setelah disintesis      │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│ LAYER 2: ROLE DOSSIER (Episodic / Wawasan Peran)            │
│ • Ukuran: Dinamis (top 5-10 solusi teknis paling relevan)   │
│ • Isi   : Trik kompilasi, bugfix sukses, pola desain        │
│ • Manfaat: Agent baru langsung mewarisi kepintaran rekan     │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│ LAYER 3: ARCHIVAL & KNOWLEDGE GRAPH (Storage / Disk FTS5)   │
│ • Ukuran: Tak terbatas (query on-demand via FTS5 / BM25)    │
│ • Isi   : Dokumen spesifikasi, ADR arsitektur, catatan tim │
└─────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Panduan Perintah CLI (`mem`)

### 1. Layer 0: Core Memory (`mem core`)
Melihat persona diri, kapabilitas peran, dan aturan sistem yang tidak boleh dilanggar.
```bash
# Melihat core memory peran backend
mem core backend

# Melihat core memory peran QA / DevOps / Docs / Matt
mem core qa
mem core matt

# Output JSON
mem core backend --json
```

### 2. Layer 1: Working Memory (`mem working`)
Tempat mencatat variabel kerja aktif (scratchpad) dan checkpoint tugas per-user. Masing-masing agent memiliki working memory terisolasi.
```bash
# Simpan checkpoint tugas aktif
mem working set current_task "Refaktor skema SQLite WAL"
mem working set step "2/4 unit test passed"

# Ambil status kerja aktif
mem working get

# Ambil nilai key spesifik
mem working get current_task

# Reset working memory setelah tugas selesai
mem working clear
```

### 3. Layer 2: Role Dossier (`mem learn` & `mem dossier`)
Menyimpan dan membaca wawasan teknis sukses per peran. Saat satu agent menemukan solusi bug atau trik optimasi, simpan di sini agar agent generasi berikutnya langsung mewarisinya.
```bash
# Simpan solusi teknis sukses
mem learn "Gunakan PRAGMA busy_timeout=5000 untuk mencegah SQLite database locked" --role backend

# Baca akumulasi wawasan peran backend
mem dossier backend

# Baca seluruh wawasan dari semua peran
mem dossier --limit 15
```

### 4. Layer 3: Archival Knowledge (`mem store` & `mem search`)
Penyimpanan dokumen jangka panjang permanen dengan pencarian teks lengkap berkecepatan tinggi via **SQLite FTS5**.
```bash
# Simpan dokumen / catatan arsitektur baru
mem store "Panduan Optimasi WAL" "Konfigurasi PRAGMA wal_checkpoint(TRUNCATE) disarankan dieksekusi secara periodik." --tags "sqlite,performance,wal"

# Cari arsip dokumen berdasarkan kata kunci
mem search "rules"
mem search "timeout"
mem search "architecture" --limit 3

# Format JSON
mem search "database" --json
```

### 5. Snapshot Sistem (`mem snapshot`)
Melihat ringkasan eksekutif seluruh status 4 layer memori dalam 1 layar.
```bash
mem snapshot
```

---

## 📋 Cheatsheet Cepat untuk Diberikan ke AI Agent di Arena

```text
Di terminal VPS ini sudah tersedia alat memori berjenjang bernama 'mem'.
Gunakan alat ini untuk menjaga state tugas dan berbagi pengetahuan teknis:

1. Lihat aturan dan kapabilitas peran Anda:
   mem core <backend|frontend|qa|devops|docs|matt>

2. Catat progress / checkpoint tugas aktif Anda:
   mem working set task_id "T01"
   mem working set progress "implementasi endpoint register selesai"
   mem working get

3. Jika Anda menemukan trik/solusi teknis baru, simpan ke wawasan tim:
   mem learn "Solusi teknis yang berhasil Anda temukan" --role backend

4. Cari dokumen spesifikasi, aturan, atau arsip proyek:
   mem search "kata_kunci"

5. Lihat rangkuman memori:
   mem snapshot
```
