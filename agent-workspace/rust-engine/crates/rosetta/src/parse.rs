//! Parser workflow JSON → grafik kanonik (M1).
//!
//! Kontrak:
//! - Murni baca: TIDAK menulis apa pun (R-1: nol mutasi file sumber).
//! - Deterministik: urutan node = urutan sumber; tidak ada state global (R-8).
//! - Opaque-safe: node bertipe tak dikenal tetap masuk grafik utuh + dicatat
//!   (R-2). Hanya JSON tidak valid / struktur rusak (bukan tipe asing) yang
//!   menghasilkan error — fail-loud disengaja, supaya file korup terlihat.

use std::path::Path;

use serde_json::{Map, Value};

use crate::catalog::origin_of;
use crate::error::RosettaError;
use crate::model::{CanonNode, NodeOrigin, OpaqueNote, TypeVersion, Workflow};

/// Batas kedalaman JSON untuk parse workflow (konsisten mcp = 64; kernel R41/47b).
pub const MAX_JSON_DEPTH: u32 = 64;

/// Parse workflow dari byte JSON.
pub fn parse_workflow_bytes(bytes: &[u8], label: &str) -> Result<Workflow, RosettaError> {
    // Guard kedalaman di BATAS API crate (RULING 50b/50c): input tak-tepercaya
    // (termasuk parse_workflow_path yang membaca disk) ditolak fail-closed SEBELUM
    // serde_json — kernel::json_depth_exceeded memberi kedalaman dalam satu lintasan.
    if let Some(found) = kernel::json_depth_exceeded(bytes, MAX_JSON_DEPTH) {
        return Err(RosettaError::DepthExceeded {
            path: label.to_string(),
            found,
            max: MAX_JSON_DEPTH,
        });
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|e| RosettaError::Json {
        path: label.to_string(),
        message: e.to_string(),
    })?;
    parse_workflow_value(value, label)
}

/// Parse workflow dari string JSON.
pub fn parse_workflow_str(s: &str, label: &str) -> Result<Workflow, RosettaError> {
    parse_workflow_bytes(s.as_bytes(), label)
}

/// Baca + parse file. Tidak pernah memodifikasi file (R-1).
pub fn parse_workflow_path(path: &Path) -> Result<Workflow, RosettaError> {
    let bytes = std::fs::read(path).map_err(|e| RosettaError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    parse_workflow_bytes(&bytes, &path.display().to_string())
}

fn parse_workflow_value(value: Value, label: &str) -> Result<Workflow, RosettaError> {
    let obj = value
        .as_object()
        .ok_or_else(|| structure(label, "root bukan objek JSON"))?;

    // --- nodes -----------------------------------------------------------
    let mut nodes = Vec::new();
    let mut opaque_nodes: Vec<OpaqueNote> = Vec::new();

    let Some(raw_nodes) = obj.get("nodes") else {
        return Err(structure(label, "kunci `nodes` tidak ada"));
    };
    let arr = raw_nodes
        .as_array()
        .ok_or_else(|| structure(label, "`nodes` bukan array"))?;

    for (i, raw) in arr.iter().enumerate() {
        let n = raw
            .as_object()
            .ok_or_else(|| structure(label, &format!("nodes[{i}] bukan objek")))?;

        let name = get_string(n, "name", label, i)?
            .ok_or_else(|| structure(label, &format!("nodes[{i}] tanpa `name` (string)")))?;

        let type_full = get_string(n, "type", label, i)?
            .ok_or_else(|| structure(label, &format!("nodes[{i}] tanpa `type` (string)")))?;

        // typeVersion: number utuh (1, 1.2, 1.3) atau string; aneh → None
        // (mentah dipertahankan di extra supaya tak ada informasi hilang)
        let mut type_version = n.get("typeVersion").and_then(TypeVersion::from_value);
        let mut extra_pre: Map<String, Value> = Map::new();
        if n.contains_key("typeVersion") && type_version.is_none() {
            extra_pre.insert(
                "typeVersion_raw".to_string(),
                n.get("typeVersion").cloned().unwrap(),
            );
            type_version = None;
        }

        let position = n.get("position").and_then(|p| {
            let a = p.as_array()?;
            if a.len() != 2 {
                return None;
            }
            let x = a[0].as_f64()?;
            let y = a[1].as_f64()?;
            Some([x, y])
        });

        let parameters = n
            .get("parameters")
            .cloned()
            .unwrap_or_else(|| Value::Object(Map::new()));

        // kunci lain (id, disabled, notes, webhookId, dst.) — preserved utuh
        let mut extra = extra_pre;
        for (k, v) in n {
            if !matches!(
                k.as_str(),
                "name" | "type" | "typeVersion" | "position" | "parameters"
            ) {
                extra.insert(k.clone(), v.clone());
            }
        }

        let origin = origin_of(&type_full);
        let node = CanonNode {
            name,
            type_full,
            type_version,
            origin,
            position,
            parameters,
            extra,
        };

        if node.origin.is_opaque() {
            opaque_nodes.push(OpaqueNote {
                node_index: nodes.len(),
                node_name: node.name.clone(),
                type_full: node.type_full.clone(),
                reason: opaque_reason(node.origin).to_string(),
            });
        }

        nodes.push(node);
    }

    // --- connections + sisa kunci ---------------------------------------
    let connections = obj
        .get("connections")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));

    let mut extra = Map::new();
    for (k, v) in obj {
        if !matches!(k.as_str(), "nodes" | "connections") {
            extra.insert(k.clone(), v.clone());
        }
    }

    // name di workflow: tersimpan terstruktur bila string
    let name = extra
        .remove("name")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    Ok(Workflow {
        name,
        nodes,
        connections,
        extra,
        opaque_nodes,
    })
}

