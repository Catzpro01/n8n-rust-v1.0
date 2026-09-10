//! R-6 kindmap — pemetaan tipe n8n → NodeKind kernel + version u16.
//!
//! Konvensi (keputusan provisional #tech-debate #1176, agent1; override
//! terbuka @matt/@fern): Opsi-A tabel dot-case EKSPLISIT + fail-loud
//! unmapped; `splitInBatches` utuh (tidak dipecah). Kernel examples
//! (node.rs): `http.request`, `if`, `merge`, `generated.stripe.charges`.
//!
//! Status SEMUA baris: RATIFIED — 2026-09-09, agent1 #1241
//! (RATIFIED-WITH-6-ROW-CORRECTIONS) + matt RULING 35 (#1249) mengesahkan
//! 6 koreksi + KEEP splitInBatches. Tabel digenerate dari korpus 171
//! fixture: derivasi = strip prefix `n8n-nodes-base.` → camel-split +
//! lowercase dot-join; exceptions terkurai utk brand/akronim.
//!
//! # Kelas penamaan (prinsip RULING 35: TITIK = HIERARKI, BUKAN PEMISAH KATA)
//! 1. vendor.product — hierarki nyata: `google.sheets`, `aws.s3`,
//!    `aws.rekognition`, `microsoft.excel`, `http.request` (contoh node.rs).
//! 2. brand token-tunggal (satu merek, digabung): `mongodb`, `mailerlite`,
//!    `sendinblue`, `nextcloud`, `nocodb`, `woocommerce`, `linkedin`,
//!    `whatsapp`, `youtube`, `sendgrid`, `highlevel`, `noop`, `n8n`.
//! 3. split deskriptif yang DIRATIFIKASI KEEP (agent1 #1241): `hacker.news`,
//!    `one.simple.api`, `ai.transform`, `date.time` (+ `*Tool`).
//! 4. node inti n8n — nama upstream dipertahankan apa adanya (RULING 35):
//!    `splitInBatches` → `splitInBatches`. ANGGOTA KELAS, bukan pengecualian
//!    tunggal: node inti camelCase lain yang muncul kemudian masuk kelas ini
//!    tanpa keputusan baru.
//!
//! Tipe di luar keluarga base (langchain/komunitas) TIDAK dipetakan di
//! sini — kategori opaque (R-2); pemanggil wajib menangani
//! (receipt/opaque path).
//!
//! Keluarga varian `error.rs` kernel: korpus 186 tipe node TIDAK memuat
//! varian error kernel → pengecualian RULING 30 vacuous utk tabel ini;
//! pemetaan apa pun yang bergantung RFC §3.4 TIDAK dimulai (PROVISIONAL-
//! MENUNGGU-§3.4 bila nanti ada).

/// Error pemetaan kind — fail-loud (jangan pernah menebak).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindError {
    /// Tipe tidak dikenal — tak ada baris di tabel.
    Unmapped,
    /// Tipe di luar keluarga base (langchain/komunitas) — kategori opaque.
    NonBase,
}

impl std::fmt::Display for KindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KindError::Unmapped => write!(f, "tipe tak dikenal (unmapped)"),
            KindError::NonBase => write!(f, "tipe non-base (opaque, R-2)"),
        }
    }
}

/// Pemetaan tipe → kind kernel (NodeKind = String dotted).
///
/// Kunci = `type_full` persis seperti di JSON workflow n8n — tabel
/// eksplisit, greppable, tak bisa salah-tebak. Nilai = kind dot-case.
pub fn kind_for_type(type_full: &str) -> Result<&'static str, KindError> {
    if !type_full.starts_with("n8n-nodes-base.") {
        return Err(KindError::NonBase);
    }
    BASE_KIND
        .binary_search_by_key(&type_full, |(t, _)| t)
        .map(|i| BASE_KIND[i].1)
        .map_err(|_| KindError::Unmapped)
}

/// Konversi TypeVersion → version u16 kernel: MAJOR*10+minor, guard keras
/// minor<10 else unmappable (keputusan #1176; verifikasi korpus: 0
/// pelanggaran, #1199). TypeVersion pasca-P1 (#1188) menjamin minor<=9;
/// guard tetap fail-closed di sini (jaga-jaga, biaya nol).
pub fn version_u16(tv: crate::model::TypeVersion) -> Option<u16> {
    if tv.minor >= 10 {
        return None;
    }
    let v = tv.major.checked_mul(10)?.checked_add(tv.minor)?;
    u16::try_from(v).ok()
}

/// Jumlah baris tabel (konstanta; test korpus memakai ini utk memastikan
/// tabel == tipe yang teramati di korpus).
pub const fn table_len() -> usize {
    BASE_KIND.len()
}

