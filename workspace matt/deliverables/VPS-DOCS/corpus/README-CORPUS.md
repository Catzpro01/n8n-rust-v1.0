# CORPUS WORKFLOW N8N (differential testing)
Tujuan (PRD-2 S10.2 & Fase 1 1.2): minimal 50 workflow n8n NYATA sebagai korpus differential testing.
Aturan: BUKAN ditulis tangan untuk lulus; tiap file wajib punya provenansi yang bisa ditelusuri.
## Isi saat ini (2026-09-09, agent4)
- 171 template publik NYATA dari n8n.io template store (api.n8n.io) -> tpl-*.json, apa adanya, provenansi di MANIFEST-CORPUS-ALL.md
- _fixtures-repo-internal/: 12 file fixtures repo resmi n8n (ai-workflow-builder reference-workflows + instance-ai computer-use) - TIDAK dihitung sbg corpus real (bukan ekspor n8n; tanpa field metadata ekspor; mayoritas node AI/LLM). Disimpan utk uji L1 parser & katalog deviasi saja.
## KRITERIA PEMILIHAN 171 TEMPLATE (jawaban formal pertanyaan matt #289/#330; lihat juga pesan #294)
Metode: BUKAN sampel acak dan BUKAN pilihan "paling populer". Korpus = SEMUA template publik yang DITEMUKAN HIDUP dalam frame id api.n8n.io yang diproba, dengan 2 sumber:
1. agent4: probe kontinu TANPA step, id 1..700 (setiap id dicoba; yang publik & unik disimpan). Skrip: /opt/agent-workspace/qa/harvest_templates.py (1..260) & harvest_templates2.py (261..700). Dedup: signature 500 byte pertama dari nodes.
2. agent1: probe step-10 id 701..2500 + kumpulan id besar (sparse). Dedup sama.
Distribusi id: 56 file id <=700, 75 file 701..2500, 40 file >2500 (min 1, median 1751, max 17103, 26 gap >100).
BIAS TERUKUR (per band): file mengandung node AI/langchain = 0/56 (0%) di <=700, 11/75 (15%) di 701..2500, 29/40 (73%) di >2500.
Konsekuensi bias (tercatat di ANALISIS-KORPUS-NODE.md S1.4 oleh matt):
- Korpus berat ke template pra-era-AI (frame 1..2500 rapat). Angka "pasar AI 23%" = UNDERCOUNT vs populasi penuh n8n.io.
- Angka "13% workflow jalan penuh oleh MVP-26" kemungkinan LEBIH TINGGI dari populasi sebenarnya.
- Kesimpulan long-tail (Temuan 1-4 matt) terjadi di wilayah rapat yang TIDAK bias -> KUAT & tidak berubah.
- Utk produk non-AI, frame 1..2500 justru lebih representatif daripada populasi penuh.
## Konvensi penamaan
- tpl-<templateId>-<slug>.json : template publik n8n.io
- <sumber>-<nama>.json : sumber lain; metadata di MANIFEST-*.md per kontributor
## Verifikasi (Pasal 2.5 Zero Fake Work)
Setiap batch menyertakan sha256 + provenansi sumber + tanggal. File tidak diedit setelah diunduh (sha256 di MANIFEST-CORPUS-ALL.md = file apa adanya).
