# NODE COVERAGE REPORT korpus real (agent4, 2026-09-09)

Dianalisis: 96 file tpl-* (template publik n8n.io). MVP = 26 node PRD-2 7.3 (T1 13 + T2 8 + T3 5).

- File seluruh-node-nya di MVP-26: **6 (6%)**
- File seluruh-node-nya di Tier-1 (13): **5 (5%)**

## Implikasi
1. Hanya 6% template publik modern yang bisa dieksekusi penuh oleh MVP-26 -> gate L2 harus memakai korpus STRATIFIKASI: subset MVP-runnable utk exec-diff, sisanya utk L1 impor (gagal impor eksplisit = perilaku benar, PRD-2 10.1).
2. Template publik n8n.io saat ini didominasi AI/LLM (@n8n/n8n-nodes-langchain.*: 28 agent, 23 lmChatOpenAi, 17 outputParserStructured, 14 openAi) - di luar target MVP; jangan jadikan statistik ini alasan menambah node AI.
3. Node LUAR-MVP paling sering (jumlah file): stickyNote 38 (dekoratif UI - harus DIABAIKAN dlm hitungan L1, bukan dianggap node), agent 20, lmChatOpenAi 13, outputParserStructured 11, function 10 (legacy Code), openAi 9, cron 8, splitInBatches 8 (Loop Over Items - tidak ada di MVP!), googleDrive 8, writeBinaryFile 6, formTrigger 6, spreadsheetFile 6, readBinaryFile 5, rssFeedRead 5, emailSend 5.
4. Node kandidat tambahan berbasis data (usul utk T-4): cron, splitInBatches (Loop Over Items - 8 file), function/functionItem (legacy Code - 10 file), spreadsheetFile + read/writeBinaryFile (keluarga file - era awal), rssFeedRead, formTrigger. Catatan: readBinaryFile/writeBinaryFile/spreadsheetFile TIDAK ADA di Tier1/2 padahal dominan di template era awal (2019-2021) - relevan utk klaim kompatibilitas L1 jangka panjang.
5. Node MVP yang paling sering dipakai (validasi arah daftar benar): httpRequest 167, set 92, code 63, googleSheets 59, if 53, manualTrigger 38, wait 28, merge 21, scheduleTrigger 17, telegram 17, webhook 13, gmail 12, slack 10, noOp 9, respondToWebhook 7, splitOut 7.

Metode: python scan node.type per file; skrip /opt/agent-workspace/qa/coverage_scan.py (reproducible).
