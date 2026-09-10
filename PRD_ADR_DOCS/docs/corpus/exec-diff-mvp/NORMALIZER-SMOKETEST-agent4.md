# SMOKE-TEST NORMALIZER atas exec-diff-mvp (agent4)

Uji chain normalisasi workflow (N-20 strip, N-21 timezone) + klasifikasi trigger (N-22) pada 13 file subset nyata.

| File | Status | Node | Trigger | Exec | sha-in | sha-norm |
|---|---|---|---|---|---|---|
| tpl-119-webhook-returning-xml.json | OK | 4 | replaceable | SKIP | 2e41ad4ad06c | a925f16c12cd |
| tpl-122-report-phishing-websites-to-steam-and-cloudflare.json | OK | 9 | replaceable | SKIP | ed9366ed36b0 | 1a7d924d648b |
| tpl-2221-youtube-advanced-rss-generator-with-telegram-forma.json | OK | 20 | skip | SKIP | a8a0c14f95b2 | e5452374fe72 |
| tpl-226-receive-google-sheet-data-via-rest-api.json | OK | 2 | replaceable | SKIP | 61a5418d63d0 | 95324caa49d7 |
| tpl-27-telegram-sticker-bot.json | OK | 4 | skip | SKIP | 4b22a546b08c | 02c3450246eb |
| tpl-4-send-selected-github-events-to-slack.json | OK | 4 | skip | SKIP | d9b1a0484b89 | 08fea1a4b6e4 |
| tpl-526-assign-values-to-variables-using-the-set-node.json | OK | 2 | deterministic | RUN | 5b89bb4b5b84 | dff719f3443c |
| tpl-581-execute-set-node-based-on-function-output.json | OK | 5 | deterministic | RUN | 2e7bcc50380d | 78ce7c3418f3 |
| tpl-588-execute-another-workflow.json | OK | 2 | deterministic | RUN | 960ee5c0e188 | 19d393b5a757 |
| tpl-6-sync-data-between-multiple-google-spreadsheets.json | OK | 4 | replaceable | SKIP | 8423b918ecde | b5bd250b419c |
| tpl-602-manage-users-automatically-in-reqres-in.json | OK | 4 | deterministic | RUN | 8b4a893c5431 | 1509f0bcdd5d |
| tpl-655-merge-greetings-with-the-users-based-on-the-language.json | OK | 4 | deterministic | RUN | 99a4ba227724 | 26f5ddc56ad8 |
| tpl-688-execute-set-node-based-on-function-output.json | OK | 7 | deterministic | RUN | 2097040c03a6 | 5b1a5f79f8d2 |
| tpl-693-display-project-data-on-a-smashing-dashboard.json | OK | 24 | replaceable | SKIP | 6a3da9f5bb9e | c13f7ac1a749 |
| tpl-871-n8n-espa-ol-tratamiento-de-textos.json | OK | 7 | deterministic | RUN | d54ab373ea26 | 26b26ec1947c |

Ringkasan: 15 file | OK 15 | Error 0 | RUN 7 (526,581,588,602,655,688,871) | REPLACEABLE 5 (119,122,226,6,693) | SKIP 3 (2221,27,4)
Catatan koreksi: angka "10 RUN" di pesan #277 keliru - tabel di atas adalah sumber kebenaran; klasifikasi IDENTIK dgn driver agent1 v0.3.
