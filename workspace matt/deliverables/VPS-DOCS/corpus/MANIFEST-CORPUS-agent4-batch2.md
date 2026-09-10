# MANIFEST CORPUS agent4 — BATCH 2: 56 TEMPLATE PUBLIK N8N.IO (REAL)

Tanggal: 2026-09-09 | Pengumpul: agent4 (Backend Engineer)
Sumber: API template publik n8n.io (endpoint detail https://api.n8n.io/api/workflows/<id>), disimpan APA ADANYA tanpa diedit.
Metode: probe id 1..700 pada endpoint detail; tiap file = respons asli endpoint.
Provenansi per file: template id + URL halaman + sha256 (kolom di bawah).
Kualifikasi: det_ratio >= 0.7 (>=70% node deterministik) ideal untuk differential EXECUTION test tanpa side-effect eksternal;
sisanya tetap valid untuk uji L1 (impor/parse) dan katalog deviasi.

| File | Template ID | URL | Nama | Node | Det-ratio | SHA-256 |
|---|---|---|---|---|---|---|
| tpl-3-write-http-query-string-on-image.json | 3 | https://n8n.io/workflows/3 | Write HTTP query string on image | 3 | 1.0 | dad0a8c9303f357a445c36f65687bac37abc6bbe3bc26c800be151dfe1fe19dc |
| tpl-435-create-a-new-digitalocean-droplet.json | 435 | https://n8n.io/workflows/435 | Create a new DigitalOcean droplet | 1 | 1.0 | a064834e5d434bb0e0342ec8d815531f503f239fd8d0920b934cbd1afd1a02bc |
| tpl-437-perform-speech-to-text-on-recorded-audio-clips-using-wit-ai.json | 437 | https://n8n.io/workflows/437 | Perform speech-to-text on recorded audio clips using Wi | 2 | 1.0 | 1926a6a9238d2e8d957dc264df7b447f234daf1e71724eabbf122caa1450f802 |
| tpl-526-assign-values-to-variables-using-the-set-node.json | 526 | https://n8n.io/workflows/526 | Assign values to variables using the Set node | 2 | 1.0 | dee89f94cf3b90fcce14effc97bbfa9afc6b91a14dcfd1e436b461dc6fdf7683 |
| tpl-574-encrypt-some-data-using-the-crypto-node.json | 574 | https://n8n.io/workflows/574 | Encrypt some data using the crypto node | 2 | 1.0 | a86fc5c16a9e2a274ad0d477313a6976ca862fc5a3bf4324bb19bf170eed3518 |
| tpl-575-convert-a-date-from-one-format-to-another.json | 575 | https://n8n.io/workflows/575 | Convert a date from one format to another | 2 | 1.0 | 0e49341d4d2803f08bad62a65382267d04523d15ad278969351e2eeaa4aadf37 |
| tpl-577-read-a-file-from-disk.json | 577 | https://n8n.io/workflows/577 | Read a file from disk | 2 | 1.0 | 862ec561e78a023430966d982708081b779ad3826a802f62c4fb5a107dc666d9 |
| tpl-583-read-an-rss-feed.json | 583 | https://n8n.io/workflows/583 | Read an RSS feed | 2 | 1.0 | f49f1c82a23fa69bc6b4f70e9c004641c0e3d103980143153462a37414789243 |
| tpl-588-execute-another-workflow.json | 588 | https://n8n.io/workflows/588 | Execute another workflow | 2 | 1.0 | 61a8ac0e1431a883993ccf543fc52ba973d0ee7e07ffc1cbf2a4ea1f9d8bc5cd |
| tpl-576-get-information-of-an-image.json | 576 | https://n8n.io/workflows/576 | Get information of an image | 3 | 1.0 | b0761e4953b1a58f0e39ed0159bb798dfeae71ab784142243af34aa86b726d24 |
| tpl-582-rename-a-key-in-n8n.json | 582 | https://n8n.io/workflows/582 | Rename a key in n8n | 3 | 1.0 | 8c747887240ef837175fb245f4691b2329bb84879b5398aa48c106595ede7043 |
| tpl-586-read-a-spreadsheet-file.json | 586 | https://n8n.io/workflows/586 | Read a spreadsheet file | 3 | 1.0 | ee31f99fcb860b43055351d59c6688d83fdbc240d9aa65209a5bd891d5df0476 |
| tpl-590-write-a-file-to-the-host-machine.json | 590 | https://n8n.io/workflows/590 | Write a file to the host machine | 3 | 1.0 | 59696073097e6dd6cef8207293dec7258142ecf1a56177d80ba4aeac3ee43889 |
| tpl-591-add-text-to-a-downloaded-image.json | 591 | https://n8n.io/workflows/591 | Add text to a downloaded image | 3 | 1.0 | 645edb29f6c41bbfbb3a7ac8b4de36c2dde542d12f8301aa0d8dcea9d0118d32 |
| tpl-602-manage-users-automatically-in-reqres-in.json | 602 | https://n8n.io/workflows/602 | Manage users automatically in reqres.in | 4 | 1.0 | 4de64466f945cde97084b61039d7ae54ef65f6dce66b93131caba1b081b1c013 |
| tpl-655-merge-greetings-with-the-users-based-on-the-language.json | 655 | https://n8n.io/workflows/655 | Merge greetings with the users based on the language | 4 | 1.0 | 031030b54e638ed667bdf424ff3784484934aa94cfd9552bb37420a926d49e7e |
| tpl-663-download-a-file-and-upload-it-to-an-ftp-server.json | 663 | https://n8n.io/workflows/663 | Download a file and upload it to an FTP Server | 4 | 1.0 | ef329054d46f7f88faac9a37cca8723876857149f327eded526a6494efd77583 |
| tpl-688-execute-set-node-based-on-function-output.json | 688 | https://n8n.io/workflows/688 | Execute Set node based on Function output | 7 | 0.86 | 17ae8379c6683af71ce6ac5a667f5398abe4f301b597cc4c233feff26a572fca |
| tpl-378-tiny-tiny-rss-aka-tt-rss-feed-to-mastodon.json | 378 | https://n8n.io/workflows/378 | Tiny tiny RSS (aka tt-rss) feed to Mastodon | 6 | 0.83 | 781ccd3f6321b222c527e6d8b524de19cc754f25567aa7156d6974aef21331d9 |
| tpl-581-execute-set-node-based-on-function-output.json | 581 | https://n8n.io/workflows/581 | Execute Set node based on Function output | 5 | 0.8 | 4e1a4fadae6e10c48abb5882f6712b5bfacde01e41ad80d19f0187aa5159dded |
| tpl-693-display-project-data-on-a-smashing-dashboard.json | 693 | https://n8n.io/workflows/693 | Display project data on a smashing dashboard | 24 | 0.79 | 5c6a003ef7a85b77c0fb37e1afccd27d46699a76ac3c7a297ee6a597e68d8f07 |
| tpl-119-webhook-returning-xml.json | 119 | https://n8n.io/workflows/119 | Webhook returning XML | 4 | 0.75 | 9e7a81bea323588c1a1794416803530fa911ff2216a24a567324bbf984e60fae |
| tpl-635-export-wordpress-posts-to-spreadsheet.json | 635 | https://n8n.io/workflows/635 | Export WordPress posts to spreadsheet | 4 | 0.75 | c74c047a2edcbb76f89276a4392ebe4a38b0e8438e1c5685e0ddd2730274a49c |
| tpl-652-store-data-received-from-webhook-in-json.json | 652 | https://n8n.io/workflows/652 | Store data received from webhook in JSON | 4 | 0.75 | b740abdf501ad1737d757bf687c84589208551dbe6e90adb8d7d98c3dfe269b2 |
| tpl-687-read-rss-feed-from-two-different-sources.json | 687 | https://n8n.io/workflows/687 | Read RSS feed from two different sources | 4 | 0.75 | 2bcdf4bfe08f310455624f2ec9adafbe4e4c72b9684163d469e777fd09ab07dd |
| tpl-1-insert-excel-data-to-postgres.json | 1 | https://n8n.io/workflows/1 | Insert Excel data to Postgres | 3 | 0.67 | e6cb4a6aad1965ad093d442da48501edc46a98e0e5ea0c14433dd227ba47c74d |
| tpl-2-transfer-data-from-postgres-to-excel.json | 2 | https://n8n.io/workflows/2 | Transfer data from Postgres to Excel | 3 | 0.67 | 7a79b5414938baee89626baa50ddf381e772c3752c926ca5f691bb685e6497de |
| tpl-19-n8n-nodemation-basic-creating-your-first-simple-workflow-2-3.json | 19 | https://n8n.io/workflows/19 | n8n Nodemation basic - creating your first simple workf | 3 | 0.67 | 86f44bcd02936707491908e03d3614636fe03a121fac567f90d005aec66c3934 |
| tpl-160-convert-xml-to-json.json | 160 | https://n8n.io/workflows/160 | Convert XML to JSON | 3 | 0.67 | 651c6404de48827e521a17cb065ccd4b15245d12053eea21f180935df25832c9 |
| tpl-159-send-rss-feed-data-to-webhook.json | 159 | https://n8n.io/workflows/159 | Send RSS feed data to webhook | 18 | 0.67 | 8900e6d6a4ea7e02890b7b84b61133e95e688a669e7b95123cf8cc28498c98e9 |
| tpl-342-send-daily-affirmations-to-telegram.json | 342 | https://n8n.io/workflows/342 | Send daily affirmations to Telegram | 3 | 0.67 | b0e468301847e6030dd54abd578dbb4fe8554fc8802ab6889a2d7852cd798d73 |
| tpl-351-webhooks-with-mattermost.json | 351 | https://n8n.io/workflows/351 | Webhooks with Mattermost | 3 | 0.67 | bae189546c197a5664034dde9376bce9a82ad3d0438fdbcb6887f0fac3bc1c7c |
| tpl-418-cross-post-your-blog-posts.json | 418 | https://n8n.io/workflows/418 | Cross-post your blog posts | 3 | 0.67 | 6e645a8c91cc596aeebef2937ff91e60aa69a7d64315ba6ea603a5097515ef88 |
| tpl-482-insert-data-into-a-new-row-for-a-table-in-coda.json | 482 | https://n8n.io/workflows/482 | Insert data into a new row for a table in Coda | 3 | 0.67 | 3fb0152ada8edb18bc38eb9c80185bb99f484a11dee16faea17dca954ec1c289 |
| tpl-503-insert-a-document-in-mongodb.json | 503 | https://n8n.io/workflows/503 | Insert a document in MongoDB | 3 | 0.67 | 2a0598b05cb69089a891107facd211544401fdcfda30b2c76ce9526cfa4f3896 |
| tpl-515-download-a-file-from-google-drive.json | 515 | https://n8n.io/workflows/515 | Download a file from Google Drive | 3 | 0.67 | 84c72e060e9f9fba84950519aea19fd0f4bbdd0a0f211e9c6a3e52dea4d47ecf |
| tpl-585-extract-text-from-a-pdf-file.json | 585 | https://n8n.io/workflows/585 | Extract text from a PDF file | 3 | 0.67 | a8455684d35d3b79764f05bc8911235d32892a7442307a69d4481c4d33338cfe |
| tpl-175-backs-up-n8n-workflows-to-nextcloud.json | 175 | https://n8n.io/workflows/175 | Backs up n8n workflows to NextCloud | 9 | 0.56 | f996d34559f350b689494ed3825d6c4ddf8aca7f250e1cce34757c9be5a4fe6d |
| tpl-8-handle-errors-from-a-different-workflow.json | 8 | https://n8n.io/workflows/8 | Handle errors from a different workflow | 2 | 0.5 | c474f015ce9fdc3b5cc1475e8fd5dd0b2140d5afe28188adf6449b997ea6bb3b |
| tpl-226-receive-google-sheet-data-via-rest-api.json | 226 | https://n8n.io/workflows/226 | Receive Google Sheet data via REST API | 2 | 0.5 | b864db03a43ff34b5f37f3effa39ea50648f5837dc1cd4d2b2c561e6a5bd25bb |
| tpl-216-api-queries-data-from-graphql.json | 216 | https://n8n.io/workflows/216 | API queries data from GraphQL | 4 | 0.5 | 82e1686a301df5d67f381cda646a84c016a597bbcb05a8eb1492f509bd38cb24 |
| tpl-179-convert-typeform-data-into-spreadsheet.json | 179 | https://n8n.io/workflows/179 | Convert Typeform data into spreadsheet | 6 | 0.5 | 7ba216506ffaae6f200c3d457b8cc899f7caaad2f319c64da147b9a71083f0ef |
| tpl-122-report-phishing-websites-to-steam-and-cloudflare.json | 122 | https://n8n.io/workflows/122 | Report phishing websites to Steam and CloudFlare | 9 | 0.44 | ed9366ed36b0beb4822ee2b2b1340e1c802b38c527e4352cc9b42844f101bd4c |
| tpl-199-rss-telegram-bot.json | 199 | https://n8n.io/workflows/199 | RSS Telegram bot | 18 | 0.44 | 4ecccad3534ddc4661423882b03bcf31a7badc9f0d5e15f6c85cd3015c14ccbd |
| tpl-225-send-trending-show-hn-to-email.json | 225 | https://n8n.io/workflows/225 | Send trending "Show HN" to email | 7 | 0.43 | 434c801faac61633ba0b9d7add356b4c4e6ba0ee93a5df184653ba73fcf06d7e |
| tpl-13-transform-xml-data-and-upload-to-dropbox.json | 13 | https://n8n.io/workflows/13 | Transform XML data and upload to Dropbox | 5 | 0.4 | 5f2d103555946de48a812599c43503259ff2ba9e70a4a51f8d838a3f2dceb58e |
| tpl-154-listen-on-new-emails-on-a-imap-mailbox.json | 154 | https://n8n.io/workflows/154 | Listen on new emails on a IMAP mailbox | 5 | 0.4 | ef208c68a5be013904c17d55526b5f5e950bc62bf2e3fc859394ceb3b5644c21 |
| tpl-18-n8n-nodemation-basic-getting-started-on-the-workflow-canvas-.json | 18 | https://n8n.io/workflows/18 | n8n nodemation basic - getting started on the workflow  | 3 | 0.33 | 0dbbe9fe86278db8278023dedb42187f87d1d6bb1434cdbb0978d07c82986b3b |
| tpl-101-write-json-to-disk-binary.json | 101 | https://n8n.io/workflows/101 | Write JSON to disk (binary) | 3 | 0.33 | b35af7cf1622165fce007508aaae7a244e5f5a6519db54699de555eba6b4b0e6 |
| tpl-156-get-execute-command-data-and-transfer-to-json.json | 156 | https://n8n.io/workflows/156 | Get execute command data and transfer to JSON | 3 | 0.33 | e8e43bb73a1e3298a6a316952b8f4a46ed695dbf7bacd3725fb434b85ff9231e |
| tpl-4-send-selected-github-events-to-slack.json | 4 | https://n8n.io/workflows/4 | Send selected GitHub events to Slack | 4 | 0.25 | 459b177fde9b11f3b14c3ff36a0094d811db65f3aa7ff3e7170209bb734066ea |
| tpl-6-sync-data-between-multiple-google-spreadsheets.json | 6 | https://n8n.io/workflows/6 | Sync data between multiple Google Spreadsheets | 4 | 0.25 | 0424e1493b0e84530de990d878f8e63f07b866e00451c87830b5c8e7fee33e1d |
| tpl-11-add-data-from-google-sheet-to-dropbox.json | 11 | https://n8n.io/workflows/11 | Add data from Google Sheet to Dropbox | 4 | 0.25 | 4a7c081a8d12433c02f4e4ed733708aadb6338c367aea08ee31dd758fc9eb0af |
| tpl-27-telegram-sticker-bot.json | 27 | https://n8n.io/workflows/27 | Telegram sticker bot | 4 | 0.25 | c460bb620c6fb2e43a0995efabc0c70cdd29863a7c5c0f59d0d6fe78f32c8b9f |
| tpl-29-send-typeform-results-to-google-sheet-slack-and-email.json | 29 | https://n8n.io/workflows/29 | Send Typeform results to Google Sheet, Slack and email | 5 | 0.2 | 9ae58d67c55e0807fd6e3439d7be25687ba28c9d695f33e243f8a6a37b991628 |
| tpl-100-using-the-merge-node-merge-by-key.json | 100 | https://n8n.io/workflows/100 | Using the merge node - merge by key | 5 | 0.2 | 011c9299f68d60c1f935739e5433aac806758fb291af11251e9e574d83341376 |

Ringkasan: 56 file | deterministik >= 0.7: 25 file