// GENERATED dari korpus 171 fixture (scan type n8n-nodes-base.*, 2026-09-09).
// Derivation: strip prefix; camel-split + lowercase dot-join;
// EXCEPTIONS terkurai (brand/akronim: n8n, awsS3→aws.s3, linkedIn→linkedin,
// whatsApp→whatsapp, youTube→youtube, sendGrid→sendgrid, highLevel→highlevel,
// noOp→noop, mongoDb→mongodb) + splitInBatches utuh (RULING 35).
// RATIFIED: agent1 #1241 (6 koreksi: mailerLite→mailerlite,
// sendInBlue→sendinblue, nextCloud→nextcloud, nocoDb→nocodb,
// wooCommerce→woocommerce + Tool) + matt RULING 35 (#1249).
// Kolom komentar: nama remainder + versi korpus utk (tipe,major) cek lintas.
/// Tabel pemetaan eksplisit (tipe_full n8n → kind kernel). Terurut by tipe
/// (prasyarat `binary_search_by_key`); PUBLIK utk gate korpus eksternal
/// (tests/kindmap_corpus.rs) — read-only, jangan dimutasi.
pub static BASE_KIND: &[(&str, &str)] = &[
    ("n8n-nodes-base.aggregate", "aggregate"),
    ("n8n-nodes-base.aiTransform", "ai.transform"),
    ("n8n-nodes-base.airtable", "airtable"),
    ("n8n-nodes-base.airtableTrigger", "airtable.trigger"),
    ("n8n-nodes-base.asana", "asana"),
    ("n8n-nodes-base.autopilotTrigger", "autopilot.trigger"),
    ("n8n-nodes-base.awsRekognition", "aws.rekognition"),
    ("n8n-nodes-base.awsS3", "aws.s3"),
    ("n8n-nodes-base.awsSes", "aws.ses"),
    ("n8n-nodes-base.awsTranscribe", "aws.transcribe"),
    ("n8n-nodes-base.bitwarden", "bitwarden"),
    ("n8n-nodes-base.box", "box"),
    ("n8n-nodes-base.bubble", "bubble"),
    ("n8n-nodes-base.calendlyTrigger", "calendly.trigger"),
    ("n8n-nodes-base.clockify", "clockify"),
    ("n8n-nodes-base.coda", "coda"),
    ("n8n-nodes-base.code", "code"),
    ("n8n-nodes-base.compareDatasets", "compare.datasets"),
    ("n8n-nodes-base.compression", "compression"),
    ("n8n-nodes-base.convertToFile", "convert.to.file"),
    ("n8n-nodes-base.copper", "copper"),
    ("n8n-nodes-base.cron", "cron"),
    ("n8n-nodes-base.crypto", "crypto"),
    ("n8n-nodes-base.cryptoTool", "crypto.tool"),
    ("n8n-nodes-base.dateTime", "date.time"),
    ("n8n-nodes-base.dateTimeTool", "date.time.tool"),
    ("n8n-nodes-base.dhl", "dhl"),
    ("n8n-nodes-base.discord", "discord"),
    ("n8n-nodes-base.dropbox", "dropbox"),
    ("n8n-nodes-base.dropcontact", "dropcontact"),
    ("n8n-nodes-base.editImage", "edit.image"),
    ("n8n-nodes-base.elasticsearch", "elasticsearch"),
    ("n8n-nodes-base.emailReadImap", "email.read.imap"),
    ("n8n-nodes-base.emailSend", "email.send"),
    ("n8n-nodes-base.emelia", "emelia"),
    ("n8n-nodes-base.errorTrigger", "error.trigger"),
    ("n8n-nodes-base.executeCommand", "execute.command"),
    ("n8n-nodes-base.executeWorkflow", "execute.workflow"),
    ("n8n-nodes-base.executeWorkflowTrigger", "execute.workflow.trigger"),
    ("n8n-nodes-base.extractFromFile", "extract.from.file"),
    ("n8n-nodes-base.filter", "filter"),
    ("n8n-nodes-base.formTrigger", "form.trigger"),
    ("n8n-nodes-base.ftp", "ftp"),
    ("n8n-nodes-base.function", "function"),
    ("n8n-nodes-base.functionItem", "function.item"),
    ("n8n-nodes-base.github", "github"),
    ("n8n-nodes-base.githubTrigger", "github.trigger"),
    ("n8n-nodes-base.gmail", "gmail"),
    ("n8n-nodes-base.gmailTool", "gmail.tool"),
    ("n8n-nodes-base.gmailTrigger", "gmail.trigger"),
    ("n8n-nodes-base.googleAds", "google.ads"),
    ("n8n-nodes-base.googleBooks", "google.books"),
    ("n8n-nodes-base.googleCalendar", "google.calendar"),
    ("n8n-nodes-base.googleCalendarTool", "google.calendar.tool"),
    ("n8n-nodes-base.googleCalendarTrigger", "google.calendar.trigger"),
    ("n8n-nodes-base.googleDocs", "google.docs"),
    ("n8n-nodes-base.googleDrive", "google.drive"),
    ("n8n-nodes-base.googleSheets", "google.sheets"),
    ("n8n-nodes-base.googleSheetsTool", "google.sheets.tool"),
    ("n8n-nodes-base.googleSheetsTrigger", "google.sheets.trigger"),
    ("n8n-nodes-base.graphql", "graphql"),
    ("n8n-nodes-base.hackerNews", "hacker.news"),
    ("n8n-nodes-base.highLevel", "highlevel"),
    ("n8n-nodes-base.html", "html"),
    ("n8n-nodes-base.htmlExtract", "html.extract"),
    ("n8n-nodes-base.httpRequest", "http.request"),
    ("n8n-nodes-base.httpRequestTool", "http.request.tool"),
    ("n8n-nodes-base.hubspot", "hubspot"),
    ("n8n-nodes-base.hubspotTrigger", "hubspot.trigger"),
    ("n8n-nodes-base.hunter", "hunter"),
    ("n8n-nodes-base.if", "if"),
    ("n8n-nodes-base.interval", "interval"),
    ("n8n-nodes-base.itemLists", "item.lists"),
    ("n8n-nodes-base.jiraTrigger", "jira.trigger"),
    ("n8n-nodes-base.keap", "keap"),
    ("n8n-nodes-base.lemlist", "lemlist"),
    ("n8n-nodes-base.limit", "limit"),
    ("n8n-nodes-base.linkedIn", "linkedin"),
    ("n8n-nodes-base.mailchimp", "mailchimp"),
    ("n8n-nodes-base.mailerLite", "mailerlite"),
    ("n8n-nodes-base.mailgun", "mailgun"),
    ("n8n-nodes-base.manualTrigger", "manual.trigger"),
    ("n8n-nodes-base.markdown", "markdown"),
    ("n8n-nodes-base.mattermost", "mattermost"),
    ("n8n-nodes-base.mautic", "mautic"),
    ("n8n-nodes-base.mauticTrigger", "mautic.trigger"),
    ("n8n-nodes-base.medium", "medium"),
    ("n8n-nodes-base.merge", "merge"),
    ("n8n-nodes-base.microsoftExcel", "microsoft.excel"),
    ("n8n-nodes-base.mindee", "mindee"),
    ("n8n-nodes-base.mongoDb", "mongodb"),
    ("n8n-nodes-base.moveBinaryData", "move.binary.data"),
    ("n8n-nodes-base.mqtt", "mqtt"),
    ("n8n-nodes-base.mqttTrigger", "mqtt.trigger"),
    ("n8n-nodes-base.n8n", "n8n"),
    ("n8n-nodes-base.n8nTrainingCustomerDatastore", "n8n.training.customer.datastore"),
    ("n8n-nodes-base.nextCloud", "nextcloud"),
    ("n8n-nodes-base.noOp", "noop"),
    ("n8n-nodes-base.nocoDb", "nocodb"),
    ("n8n-nodes-base.notion", "notion"),
    ("n8n-nodes-base.notionTrigger", "notion.trigger"),
    ("n8n-nodes-base.oneSimpleApi", "one.simple.api"),
    ("n8n-nodes-base.onfleet", "onfleet"),
    ("n8n-nodes-base.perplexity", "perplexity"),
    ("n8n-nodes-base.pipedrive", "pipedrive"),
    ("n8n-nodes-base.postgres", "postgres"),
    ("n8n-nodes-base.readBinaryFile", "read.binary.file"),
    ("n8n-nodes-base.readPDF", "read.pdf"),
    ("n8n-nodes-base.removeDuplicates", "remove.duplicates"),
    ("n8n-nodes-base.renameKeys", "rename.keys"),
    ("n8n-nodes-base.respondToWebhook", "respond.to.webhook"),
    ("n8n-nodes-base.rssFeedRead", "rss.feed.read"),
    ("n8n-nodes-base.rssFeedReadTool", "rss.feed.read.tool"),
    ("n8n-nodes-base.scheduleTrigger", "schedule.trigger"),
    ("n8n-nodes-base.sendGrid", "sendgrid"),
    ("n8n-nodes-base.sendInBlue", "sendinblue"),
    ("n8n-nodes-base.set", "set"),
    ("n8n-nodes-base.shopifyTrigger", "shopify.trigger"),
    ("n8n-nodes-base.slack", "slack"),
    ("n8n-nodes-base.slackTrigger", "slack.trigger"),
    ("n8n-nodes-base.sort", "sort"),
    ("n8n-nodes-base.splitInBatches", "splitInBatches"), // kelas: node inti n8n — nama upstream dipertahankan (RULING 35)
    ("n8n-nodes-base.splitOut", "split.out"),
    ("n8n-nodes-base.spreadsheetFile", "spreadsheet.file"),
    ("n8n-nodes-base.stickyNote", "sticky.note"),
    ("n8n-nodes-base.summarize", "summarize"),
    ("n8n-nodes-base.supabase", "supabase"),
    ("n8n-nodes-base.switch", "switch"),
    ("n8n-nodes-base.telegram", "telegram"),
    ("n8n-nodes-base.telegramTool", "telegram.tool"),
    ("n8n-nodes-base.telegramTrigger", "telegram.trigger"),
    ("n8n-nodes-base.trello", "trello"),
    ("n8n-nodes-base.twilio", "twilio"),
    ("n8n-nodes-base.twilioTrigger", "twilio.trigger"),
    ("n8n-nodes-base.typeformTrigger", "typeform.trigger"),
    ("n8n-nodes-base.uproc", "uproc"),
    ("n8n-nodes-base.wait", "wait"),
    ("n8n-nodes-base.webhook", "webhook"),
    ("n8n-nodes-base.whatsApp", "whatsapp"),
    ("n8n-nodes-base.whatsAppTrigger", "whatsapp.trigger"),
    ("n8n-nodes-base.wooCommerce", "woocommerce"),
    ("n8n-nodes-base.wooCommerceTool", "woocommerce.tool"),
    ("n8n-nodes-base.wordpress", "wordpress"),
    ("n8n-nodes-base.writeBinaryFile", "write.binary.file"),
    ("n8n-nodes-base.xml", "xml"),
    ("n8n-nodes-base.youTube", "youtube"),
    ("n8n-nodes-base.zendesk", "zendesk"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TypeVersion;

    #[test]
    fn kind_contoh_kernel_dan_keputusan() {
        // contoh docstring kernel node.rs + keputusan #1176
        assert_eq!(
            kind_for_type("n8n-nodes-base.httpRequest"),
            Ok("http.request")
        );
        assert_eq!(kind_for_type("n8n-nodes-base.if"), Ok("if"));
        assert_eq!(kind_for_type("n8n-nodes-base.merge"), Ok("merge"));
        assert_eq!(kind_for_type("n8n-nodes-base.code"), Ok("code"));
        assert_eq!(kind_for_type("n8n-nodes-base.googleSheets"), Ok("google.sheets"));
        // splitInBatches UTUH (keputusan #1176) — bukan split.in.batches
        assert_eq!(
            kind_for_type("n8n-nodes-base.splitInBatches"),
            Ok("splitInBatches")
        );
        // exception brand/akronim
        assert_eq!(kind_for_type("n8n-nodes-base.awsS3"), Ok("aws.s3"));
        assert_eq!(kind_for_type("n8n-nodes-base.n8n"), Ok("n8n"));
    }

    #[test]
    fn kind_fail_loud_nonbase_dan_unmapped() {
        assert_eq!(kind_for_type("@n8n/n8n-nodes-langchain.agent"), Err(KindError::NonBase));
        assert_eq!(kind_for_type("@scope/nodes-x.y"), Err(KindError::NonBase));
        assert_eq!(kind_for_type("n8n-nodes-base.tidakAdaTipeIni"), Err(KindError::Unmapped));
        assert_eq!(kind_for_type("nonsense"), Err(KindError::NonBase));
    }

    #[test]
    fn version_u16_konvensi_dan_guard() {
        assert_eq!(version_u16(TypeVersion::new(1, 0)), Some(10));
        assert_eq!(version_u16(TypeVersion::new(1, 3)), Some(13));
        assert_eq!(version_u16(TypeVersion::new(4, 2)), Some(42));
        assert_eq!(version_u16(TypeVersion::new(2, 0)), Some(20));
        // minor>=10 → None fail-closed (tak mungkin lewat TypeVersion pasca-P1,
        // guard ganda biaya nol)
        assert_eq!(version_u16(TypeVersion::new(4, 10)), None);
        assert_eq!(version_u16(TypeVersion::new(5, 0)), Some(50));
    }

    #[test]
    fn tabel_urut_dan_tanpa_duplikat() {
        // prasyarat binary_search_by_key: tabel harus sorted by key & unik
        let mut prev: Option<&str> = None;
        for (t, _) in BASE_KIND {
            if let Some(p) = prev {
                assert!(p < t, "urutan tabel rusak: {p} >= {t}");
            }
            prev = Some(t);
        }
    }
}
