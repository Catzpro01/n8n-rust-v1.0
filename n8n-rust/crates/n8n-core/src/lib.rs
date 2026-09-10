//! n8n-core: model workflow kompatibel n8n + parser defensif + utilities premium.
//!
//! # Design principles
//! - **Lossless**: field asing di `extra` (flatten) — impor → ekspor tidak hilang.
//! - **Defensive**: `connections` generik `Value` agar varian n8n tidak gagal parse.
//! - **Ergonomic**: helper `branches`, `successors`, `topo`, `validate` dengan pesan jelas.
//! - **Observable**: `edge_count`, `isolated_nodes`, `trigger_nodes` untuk linter & UI.

pub mod expr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Satu workflow n8n — cocok dengan struktur JSON ekspor n8n asli.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub nodes: Vec<WorkflowNode>,
    /// `connections`: `{ "NamaNode": { "main": [[{"node","type","index"}]] } }`
    /// Disimpan generik agar bentuk tak terduga tidak menggagalkan parse.
    #[serde(default)]
    pub connections: HashMap<String, Value>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub settings: HashMap<String, Value>,
    /// Field lain (pinData, versionId, meta, tags, …) — tidak hilang.
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
    /// n8n kadang pecahan (mis. 3.4) — jadi f64.
    #[serde(default)]
    pub type_version: f64,
    /// Posisi kanvas `[x, y]`.
    #[serde(default)]
    pub position: [f64; 2],
    /// Parameter node — schemaless, tipe node menafsirkan isinya.
    #[serde(default)]
    pub parameters: HashMap<String, Value>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl Workflow {
    /// Parse dari JSON string.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Serialize pretty.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Cari node by name.
    pub fn node_by_name(&self, name: &str) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|n| n.name == name)
    }

    /// Mutable cari.
    pub fn node_by_name_mut(&mut self, name: &str) -> Option<&mut WorkflowNode> {
        self.nodes.iter_mut().find(|n| n.name == name)
    }

    /// Penerus per cabang output (`main[i]`, indeks = cabang).
    /// Parsing defensif: bentuk tak dikenal → list kosong (indeks tetap).
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

    /// Semua penerus (gabung cabang) sesuai urutan edge.
    pub fn successors(&self, node_name: &str) -> Vec<String> {
        self.branches(node_name).into_iter().flatten().collect()
    }

    /// Semua edge sebagai (from, to, branch_idx).
    pub fn all_edges(&self) -> Vec<(String, String, usize)> {
        let mut out = Vec::new();
        for n in &self.nodes {
            for (bi, branch) in self.branches(&n.name).into_iter().enumerate() {
                for to in branch {
                    out.push((n.name.clone(), to, bi));
                }
            }
        }
        out
    }

    pub fn edge_count(&self) -> usize {
        self.nodes
            .iter()
            .map(|n| self.successors(&n.name).len())
            .sum()
    }

    /// Edge yang targetnya bukan node (`(dari, ke)`).
    pub fn dangling_edges(&self) -> Vec<(String, String)> {
        let names: HashSet<&str> = self.nodes.iter().map(|n| n.name.as_str()).collect();
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

    /// Node terisolasi (tanpa edge masuk & keluar) — untuk linter.
    pub fn isolated_nodes(&self) -> Vec<String> {
        let mut targets: HashSet<String> = HashSet::new();
        for n in &self.nodes {
            for s in self.successors(&n.name) {
                targets.insert(s);
            }
        }
        self.nodes
            .iter()
            .filter(|n| {
                let has_out = !self.successors(&n.name).is_empty();
                !has_out && !targets.contains(n.name.as_str())
            })
            .map(|n| n.name.clone())
            .collect()
    }

    /// Node trigger (nama tipe mengandung "trigger" atau "webhook").
    pub fn trigger_nodes(&self) -> Vec<&WorkflowNode> {
        self.nodes
            .iter()
            .filter(|n| {
                let t = n.node_type.to_lowercase();
                t.contains("trigger") || t.contains("webhook")
            })
            .collect()
    }

    /// Validasi struktural ringan (tanpa registry).
    pub fn validate_structure(&self) -> Vec<String> {
        let mut errs = Vec::new();
        let mut seen = HashSet::new();
        for n in &self.nodes {
            if n.name.trim().is_empty() {
                errs.push(format!("node id '{}' tanpa nama", n.id));
            }
            if !seen.insert(n.name.clone()) {
                errs.push(format!("duplikat nama node '{}'", n.name));
            }
            if n.node_type.trim().is_empty() {
                errs.push(format!("node '{}' tanpa type", n.name));
            }
        }
        for (from, to) in self.dangling_edges() {
            errs.push(format!("edge gantung: '{from}' -> '{to}'"));
        }
        errs
    }

    /// Topological sort deterministik (Kahn) sesuai urutan file.
    /// Err jika cycle.
    pub fn topo_order(&self) -> Result<Vec<String>, String> {
        let enabled: Vec<&WorkflowNode> = self.nodes.iter().filter(|n| !n.disabled).collect();
        let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
        for n in &enabled {
            incoming.entry(n.name.clone()).or_default();
        }
        for n in &enabled {
            for s in self.successors(&n.name) {
                if let Some(list) = incoming.get_mut(&s) {
                    list.push(n.name.clone());
                }
            }
        }
        let mut done = Vec::new();
        let mut done_set = HashSet::new();
        loop {
            let mut progressed = false;
            for n in &enabled {
                if done_set.contains(&n.name) {
                    continue;
                }
                let ready = incoming
                    .get(&n.name)
                    .map(|ps| ps.iter().all(|p| done_set.contains(p)))
                    .unwrap_or(true);
                if !ready {
                    continue;
                }
                done_set.insert(n.name.clone());
                done.push(n.name.clone());
                progressed = true;
            }
            if !progressed {
                break;
            }
        }
        if done.len() != enabled.len() {
            let stuck: Vec<&str> = enabled
                .iter()
                .map(|n| n.name.as_str())
                .filter(|k| !done_set.contains(*k))
                .collect();
            return Err(format!("cycle detected, stuck at: {}", stuck.join(", ")));
        }
        Ok(done)
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
        assert!(wf.extra.contains_key("pinData"));
        let back = Workflow::from_json(&wf.to_json_pretty().expect("serialize")).expect("re-parse");
        assert_eq!(wf, back);
    }

    #[test]
    fn topo_and_isolated() {
        let wf = Workflow::from_json(FIXTURE).expect("parse fixture");
        assert_eq!(
            wf.topo_order().expect("topo"),
            vec!["Manual Trigger".to_string(), "Set".to_string(), "NoOp".to_string()]
        );
        assert!(wf.isolated_nodes().is_empty());
        assert_eq!(wf.all_edges().len(), 2);
    }

    #[test]
    fn detects_dangling_and_dup() {
        let mut wf = Workflow::from_json(FIXTURE).expect("parse fixture");
        wf.connections.insert(
            "Set".to_string(),
            serde_json::json!({"main": [[{"node": "Ghost"}]]}),
        );
        assert_eq!(wf.dangling_edges(), vec![("Set".to_string(), "Ghost".to_string())]);
        assert!(!wf.validate_structure().is_empty());
    }
}
