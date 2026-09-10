# MANIFEST CORPUS WORKFLOW N8N — batch agent4 (2026-09-09 04:12)

Sumber: repo resmi https://github.com/n8n-io/n8n @ master 3afdf4a0f8a46fe4087ed463b2eda2e888beabbe
Prefix: awb = ai-workflow-builder.ee/evaluations/fixtures/reference-workflows | iai = instance-ai/evaluations/computer-use/fixtures

| File | Node | Tipe node (unik) | SHA-256 |
|---|---|---|---|
| awb-ai-news-digest.json | 8 | @n8n/n8n-nodes-langchain.openAi,n8n-nodes-base.code,n8n-nodes-base.httpRequest,n8n-nodes-base.scheduleTrigger,n8n-nodes- | f33e05cee847d9ade4c017b68b5d22577b36f53793c810169b6e469bad1013e8 |
| awb-daily-weather-report.json | 5 | @n8n/n8n-nodes-langchain.openAi,n8n-nodes-base.gmail,n8n-nodes-base.openWeatherMap,n8n-nodes-base.scheduleTrigger,n8n-no | c74b2e23595ad4fffea92fcc1b0fc4dba1b4ee05da3a706b1bb46e607e4192b6 |
| awb-email-summary.json | 9 | @n8n/n8n-nodes-langchain.agent,@n8n/n8n-nodes-langchain.lmChatOpenAi,@n8n/n8n-nodes-langchain.outputParserStructured,n8n | 461e0cf570152ba929a952872b331bf8d5d1065a304b39ddbb0f458356e98dec |
| awb-extract-from-file.json | 9 | n8n-nodes-base.code,n8n-nodes-base.dataTable,n8n-nodes-base.extractFromFile,n8n-nodes-base.formTrigger,n8n-nodes-base.gm | e45c657524e133c3e4b05616d8c2c9a8369c978cdfa2afa678dcb6d0199d7a54 |
| awb-google-sheets-processing.json | 10 | n8n-nodes-base.googleSheets,n8n-nodes-base.httpRequest,n8n-nodes-base.if,n8n-nodes-base.scheduleTrigger,n8n-nodes-base.s | 11f55fca4f8ad0b6d842ae9b73a1608ea0581efc9006e864cf3fe36b85421d0b |
| awb-invoice-pipeline.json | 15 | @n8n/n8n-nodes-langchain.openAi,n8n-nodes-base.code,n8n-nodes-base.dataTable,n8n-nodes-base.extractFromFile,n8n-nodes-ba | 4c1549957a68baeadcc9868d831b869c46ed269ecd3f28d04fee925d15ae13bc |
| awb-lead-qualification.json | 9 | @n8n/n8n-nodes-langchain.agent,@n8n/n8n-nodes-langchain.lmChatAnthropic,@n8n/n8n-nodes-langchain.outputParserStructured, | a327764102793fbf3a23a24f96deabfeedaeca5ec259a3e2fce1b76c498b5dbf |
| awb-multi-agent-research.json | 18 | @n8n/n8n-nodes-langchain.agent,@n8n/n8n-nodes-langchain.agentTool,@n8n/n8n-nodes-langchain.lmChatOpenAi,@n8n/n8n-nodes-l | 9865c5a55e109d6312e667d695995463a6cc884cf45f28f3d216f3240a5f9cee |
| awb-rag-assistant.json | 17 | @n8n/n8n-nodes-langchain.agent,@n8n/n8n-nodes-langchain.chatTrigger,@n8n/n8n-nodes-langchain.documentDefaultDataLoader,@ | ed4514d709caa06d781f11fdd7da4d5d2a253c42ca830fbab549c28c3e3d07ba |
| awb-youtube-auto-chapters.json | 12 | @n8n/n8n-nodes-langchain.agent,@n8n/n8n-nodes-langchain.chatTrigger,@n8n/n8n-nodes-langchain.lmChatAnthropic,@n8n/n8n-no | 8710d5095dce7d83ef78876ef088511f14b0d170d2691945296f35fe5817669c |
| iai-form-trigger-workflow.json | 2 | n8n-nodes-base.formTrigger,n8n-nodes-base.set | 50eb87416dc18391ed84b5efbed8b57b3e2fe29af3e2cd1562fd125172397741 |
| iai-sample-workflow.json | 3 | n8n-nodes-base.httpRequest,n8n-nodes-base.scheduleTrigger,n8n-nodes-base.slack | a4183e0cb1169d16cbddbe043d045a68b62ab71050947b5c7f37f2f82b22b484 |

Catatan verifikasi: semua file lolos parse JSON dan memiliki kunci name+nodes+connections.
6 file instance-ai/evaluations/data/workflows DITOLAK: format evaluasi AI (bukan workflow JSON mentah).
