//! Migrator (M2): terapkan aturan registry v1 ke grafik kanonik → grafik
//! bersih + laporan. Murni fungsi: input `&Workflow`, output node baru +
//! `MigrationReport`. File sumber TIDAK pernah disentuh (R-1); deterministik
//! (R-8). Node yang tidak bisa dimigrasikan dengan aman TIDAK diubah dan
//! dicatat di `unresolved` — konsumen (engine agent1 / Hub agent9) memutuskan
//! fail-loud berdasar receipt (aturan katalog alias §3: gagal eksplisit >
//! hasil salah diam-diam).

use serde_json::Value;

use crate::model::{CanonNode, TypeVersion, Workflow};
use crate::registry::{
    cron_params_to_rule, function_params_to_code, UnresolvedKind, RULE_CRON_TO_SCHEDULE,
    RULE_FUNCTION_TO_CODE,
};

/// TypeVersion target Code (v2 = terbukti 87× integer 2 di korpus).
pub const CODE_TARGET: TypeVersion = TypeVersion::new(2, 0);
/// TypeVersion target scheduleTrigger: 1.3 — tertinggi terbukti di korpus
/// (22 file memakai 1.1–1.3; bentuk rule identik). KOREKSI atas usul
/// typeVersion 2 di AGENT4-NODE-ALIAS-DEPRECATION.md v0.1 §2 (dokumen
/// menulis "2" tanpa dasar korpus; korpus & n8n saat ini memakai 1.x).
pub const SCHEDULE_TARGET: TypeVersion = TypeVersion::new(1, 3);

/// Satu migrasi yang berhasil diterapkan.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MigrationEntry {
    /// Nama node (unik dalam workflow).
    pub node_name: String,
    /// Type asli dari sumber.
    pub from_type: String,
    /// Type hasil.
    pub to_type: String,
    /// TypeVersion hasil (JSON n8n: `2` atau `1.3`).
    pub to_type_version: Value,
    /// ID rule (registry v1).
    pub rule_version: String,
    /// Asumsi transform (mis. prolog items / default K-2), bukan diam-diam.
    pub assumptions: Vec<String>,
}

/// Node yang TIDAK dimigrasikan (tetap type asli) — alasan standar.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UnresolvedEntry {
    /// Nama node.
    pub node_name: String,
    /// Type asli.
    pub from_type: String,
    /// Kode alasan (deskriptor stabil; ID DEVIATION-CATALOG menyusul — R-5).
    pub reason_code: String,
    /// Detail tambahan.
    pub detail: String,
}

/// Laporan migrasi satu workflow (bahan receipt, M3).
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MigrationReport {
    /// Migrasi berhasil.
    pub migrations: Vec<MigrationEntry>,
    /// Node tidak dimigrasikan.
    pub unresolved: Vec<UnresolvedEntry>,
    /// Catatan umum (mis. selisih target typeVersion vs katalog alias).
    pub notes: Vec<String>,
}

impl MigrationReport {
    /// Jumlah migrasi berhasil.
    pub fn migrated_count(&self) -> usize {
        self.migrations.len()
    }
    /// Jumlah unresolved.
    pub fn unresolved_count(&self) -> usize {
        self.unresolved.len()
    }
}

/// Migrasikan seluruh node `Workflow` sesuai registry v1.
/// Mengembalikan (node hasil — urutan sama dgn sumber, R-8 — , laporan).
pub fn migrate_workflow(wf: &Workflow) -> (Vec<CanonNode>, MigrationReport) {
    let mut out = Vec::with_capacity(wf.nodes.len());
    let mut report = MigrationReport::default();

    for node in &wf.nodes {
        match try_migrate_node(node) {
            Ok((mut migrated, entry)) => {
                // jejak migrasi deterministik di extra (bukan di parameters)
                migrated.extra.insert(
                    "rosetta".into(),
                    serde_json::json!({
                        "migrated": true,
                        "from_type": node.type_full,
                        "rule": entry.rule_version,
                    }),
                );
                report.migrations.push(entry);
                out.push(migrated);
            }
            Err(unresolved) => {
                if unresolved.reason_code == "not-in-scope-v1" {
                    // node di luar 3 aturan v1: bukan unresolved masalah — bersih
                    out.push(node.clone());
                } else {
                    report.unresolved.push(unresolved);
                    out.push(node.clone());
                }
            }
        }
    }
    (out, report)
}

