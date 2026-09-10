//! n8n-core: tipe format workflow yang kompatibel dengan n8n asli (JSON).
//!
//! Prinsip: field yang dikenal dimodelkan bertipe, field asing ditampung di
//! `extra` (flatten) supaya tidak ada data yang hilang saat impor → ekspor.

pub mod expr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Satu workflow n8n: cocok dengan struktur JSON ekspor n8n
/// (`name`, `nodes`, `connections`, `active`, `settings`, ...).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub nodes: Vec<WorkflowNode>,
    /// `connections` n8n: `{ "NamaNode": { "main": [[{"node","type","index"}]] } }`.
    /// Disimpan generik (Value) agar varian bentuk tidak menggagalkan parse.
    #[serde(default)]
    pub connections: HashMap<String, Value>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub settings: HashMap<String, Value>,
    /// Field lain apa pun (pinData, versionId, meta, tags, ...) — tidak hilang.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Satu node dalam workflow. `type` n8n dipetakan ke `node_type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowNode {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    /// n8n menulis ini sebagai angka, kadang pecahan (mis. 3.4) — jadi f64.
    #[serde(default)]
    pub type_version: f64,
    /// Posisi kanvas `[x, y]`.
    #[serde(default)]
    pub position: [f64; 2],
    /// Parameter node — schemaless seperti n8n (tipe node menafsirkan isinya).
    #[serde(default)]
    pub parameters: HashMap<String, Value>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl Workflow {
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn node_by_name(&self, name: &str) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|n| n.name == name)
    }

    /// Penerus per cabang output (`main[i]`, indeks = cabang) — untuk node
    /// multi-output (If). Cabang tak berbentuk jadi list kosong (indeks tetap).
    /// Parsing defensif: bentuk yang tak dikenal dilewati, bukan error.
    pub fn branches(&self, node_name: &str) -> Vec<Vec<String>> {
        let entry = match self.connections.get(node_name) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let main = match entry.get("main").and_then(Value::as_array) {
            Some(a) => a,
            None => return Vec::new(),
        };
        main.iter()
            .map(|branch| {
                branch
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(Value::as_object)
                            .filter_map(|o| o.get("node"))
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .collect()
    }

    /// Nama node-node penerus via output `main` (semua cabang digabung),
    /// sesuai urutan edge.
    pub fn successors(&self, node_name: &str) -> Vec<String> {
        self.branches(node_name).into_iter().flatten().collect()
    }

    pub fn edge_count(&self) -> usize {
        self.nodes
            .iter()
            .map(|n| self.successors(&n.name).len())
            .sum()
    }

    /// Edge yang targetnya bukan node (`(dari, ke)`) — untuk linter.
    pub fn dangling_edges(&self) -> Vec<(String, String)> {
        let names: std::collections::HashSet<&str> =
            self.nodes.iter().map(|n| n.name.as_str()).collect();
        let mut out = Vec::new();
        for n in &self.nodes {
            for s in self.successors(&n.name) {
                if !names.contains(s.as_str()) {
                    out.push((n.name.clone(), s));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../fixtures/manual-to-set.json");

    #[test]
    fn parses_real_n8n_format() {
        let wf = Workflow::from_json(FIXTURE).expect("parse fixture");
        assert_eq!(wf.name, "Manual to Set (v1 fixture)");
        assert_eq!(wf.nodes.len(), 3);
        assert_eq!(wf.successors("Manual Trigger"), vec!["Set".to_string()]);
        assert_eq!(wf.successors("Set"), vec!["NoOp".to_string()]);
        assert!(wf.successors("NoOp").is_empty());
        assert_eq!(wf.edge_count(), 2);
    }

    #[test]
    fn unknown_fields_survive_roundtrip() {
        let wf = Workflow::from_json(FIXTURE).expect("parse fixture");
        // pinData bukan field eksplisit Workflow -> harus mendarat di extra.
        assert!(wf.extra.contains_key("pinData"));
        let back = Workflow::from_json(&wf.to_json_pretty().expect("serialize")).expect("re-parse");
        assert_eq!(wf, back);
    }
}
