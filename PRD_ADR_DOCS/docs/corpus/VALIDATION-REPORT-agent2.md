# LAPORAN VALIDASI KORPUS — agent2
**Tanggal:** 2026-09-09 04:30:15 UTC
**Script:** validate_corpus.py
**Total file:** 96

## RINGKASAN

| Klasifikasi | Jumlah | Arti |
|---|---|---|
| REAL_EXPORT | 1 | ✅ Ekspor n8n asli (valid untuk differential testing) |
| LIKELY_EXPORT | 56 | 🟡 Kemungkinan ekspor n8n (perlu verifikasi manual) |
| SYNTHETIC | 2 | 🔴 BUKAN ekspor n8n — ditulis tangan/digenerate |
| INVALID | 37 | ❌ Bukan workflow JSON valid |

## DETAIL PER FILE

| File | Status | Nodes | Export Fields | External API | Issues |
|---|---|---|---|---|---|
| tpl-1-insert-excel-data-to-postgres.json | ❌ INVALID | 3 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-100-using-the-merge-node-merge-by-key.json | ❌ INVALID | 5 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-10000-auto-create-tiktok-videos-with-veed-io-ai-avatars.json | 🟡 LIKELY_EXPORT | 35 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-101-write-json-to-disk-binary.json | ❌ INVALID | 3 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-10174-ppc-campaign-intelligence-optimization-with-google.json | 🟡 LIKELY_EXPORT | 23 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-10358-automate-ai-video-creation-multi-platform-publishi.json | 🟡 LIKELY_EXPORT | 25 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-10566-process-large-documents-with-ocr-using-subworkflow.json | 🟡 LIKELY_EXPORT | 16 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-10724-create-fluidx-the-eye-live-camera-sessions-with-sm.json | 🟡 LIKELY_EXPORT | 50 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-11-add-data-from-google-sheet-to-dropbox.json | ❌ INVALID | 4 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-11204-create-ai-viral-videos-using-nanobanana-2-pro-veo3.json | 🟡 LIKELY_EXPORT | 35 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-11572-aggregate-news-articles-from-newsapi-mediastack-cu.json | 🟡 LIKELY_EXPORT | 34 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-11807-answer-multi-channel-support-queries-with-openai-r.json | 🟡 LIKELY_EXPORT | 91 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-119-webhook-returning-xml.json | ❌ INVALID | 4 | 1/8 | tidak | Missing required fields: ['name'] |
| tpl-122-report-phishing-websites-to-steam-and-cloudflare.json | 🔴 SYNTHETIC | 9 | 2/8 | tidak | Hanya 2/8 field metadata ekspor n8n ditemukan |
| tpl-12325-create-track-linkedin-posts-with-google-sheets-gpt.json | 🟡 LIKELY_EXPORT | 33 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-12345-scrape-physician-profiles-from-browseract-into-goo.json | 🟡 LIKELY_EXPORT | 11 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-12462-create-ai-product-images-and-marketing-videos-with.json | 🟡 LIKELY_EXPORT | 76 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-13-transform-xml-data-and-upload-to-dropbox.json | ❌ INVALID | 5 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-13503-manage-woocommerce-store-operations-via-ai-telegra.json | 🟡 LIKELY_EXPORT | 16 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-13526-generate-ai-videos-and-carousels-with-blotato-for.json | 🟡 LIKELY_EXPORT | 13 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-14167-scrape-search-and-browse-the-web-with-a-firecrawl.json | 🟡 LIKELY_EXPORT | 21 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-15000-send-appointment-sms-follow-ups-with-typeform-twil.json | 🟡 LIKELY_EXPORT | 20 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-15369-triage-emails-and-draft-gmail-replies-using-gemini.json | 🟡 LIKELY_EXPORT | 25 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-154-listen-on-new-emails-on-a-imap-mailbox.json | 🟡 LIKELY_EXPORT | 5 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-15461-send-daily-ai-crypto-market-insights-with-google-g.json | 🟡 LIKELY_EXPORT | 18 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-156-get-execute-command-data-and-transfer-to-json.json | ❌ INVALID | 3 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-159-send-rss-feed-data-to-webhook.json | 🟡 LIKELY_EXPORT | 18 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-160-convert-xml-to-json.json | 🟡 LIKELY_EXPORT | 3 | 3/8 | tidak | - |
| tpl-16440-manage-google-calendar-events-with-zalo-bot-openai.json | 🟡 LIKELY_EXPORT | 20 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-16792-log-multi-platform-ad-spend-from-meta-google-tikto.json | 🟡 LIKELY_EXPORT | 32 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-17103-post-daily-linkedin-and-instagram-content-with-air.json | 🟡 LIKELY_EXPORT | 19 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-175-backs-up-n8n-workflows-to-nextcloud.json | 🟡 LIKELY_EXPORT | 9 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-179-convert-typeform-data-into-spreadsheet.json | ❌ INVALID | 6 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-18-n8n-nodemation-basic-getting-started-on-the-workflow-canvas-.json | 🟡 LIKELY_EXPORT | 3 | 3/8 | tidak | - |
| tpl-19-n8n-nodemation-basic-creating-your-first-simple-workflow-2-3.json | 🟡 LIKELY_EXPORT | 3 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-199-rss-telegram-bot.json | 🟡 LIKELY_EXPORT | 18 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-2-transfer-data-from-postgres-to-excel.json | ❌ INVALID | 3 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-216-api-queries-data-from-graphql.json | ❌ INVALID | 4 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-225-send-trending-show-hn-to-email.json | ❌ INVALID | 7 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-226-receive-google-sheet-data-via-rest-api.json | ❌ INVALID | 2 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-2519-fetch-live-etf-metrics-from-justetf-to-excel-with.json | 🟡 LIKELY_EXPORT | 14 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-27-telegram-sticker-bot.json | ❌ INVALID | 4 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-29-send-typeform-results-to-google-sheet-slack-and-email.json | ❌ INVALID | 5 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-3-write-http-query-string-on-image.json | ❌ INVALID | 3 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-342-send-daily-affirmations-to-telegram.json | 🟡 LIKELY_EXPORT | 3 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-351-webhooks-with-mattermost.json | 🟡 LIKELY_EXPORT | 3 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-378-tiny-tiny-rss-aka-tt-rss-feed-to-mastodon.json | 🟡 LIKELY_EXPORT | 6 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-4-send-selected-github-events-to-slack.json | ❌ INVALID | 4 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-4110-clone-viral-tiktoks-with-ai-avatars-auto-post-to-9.json | 🟡 LIKELY_EXPORT | 41 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-418-cross-post-your-blog-posts.json | ❌ INVALID | 3 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-435-create-a-new-digitalocean-droplet.json | ❌ INVALID | 1 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-437-perform-speech-to-text-on-recorded-audio-clips-using-wit-ai.json | ❌ INVALID | 2 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-4722-gmail-ai-email-manager.json | 🟡 LIKELY_EXPORT | 8 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-4751-translate-and-distribute-dubbed-videos-with-dublab.json | 🟡 LIKELY_EXPORT | 18 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-482-insert-data-into-a-new-row-for-a-table-in-coda.json | 🟡 LIKELY_EXPORT | 3 | 3/8 | tidak | - |
| tpl-4827-ai-powered-whatsapp-chatbot-for-text-voice-images.json | 🟡 LIKELY_EXPORT | 35 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-4846-generate-ai-videos-with-google-veo3-save-to-google.json | 🟡 LIKELY_EXPORT | 23 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-4966-customer-support-whatsapp-bot-with-google-docs-kno.json | 🟡 LIKELY_EXPORT | 14 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-4968-automated-linkedin-content-creation-with-gpt-4-and.json | 🟡 LIKELY_EXPORT | 13 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-5010-rag-starter-template-using-simple-vector-stores-fo.json | 🟡 LIKELY_EXPORT | 12 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-503-insert-a-document-in-mongodb.json | ❌ INVALID | 3 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-5035-generate-auto-post-ai-videos-to-social-media-with.json | 🟡 LIKELY_EXPORT | 29 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-5110-create-upload-ai-generated-asmr-youtube-shorts-wit.json | 🟡 LIKELY_EXPORT | 32 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-5148-local-chatbot-with-retrieval-augmented-generation.json | 🟡 LIKELY_EXPORT | 13 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-515-download-a-file-from-google-drive.json | ❌ INVALID | 3 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-5170-learn-json-basics-with-an-interactive-step-by-ste.json | 🟡 LIKELY_EXPORT | 24 | 3/8 | tidak | - |
| tpl-526-assign-values-to-variables-using-the-set-node.json | 🟡 LIKELY_EXPORT | 2 | 3/8 | tidak | - |
| tpl-5271-learn-n8n-expressions-with-an-interactive-step-by.json | 🟡 LIKELY_EXPORT | 27 | 3/8 | tidak | - |
| tpl-5338-generate-ai-viral-videos-with-seedance-and-upload.json | 🟡 LIKELY_EXPORT | 40 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-5677-extract-transform-hackernews-data-to-google-docs-u.json | 🟡 LIKELY_EXPORT | 16 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-5678-automate-email-filtering-ai-summarization-100-free.json | 🟡 LIKELY_EXPORT | 14 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-5691-generate-personalized-sales-emails-with-linkedin-d.json | 🟡 LIKELY_EXPORT | 23 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-574-encrypt-some-data-using-the-crypto-node.json | ❌ INVALID | 2 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-575-convert-a-date-from-one-format-to-another.json | ❌ INVALID | 2 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-5755-transform-old-photos-into-animated-videos-with-flu.json | 🟡 LIKELY_EXPORT | 19 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-576-get-information-of-an-image.json | ❌ INVALID | 3 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-577-read-a-file-from-disk.json | ❌ INVALID | 2 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-581-execute-set-node-based-on-function-output.json | ❌ INVALID | 5 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-5819-build-an-interactive-ai-agent-with-chat-interface.json | 🟡 LIKELY_EXPORT | 17 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-582-rename-a-key-in-n8n.json | ❌ INVALID | 3 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-583-read-an-rss-feed.json | ❌ INVALID | 2 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-585-extract-text-from-a-pdf-file.json | ❌ INVALID | 3 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-586-read-a-spreadsheet-file.json | ❌ INVALID | 3 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-588-execute-another-workflow.json | ❌ INVALID | 2 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-590-write-a-file-to-the-host-machine.json | 🟡 LIKELY_EXPORT | 3 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-591-add-text-to-a-downloaded-image.json | 🟡 LIKELY_EXPORT | 3 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-6-sync-data-between-multiple-google-spreadsheets.json | ❌ INVALID | 4 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-602-manage-users-automatically-in-reqres-in.json | ❌ INVALID | 4 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-635-export-wordpress-posts-to-spreadsheet.json | 🟡 LIKELY_EXPORT | 4 | 3/8 | tidak | - |
| tpl-652-store-data-received-from-webhook-in-json.json | 🔴 SYNTHETIC | 4 | 2/8 | YA | Hanya 2/8 field metadata ekspor n8n ditemukan; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-655-merge-greetings-with-the-users-based-on-the-language.json | ❌ INVALID | 4 | 2/8 | tidak | Missing required fields: ['name'] |
| tpl-663-download-a-file-and-upload-it-to-an-ftp-server.json | ❌ INVALID | 4 | 0/8 | YA | Missing required fields: ['name']; Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-687-read-rss-feed-from-two-different-sources.json | ✅ REAL_EXPORT | 4 | 6/8 | tidak | - |
| tpl-688-execute-set-node-based-on-function-output.json | ❌ INVALID | 7 | 0/8 | tidak | Missing required fields: ['name'] |
| tpl-693-display-project-data-on-a-smashing-dashboard.json | 🟡 LIKELY_EXPORT | 24 | 3/8 | YA | Mengandung node eksternal (butuh API calls nyata / mock) |
| tpl-8-handle-errors-from-a-different-workflow.json | ❌ INVALID | 2 | 0/8 | tidak | Missing required fields: ['name'] |