fn try_migrate_node(node: &CanonNode) -> Result<(CanonNode, MigrationEntry), UnresolvedEntry> {
    let unresolved = |kind: UnresolvedKind, detail: String| UnresolvedEntry {
        node_name: node.name.clone(),
        from_type: node.type_full.clone(),
        reason_code: kind.code().to_string(),
        detail,
    };
    let not_in_scope = UnresolvedEntry {
        node_name: node.name.clone(),
        from_type: node.type_full.clone(),
        reason_code: "not-in-scope-v1".into(),
        detail: "tipe di luar 3 aturan registry v1".into(),
    };

    match node.type_full.as_str() {
        "n8n-nodes-base.function" => {
            if node.effective_major() != 1 {
                return Err(unresolved(
                    UnresolvedKind::UnexpectedVersion,
                    format!("typeVersion {}", node.effective_major()),
                ));
            }
            let (params, assumptions) = function_params_to_code(&node.parameters)
                .map_err(|k| unresolved(k, "functionCode tidak berbentuk string".into()))?;
            let mut out = node.clone();
            out.type_full = "n8n-nodes-base.code".into();
            out.type_version = Some(CODE_TARGET);
            out.parameters = params;
            Ok((
                out,
                MigrationEntry {
                    node_name: node.name.clone(),
                    from_type: "n8n-nodes-base.function".into(),
                    to_type: "n8n-nodes-base.code".into(),
                    to_type_version: CODE_TARGET.to_json_value(),
                    rule_version: RULE_FUNCTION_TO_CODE.into(),
                    assumptions,
                },
            ))
        }
        "n8n-nodes-base.functionItem" => Err(unresolved(
            UnresolvedKind::FunctionItemAwaitingV3b,
            "var `item` legacy = data plain (bukti korpus tpl-156/tpl-175); \
             padanan modern belum terverifikasi upstream (V-3b OPEN, katalog \
             alias §4b) — transform ditunda sampai konfirmasi, bukan hasil salah"
                .into(),
        )),
        "n8n-nodes-base.cron" => {
            if node.effective_major() != 1 {
                return Err(unresolved(
                    UnresolvedKind::UnexpectedVersion,
                    format!("typeVersion {}", node.effective_major()),
                ));
            }
            let rule = cron_params_to_rule(&node.parameters)
                .map_err(|e| unresolved(UnresolvedKind::UnmappableShape, e))?;
            let mut out = node.clone();
            out.type_full = "n8n-nodes-base.scheduleTrigger".into();
            out.type_version = Some(SCHEDULE_TARGET);
            out.parameters = Value::Object([("rule".to_string(), rule)].into_iter().collect());
            Ok((
                out,
                MigrationEntry {
                    node_name: node.name.clone(),
                    from_type: "n8n-nodes-base.cron".into(),
                    to_type: "n8n-nodes-base.scheduleTrigger".into(),
                    to_type_version: SCHEDULE_TARGET.to_json_value(),
                    rule_version: RULE_CRON_TO_SCHEDULE.into(),
                    assumptions: vec![
                        "default field kosong = 0 (K-2 katalog alias §4b)".into(),
                        "ekspresi 6-field dibuang detiknya (V-2b: granularitas menit)".into(),
                    ],
                },
            ))
        }
        _ => Err(not_in_scope),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_workflow_str;

    const WF: &str = r#"{
        "nodes": [
            {"name":"Cron","type":"n8n-nodes-base.cron","typeVersion":1,"position":[0,0],
             "parameters":{"triggerTimes":{"item":[{"mode":"everyX","unit":"minutes","value":5}]}}},
            {"name":"Fn","type":"n8n-nodes-base.function","typeVersion":1,"position":[0,0],
             "parameters":{"functionCode":"items[0].json.x=1; return items;"}},
            {"name":"FItem","type":"n8n-nodes-base.functionItem","typeVersion":1,"position":[0,0],
             "parameters":{"functionCode":"item = item.data; return item;"}},
            {"name":"Modern","type":"n8n-nodes-base.code","typeVersion":2,"position":[0,0],
             "parameters":{"jsCode":"return $input.all();","mode":"runOnceForEachItem"}},
            {"name":"Kom","type":"@blotato/n8n-nodes-blotato.blotato","typeVersion":1,
             "position":[0,0],"parameters":{"custom":"x"}}
        ],
        "connections": {}
    }"#;

    #[test]
    fn migrasi_scope_v1() {
        let wf = parse_workflow_str(WF, "t").unwrap();
        let (out, report) = migrate_workflow(&wf);
        assert_eq!(report.migrated_count(), 2, "cron + function termigrasi");
        assert_eq!(
            report.unresolved_count(),
            1,
            "hanya functionItem (V-3b); code & komunitas = di luar scope (bukan unresolved)"
        );

        let cron = out.iter().find(|n| n.name == "Cron").unwrap();
        assert_eq!(cron.type_full, "n8n-nodes-base.scheduleTrigger");
        assert_eq!(cron.type_version, Some(TypeVersion::new(1, 3)));
        assert_eq!(
            cron.parameters
                .pointer("/rule/interval/0/expression")
                .unwrap(),
            "*/5 * * * *"
        );
        assert!(cron.extra.get("rosetta").is_some());

        let fn_node = out.iter().find(|n| n.name == "Fn").unwrap();
        assert_eq!(fn_node.type_full, "n8n-nodes-base.code");
        assert_eq!(fn_node.type_version, Some(TypeVersion::new(2, 0)));
        assert_eq!(
            fn_node.parameters.get("mode").unwrap(),
            "runOnceForAllItems"
        );
        assert!(fn_node
            .parameters
            .get("jsCode")
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("const items = $input.all();"));

        // functionItem TIDAK berubah type (unresolved jujur)
        let fitem = out.iter().find(|n| n.name == "FItem").unwrap();
        assert_eq!(fitem.type_full, "n8n-nodes-base.functionItem");
        assert!(report
            .unresolved
            .iter()
            .any(|u| u.node_name == "FItem" && u.reason_code.contains("v3b")));

        // komunitas & modern tak tersentuh (tanpa jejak rosetta)
        assert!(out.iter().any(|n| n.name == "Kom" && n.extra.is_empty()));
        assert!(out.iter().any(|n| n.name == "Modern" && n.extra.is_empty()));

        // urutan dipertahankan (R-8)
        let names: Vec<&str> = out.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, vec!["Cron", "Fn", "FItem", "Modern", "Kom"]);
    }

    #[test]
    fn deterministik_double_migrate() {
        let wf = parse_workflow_str(WF, "t").unwrap();
        let (a, ra) = migrate_workflow(&wf);
        let (b, rb) = migrate_workflow(&wf);
        assert_eq!(a, b);
        assert_eq!(ra, rb);
    }
}
