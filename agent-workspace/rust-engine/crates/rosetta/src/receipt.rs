//! Receipt migrasi (M3) — artefak JSON kecil yang menyertai workflow bersih:
//! ringkasan + tiap migrasi (rule, asumsi) + unresolved + node opaque.
//!
//! Kontrak R-5: `deviation_id` TIDAK dibuat lokal — unresolved memakai
//! `reason_code` deskriptor stabil; tautan ke DEVIATION-CATALOG (agent5)
//! ditambahkan saat katalog terbit (nol ID lokal; konsumen agent1/agent9
//! memutuskan fail-loud berdasar receipt).

use serde::{Deserialize, Serialize};

use crate::migrate::{MigrationEntry, MigrationReport, UnresolvedEntry};
use crate::model::{OpaqueNote, Workflow};

/// Skema receipt saat ini.
pub const RECEIPT_SCHEMA_VERSION: u32 = 1;

/// Satu entri receipt utk node opaque (R-2) — salinan catatan parse.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpaqueReceiptEntry {
    /// Nama node.
    pub node_name: String,
    /// Type penuh.
    pub type_full: String,
    /// Alasan opaque.
    pub reason: String,
}

impl From<&OpaqueNote> for OpaqueReceiptEntry {
    fn from(n: &OpaqueNote) -> Self {
        Self {
            node_name: n.node_name.clone(),
            type_full: n.type_full.clone(),
            reason: n.reason.clone(),
        }
    }
}

/// Ringkasan angka (memudahkan gate).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReceiptSummary {
    /// Total node input.
    pub input_nodes: usize,
    /// Node opaque (R-2).
    pub opaque_nodes: usize,
    /// Migrasi berhasil (M2).
    pub migrated: usize,
    /// Node unresolved.
    pub unresolved: usize,
}

/// Receipt lengkap satu workflow.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    /// Skema receipt.
    pub schema_version: u32,
    /// Nama workflow (bila ada di sumber).
    pub workflow_name: Option<String>,
    /// Ringkasan.
    pub summary: ReceiptSummary,
    /// Migrasi berhasil, urutan aplikasi.
    pub migrations: Vec<MigrationEntry>,
    /// Node tidak dimigrasikan.
    pub unresolved: Vec<UnresolvedEntry>,
    /// Node opaque (R-2).
    pub opaque: Vec<OpaqueReceiptEntry>,
    /// Catatan kebijakan/konteks.
    pub notes: Vec<String>,
}

/// Bangun receipt dari workflow asli + laporan migrasi.
pub fn build_receipt(wf: &Workflow, report: &MigrationReport) -> Receipt {
    Receipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        workflow_name: wf.name.clone(),
        summary: ReceiptSummary {
            input_nodes: wf.node_count(),
            opaque_nodes: wf.opaque_count(),
            migrated: report.migrated_count(),
            unresolved: report.unresolved_count(),
        },
        migrations: report.migrations.clone(),
        unresolved: report.unresolved.clone(),
        opaque: wf.opaque_nodes.iter().map(Into::into).collect(),
        notes: report.notes.clone(),
    }
}

/// Serialisasi receipt → JSON (2-space, deterministik).
pub fn receipt_to_json(receipt: &Receipt) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrate::migrate_workflow;
    use crate::parse::parse_workflow_str;

    #[test]
    fn receipt_lengkap_dan_json_ok() {
        let wf = parse_workflow_str(
            r#"{"name":"w","nodes":[
                {"name":"C","type":"n8n-nodes-base.cron","parameters":{"triggerTimes":{"item":[{"mode":"everyMinute"}]}}},
                {"name":"K","type":"@x/y.z","parameters":{}}
            ],"connections":{}}"#,
            "t",
        )
        .unwrap();
        let (_, report) = migrate_workflow(&wf);
        let rc = build_receipt(&wf, &report);
        assert_eq!(rc.summary.input_nodes, 2);
        assert_eq!(rc.summary.opaque_nodes, 1);
        assert_eq!(rc.summary.migrated, 1);
        let json = receipt_to_json(&rc).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["summary"]["migrated"], 1);
        // determinisme serialisasi
        assert_eq!(receipt_to_json(&rc).unwrap(), json);
    }
}
