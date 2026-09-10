//! Note-binding stickyNote (M4) — index deterministik node kerja yang
//! bounding-box-nya ter-cover kotak stickyNote (AGENT4-WORKFLOW-ROSETTA v1.1
//! §4.4, mandat #547).
//!
//! Aturan:
//! - Geometri murni: node `{x,y}` ter-cover note `[nx,nx+w]×[ny,ny+h]`.
//!   width/height dari `parameters` (bukti korpus: 388/450 note punya
//!   width+height; n8n stickyNote params `{content,width,height,color}`).
//! - Node kerja TIDAK termasuk stickyNote itu sendiri (note tak dieksekusi);
//!   node stickyNote lain pun TIDAK di-binding (dekoratif).
//! - Deterministik: urutan node sumber; tanpa wall-clock/random (kontrak
//!   determinisme) — 2× hitung = identik (R-3).
//! - Ambigu (node ter-cover >1 note) → node masuk `ambiguous` pada SEMUA note
//!   tsb dan TIDAK di-covered mana pun — siap override manual (manifest
//!   `documentation.notes_binding` MENANG atas geometri, §4.4.3).
//! - Note tanpa width/height lengkap → `unmeasurable` (tidak men-cover; tidak
//!   menebak default ukuran — ZERO ASSUMPTION #93).

use crate::model::CanonNode;

/// Type stickyNote.
pub const STICKY_TYPE: &str = "n8n-nodes-base.stickyNote";

/// Rectanggel stickyNote dari posisi + parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteRect {
    /// Sisi kiri (x posisi).
    pub x: f64,
    /// Sisi atas (y posisi).
    pub y: f64,
    /// Lebar (parameters.width).
    pub w: f64,
    /// Tinggi (parameters.height).
    pub h: f64,
}

impl NoteRect {
    /// Apakah titik ter-cover rect (inklusif batas).
    pub fn covers(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }
}

/// Ekstrak rect stickyNote bila dimensi & posisi lengkap.
pub fn note_rect(n: &CanonNode) -> Option<NoteRect> {
    if n.type_full != STICKY_TYPE {
        return None;
    }
    let [x, y] = n.position?;
    let w = n.parameters.get("width")?.as_f64()?;
    let h = n.parameters.get("height")?.as_f64()?;
    if w < 0.0 || h < 0.0 {
        return None;
    }
    Some(NoteRect { x, y, w, h })
}

/// Binding satu note.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NoteBinding {
    /// Nama stickyNote.
    pub note_name: String,
    /// Node kerja yang ter-cover TEPAT oleh note ini (urutan sumber).
    pub covered: Vec<String>,
    /// Node yang ambigu (ter-cover note ini + note lain) — override manual.
    pub ambiguous: Vec<String>,
}

/// Laporan binding per workflow (R-3: coverage tercatat per template).
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BindingReport {
    /// Binding per stickyNote (urutan sumber).
    pub bindings: Vec<NoteBinding>,
    /// Nama stickyNote tanpa dimensi lengkap (tidak men-cover).
    pub unmeasurable: Vec<String>,
    /// Jumlah node kerja unik yang ter-cover.
    pub covered_node_total: usize,
    /// Jumlah node ambigu unik.
    pub ambiguous_node_total: usize,
}