fn opaque_reason(origin: NodeOrigin) -> &'static str {
    match origin {
        NodeOrigin::N8nBase | NodeOrigin::N8nFirstParty => {
            "node first-party — katalog 694 verifikasi per-tipe (R-6) menyusul"
        }
        NodeOrigin::Community => "node komunitas di luar katalog resmi (R-2)",
        NodeOrigin::Unknown => "tipe di luar namespace n8n yang dikenal (R-2)",
    }
}

fn get_string(
    n: &Map<String, Value>,
    key: &str,
    label: &str,
    i: usize,
) -> Result<Option<String>, RosettaError> {
    match n.get(key) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(structure(label, &format!("nodes[{i}].{key} bukan string"))),
    }
}

fn structure(label: &str, reason: &str) -> RosettaError {
    RosettaError::Structure {
        path: label.to_string(),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WF_LEGACY: &str = r#"{
        "name": "wf-legacy",
        "nodes": [
            {"name":"Trigger","type":"n8n-nodes-base.manualTrigger","position":[0,0],"parameters":{},"typeVersion":1},
            {"name":"Cron","type":"n8n-nodes-base.cron","position":[320,440],"parameters":{"triggerTimes":{"item":[{"mode":"everyX","unit":"minutes","value":5}]}},"typeVersion":1},
            {"name":"Kom","type":"@blotato/n8n-nodes-blotato.blotato","position":[640,300],"parameters":{"custom":"x"}},
            {"name":"Lama","type":"n8n-nodes-base.function","position":[960,300],"parameters":{"functionCode":"return items;"}}
        ],
        "connections": {"Cron":{"main":[[{"node":"Kom","type":"main","index":0}]]}},
        "settings": {"executionOrder":"v1"}
    }"#;

    #[test]
    fn parse_preserves_everything() {
        let w = parse_workflow_str(WF_LEGACY, "test").expect("parse");
        assert_eq!(w.node_count(), 4);
        assert_eq!(w.name.as_deref(), Some("wf-legacy"));

        let cron = w.node("Cron").unwrap();
        assert_eq!(cron.type_full, "n8n-nodes-base.cron");
        assert_eq!(cron.origin, NodeOrigin::N8nBase);
        assert_eq!(cron.effective_major(), 1);
        assert_eq!(cron.position, Some([320.0, 440.0]));
        assert_eq!(
            cron.parameters.pointer("/triggerTimes/item/0/mode"),
            Some(&Value::String("everyX".into()))
        );

        // extra workflow (settings) preserved
        assert!(w.extra.contains_key("settings"));
    }

    #[test]
    fn node_tanpa_type_version_default_1() {
        let json = r#"{"nodes":[{"name":"N","type":"n8n-nodes-base.code","position":[0,0],"parameters":{"jsCode":"x"}}],"connections":{}}"#;
        let w = parse_workflow_str(json, "t").expect("parse");
        assert_eq!(w.node("N").unwrap().type_version, None);
        assert_eq!(w.node("N").unwrap().effective_major(), 1);
    }

    #[test]
    fn type_version_float_dipertahankan() {
        // bukti korpus: scheduleTrigger 1.1/1.2/1.3 (bukan integer!)
        for (raw, maj, min) in [("1", 1, 0), ("1.0", 1, 0), ("1.2", 1, 2), ("1.3", 1, 3)] {
            let json = format!(
                r#"{{"nodes":[{{"name":"N","type":"n8n-nodes-base.scheduleTrigger","typeVersion":{raw},"parameters":{{}}}}],"connections":{{}}}}"#
            );
            let w = parse_workflow_str(&json, "t").expect("parse");
            let tv = w.node("N").unwrap().type_version.expect("terparse");
            assert_eq!((tv.major, tv.minor), (maj, min), "typeVersion {raw}");
            // roundtrip JSON n8n: minor 0 → integer kanonik; minor >0 → `maj.min`
            let expect = if min == 0 {
                serde_json::json!(maj)
            } else {
                serde_json::json!(maj as f64 + min as f64 / 10.0)
            };
            assert_eq!(tv.to_json_value(), expect, "roundtrip {raw}");
        }
        // typeVersion tak lazim → None + mentah dipertahankan
        let json = r#"{"nodes":[{"name":"N","type":"n8n-nodes-base.x","typeVersion":"v9","parameters":{}}],"connections":{}}"#;
        let w = parse_workflow_str(json, "t").expect("parse");
        assert_eq!(w.node("N").unwrap().type_version, None);
        assert_eq!(
            w.node("N").unwrap().extra.get("typeVersion_raw").unwrap(),
            "v9"
        );
    }

    #[test]
    fn opaque_community_diterima_dan_dicatat() {
        let w = parse_workflow_str(WF_LEGACY, "test").expect("parse");
        assert_eq!(w.opaque_count(), 1);
        let note = &w.opaque_nodes[0];
        assert_eq!(note.node_name, "Kom");
        assert_eq!(note.type_full, "@blotato/n8n-nodes-blotato.blotato");
        // node tetap ADA dalam grafik dgn parameter utuh (R-2)
        let node = w.node("Kom").unwrap();
        assert_eq!(node.origin, NodeOrigin::Community);
        assert_eq!(
            node.parameters.pointer("/custom"),
            Some(&Value::String("x".into()))
        );
    }

    #[test]
    fn deterministic_double_parse() {
        let a = parse_workflow_str(WF_LEGACY, "t").unwrap();
        let b = parse_workflow_str(WF_LEGACY, "t").unwrap();
        assert_eq!(a, b, "R-8: 2× parse input sama harus identik");
    }

    #[test]
    fn fail_loud_on_corrupt() {
        let e = parse_workflow_str("{not json", "x").unwrap_err();
        assert!(matches!(e, RosettaError::Json { .. }));

        let e = parse_workflow_str(r#"{"nodes":{},"connections":{}}"#, "x").unwrap_err();
        assert!(matches!(e, RosettaError::Structure { .. }));

        let e =
            parse_workflow_str(r#"{"nodes":[{"type":"x"}],"connections":{}}"#, "x").unwrap_err();
        assert!(matches!(e, RosettaError::Structure { .. }));
    }
}