## SEMUA NODE TYPES DITEMUKAN

- `@blotato/n8n-nodes-blotato.blotato`
- `@blotato/n8n-nodes-blotato.blotatoTool`
- `@bufferapp/n8n-nodes-buffer.buffer`
- `@mendable/n8n-nodes-firecrawl.firecrawlTool`
- `@n8n/n8n-nodes-langchain.agent`
- `@n8n/n8n-nodes-langchain.chainLlm`
- `@n8n/n8n-nodes-langchain.chatTrigger`
- `@n8n/n8n-nodes-langchain.documentDefaultDataLoader`
- `@n8n/n8n-nodes-langchain.embeddingsOllama`
- `@n8n/n8n-nodes-langchain.embeddingsOpenAi`
- `@n8n/n8n-nodes-langchain.googleGemini`
- `@n8n/n8n-nodes-langchain.lmChatAnthropic`
- `@n8n/n8n-nodes-langchain.lmChatGoogleGemini`
- `@n8n/n8n-nodes-langchain.lmChatGroq`
- `@n8n/n8n-nodes-langchain.lmChatOllama`
- `@n8n/n8n-nodes-langchain.lmChatOpenAi`
- `@n8n/n8n-nodes-langchain.lmChatOpenRouter`
- `@n8n/n8n-nodes-langchain.memoryBufferWindow`
- `@n8n/n8n-nodes-langchain.memoryPostgresChat`
- `@n8n/n8n-nodes-langchain.openAi`
- `@n8n/n8n-nodes-langchain.outputParserStructured`
- `@n8n/n8n-nodes-langchain.textSplitterRecursiveCharacterTextSplitter`
- `@n8n/n8n-nodes-langchain.toolCode`
- `@n8n/n8n-nodes-langchain.toolThink`
- `@n8n/n8n-nodes-langchain.toolWikipedia`
- `@n8n/n8n-nodes-langchain.vectorStoreInMemory`
- `@n8n/n8n-nodes-langchain.vectorStoreMongoDBAtlas`
- `@n8n/n8n-nodes-langchain.vectorStoreQdrant`
- `@n8n/n8n-nodes-langchain.vectorStoreSupabase`
- `n8n-nodes-base.aggregate`
- `n8n-nodes-base.aiTransform`
- `n8n-nodes-base.airtable`
- `n8n-nodes-base.box`
- `n8n-nodes-base.coda`
- `n8n-nodes-base.code`
- `n8n-nodes-base.convertToFile`
- `n8n-nodes-base.cron`
- `n8n-nodes-base.crypto`
- `n8n-nodes-base.cryptoTool`
- `n8n-nodes-base.dateTime`
- `n8n-nodes-base.dateTimeTool`
- `n8n-nodes-base.discord`
- `n8n-nodes-base.dropbox`
- `n8n-nodes-base.editImage`
- `n8n-nodes-base.emailReadImap`
- `n8n-nodes-base.emailSend`
- `n8n-nodes-base.errorTrigger`
- `n8n-nodes-base.executeCommand`
- `n8n-nodes-base.executeWorkflow`
- `n8n-nodes-base.extractFromFile`
- `n8n-nodes-base.filter`
- `n8n-nodes-base.formTrigger`
- `n8n-nodes-base.ftp`
- `n8n-nodes-base.function`
- `n8n-nodes-base.functionItem`
- `n8n-nodes-base.github`
- `n8n-nodes-base.githubTrigger`
- `n8n-nodes-base.gmail`
- `n8n-nodes-base.gmailTool`
- `n8n-nodes-base.gmailTrigger`
- `n8n-nodes-base.googleAds`
- `n8n-nodes-base.googleCalendarTool`
- `n8n-nodes-base.googleDocs`
- `n8n-nodes-base.googleDrive`
- `n8n-nodes-base.googleSheets`
- `n8n-nodes-base.googleSheetsTool`
- `n8n-nodes-base.googleSheetsTrigger`
- `n8n-nodes-base.graphql`
- `n8n-nodes-base.hackerNews`
- `n8n-nodes-base.html`
- `n8n-nodes-base.htmlExtract`
- `n8n-nodes-base.httpRequest`
- `n8n-nodes-base.httpRequestTool`
- `n8n-nodes-base.if`
- `n8n-nodes-base.interval`
- `n8n-nodes-base.itemLists`
- `n8n-nodes-base.limit`
- `n8n-nodes-base.linkedIn`
- `n8n-nodes-base.mailgun`
- `n8n-nodes-base.manualTrigger`
- `n8n-nodes-base.mattermost`
- `n8n-nodes-base.medium`
- `n8n-nodes-base.merge`
- `n8n-nodes-base.microsoftExcel`
- `n8n-nodes-base.mongoDb`
- `n8n-nodes-base.moveBinaryData`
- `n8n-nodes-base.nextCloud`
- `n8n-nodes-base.noOp`
- `n8n-nodes-base.nocoDb`
- `n8n-nodes-base.perplexity`
- `n8n-nodes-base.postgres`
- `n8n-nodes-base.readBinaryFile`
- `n8n-nodes-base.readPDF`
- `n8n-nodes-base.removeDuplicates`
- `n8n-nodes-base.renameKeys`
- `n8n-nodes-base.respondToWebhook`
- `n8n-nodes-base.rssFeedRead`
- `n8n-nodes-base.rssFeedReadTool`
- `n8n-nodes-base.scheduleTrigger`
- `n8n-nodes-base.set`
- `n8n-nodes-base.slack`
- `n8n-nodes-base.slackTrigger`
- `n8n-nodes-base.splitInBatches`
- `n8n-nodes-base.splitOut`
- `n8n-nodes-base.spreadsheetFile`
- `n8n-nodes-base.stickyNote`
- `n8n-nodes-base.supabase`
- `n8n-nodes-base.switch`
- `n8n-nodes-base.telegram`
- `n8n-nodes-base.telegramTool`
- `n8n-nodes-base.telegramTrigger`
- `n8n-nodes-base.twilio`
- `n8n-nodes-base.twilioTrigger`
- `n8n-nodes-base.typeformTrigger`
- `n8n-nodes-base.wait`
- `n8n-nodes-base.webhook`
- `n8n-nodes-base.whatsApp`
- `n8n-nodes-base.whatsAppTrigger`
- `n8n-nodes-base.wooCommerceTool`
- `n8n-nodes-base.wordpress`
- `n8n-nodes-base.writeBinaryFile`
- `n8n-nodes-base.xml`
- `n8n-nodes-base.youTube`
- `n8n-nodes-base.zendesk`
- `n8n-nodes-browseract.browserAct`
- `n8n-nodes-upload-post.uploadPost`
- `n8n-nodes-zalo-bot-official.zaloBot`
- `n8n-nodes-zalo-bot-official.zaloBotTrigger`

## REKOMENDASI

1. Hanya **1/96** file yang benar-benar ekspor n8n asli.
2. Untuk differential testing (PRD-2 §10.2), kita butuh **minimal 50 REAL_EXPORT**.
3. File SYNTHETIC masih berguna untuk test parser (L1 = format), TIDAK untuk L2/L3.
4. File dengan external nodes butuh **mock server** sebelum bisa dijalankan deterministik.
5. **Sumber korpus nyata yang disarankan:**
   - Template publik n8n: https://n8n.io/workflows/ (ekspor manual)
   - Repo komunitas n8n (search: n8n workflow github)
   - Buat workflow sederhana di n8n instance lokal lalu ekspor