/// Hitung binding untuk satu grafik node. Murni + deterministik (R-3/R-8).
pub fn bind_notes(nodes: &[CanonNode]) -> BindingReport {
    let mut report = BindingReport::default();

    // kumpulkan rect note + indeks (urutan sumber)
    let notes: Vec<(usize, NoteRect)> = nodes
        .iter()
        .enumerate()
        .filter_map(|(i, n)| note_rect(n).map(|r| (i, r)))
        .collect();

    // node yang tak ter-cover: stickyNote TIDAK di-binding (dekoratif);
    // node tanpa posisi tidak bisa dicover → diabaikan (bukan error)
    for &(ni, rect) in &notes {
        let mut b = NoteBinding {
            note_name: nodes[ni].name.clone(),
            ..Default::default()
        };
        // hitung semua node non-sticky berposisi dalam rect
        let mut inside: Vec<&CanonNode> = Vec::new();
        for n in nodes {
            if n.type_full == STICKY_TYPE {
                continue;
            }
            if let Some([x, y]) = n.position {
                if rect.covers(x, y) {
                    inside.push(n);
                }
            }
        }
        // pisahkan ambigu: node yang juga ter-cover note lain
        for n in inside {
            let p = n.position.expect("inside => berposisi");
            let also_other = notes
                .iter()
                .any(|&(oj, or)| oj != ni && or.covers(p[0], p[1]));
            if also_other {
                b.ambiguous.push(n.name.clone());
            } else {
                b.covered.push(n.name.clone());
            }
        }
        report.bindings.push(b);
    }

    // unmeasurable
    for n in nodes {
        if n.type_full == STICKY_TYPE && note_rect(n).is_none() {
            report.unmeasurable.push(n.name.clone());
        }
    }

    // total unik
    let mut covered_set = std::collections::BTreeSet::new();
    let mut ambig_set = std::collections::BTreeSet::new();
    for b in &report.bindings {
        for c in &b.covered {
            covered_set.insert(c.clone());
        }
        for a in &b.ambiguous {
            ambig_set.insert(a.clone());
        }
    }
    report.covered_node_total = covered_set.len();
    report.ambiguous_node_total = ambig_set.len();

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn node(name: &str, x: f64, y: f64) -> CanonNode {
        CanonNode {
            name: name.into(),
            type_full: "n8n-nodes-base.manualTrigger".into(),
            type_version: None,
            origin: crate::model::NodeOrigin::N8nBase,
            position: Some([x, y]),
            parameters: json!({}),
            extra: Default::default(),
        }
    }

    fn sticky(name: &str, x: f64, y: f64, w: f64, h: f64) -> CanonNode {
        CanonNode {
            name: name.into(),
            type_full: STICKY_TYPE.into(),
            type_version: None,
            origin: crate::model::NodeOrigin::N8nBase,
            position: Some([x, y]),
            parameters: json!({"content": "", "width": w, "height": h}),
            extra: Default::default(),
        }
    }

    #[test]
    fn cover_sederhana_dan_batas() {
        let nodes = vec![
            sticky("Note", 0.0, 0.0, 100.0, 100.0),
            node("A", 10.0, 10.0),   // dalam
            node("B", 100.0, 100.0), // batas inklusif
            node("C", 200.0, 200.0), // luar
            node("T", 50.0, 50.0),   // dalam; dummy type diganti di bawah
        ];
        let r = bind_notes(&nodes);
        assert_eq!(r.bindings.len(), 1);
        assert_eq!(r.bindings[0].covered, vec!["A", "B", "T"]);
    }

    #[test]
    fn ambigu_tidak_di_covered() {
        // dua note overlap; node X di area keduanya
        let mut n1 = sticky("N1", 0.0, 0.0, 100.0, 100.0);
        n1.parameters = json!({"content": "", "width": 100.0, "height": 100.0});
        let mut n2 = sticky("N2", 50.0, 50.0, 100.0, 100.0);
        n2.parameters = json!({"content": "", "width": 100.0, "height": 100.0});
        let nodes = vec![n1, n2, node("X", 60.0, 60.0), node("Y", 10.0, 10.0)];
        let r = bind_notes(&nodes);
        assert_eq!(r.bindings.len(), 2);
        // Y hanya di N1; X ambigu di keduanya
        assert_eq!(r.bindings[0].covered, vec!["Y"]);
        assert_eq!(r.bindings[0].ambiguous, vec!["X"]);
        assert!(r.bindings[1].covered.is_empty());
        assert_eq!(r.bindings[1].ambiguous, vec!["X"]);
        assert_eq!(r.covered_node_total, 1);
        assert_eq!(r.ambiguous_node_total, 1);
    }

    #[test]
    fn sticky_tidak_bind_sesama_note() {
        // note B di dalam note A → tidak ada binding antar note
        let nodes = vec![
            sticky("A", 0.0, 0.0, 500.0, 500.0),
            sticky("B", 10.0, 10.0, 100.0, 100.0),
        ];
        let r = bind_notes(&nodes);
        assert!(r.bindings[0].covered.is_empty());
        assert!(r.bindings[1].covered.is_empty());
    }

    #[test]
    fn unmeasurable_tidak_cover() {
        let mut n = sticky("Note", 0.0, 0.0, 100.0, 100.0);
        let nodes = vec![n.clone(), node("A", 10.0, 10.0)];
        assert_eq!(bind_notes(&nodes).bindings[0].covered, vec!["A"]);
        n.parameters = json!({"content": ""}); // tanpa width/height
        let r = bind_notes(&[n, node("A", 10.0, 10.0)]);
        assert!(r.bindings.is_empty());
        assert_eq!(r.unmeasurable, vec!["Note"]);
    }
}
