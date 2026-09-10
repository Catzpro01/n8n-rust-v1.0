//! Gate R-1/R-2/R-8 atas korpus nyata 171 template (tests/fixtures — salinan
//! byte-identik dari docs/corpus; baseline sha256: 8970ba3c…1128).
//!
//! R-1: parse 171/171 tanpa gagal; 0 file termutasi (hash utuh sblm/sesudah).
//! R-2: node komunitas/unknown tetap masuk grafik + dicatat opaque.
//! R-8: 2× parse input sama = output identik.

use std::path::PathBuf;

use rosetta::model::NodeOrigin;
use rosetta::parse::{parse_workflow_bytes, parse_workflow_path};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn corpus_files() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(fixtures_dir())
        .expect("tests/fixtures ada")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .collect();
    v.sort();
    v
}

#[test]
fn r1_parse_171_171_nol_mutasi() {
    let files = corpus_files();
    assert_eq!(
        files.len(),
        171,
        "korpus fixtures harus 171 file JSON (baseline 1128)"
    );

    let mut ok = 0usize;
    let mut failures = Vec::new();
    for f in &files {
        let label = f.file_name().unwrap().to_string_lossy().to_string();
        let before = std::fs::read(f).unwrap_or_else(|e| panic!("read {label}: {e}"));
        match parse_workflow_bytes(&before, &label) {
            Ok(w) => {
                assert!(
                    !w.nodes.is_empty(),
                    "{label}: workflow tanpa node — mencurigakan"
                );
                ok += 1;
            }
            Err(e) => failures.push(format!("{label}: {e}")),
        }
        // R-1: parser murni baca — file tidak boleh berubah
        let after = std::fs::read(f).expect("read ulang");
        assert_eq!(before, after, "MUTASI TERDETEKSI pada {label} (R-1)");
    }

    assert!(
        failures.is_empty(),
        "gagal parse {} file:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert_eq!(ok, 171, "harus 171/171 parse sukses");
}

#[test]
fn r1_typeversion_korpus_tak_ada_desimal_dua_digit() {
    // GATE PENGUKURAN #1188 (P1 matt): klaim "minor n8n 0..=9" kini
    // DIPAKSA sebagai ujian berdiri di atas korpus, bukan laporan sekali
    // jalan. Scan TEKS MENTAH (bukan nilai f64 — 4.10 tak bisa dibedakan
    // dari 4.1 setelah parse JSON): kalau upstream suatu hari menulis
    // "typeVersion": 1.12 / 4.10, file ini GAGAL dengan baris yang
    // terlihat — persis yang diminta matt (kegagalan terlihat, bukan
    // peleburan senyap di digest kanonik). Tanpa dependensi regex.
    let mut hits: Vec<String> = Vec::new();
    for f in corpus_files() {
        let label = f.file_name().unwrap().to_string_lossy().to_string();
        let txt = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("read {label}: {e}"));
        let bytes = txt.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            // cari key "typeVersion"
            if bytes[i..].starts_with(br#""typeVersion""#) {
                let mut j = i + br#""typeVersion""#.len();
                // lewati spasi, ':', spasi, kutip opsional
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b':' {
                    j += 1;
                }
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                let quoted = j < bytes.len() && bytes[j] == b'"';
                if quoted {
                    j += 1;
                }
                // baca token sampai koma/brace/kutip
                let start = j;
                while j < bytes.len()
                    && !matches!(bytes[j], b',' | b'}' | b']' | b'"' | b'\n' | b'\r')
                {
                    j += 1;
                }
                let tok = &txt[start..j];
                if let Some((_int, frac)) = tok.split_once('.') {
                    if frac.len() > 1 {
                        hits.push(format!(
                            "{label}: typeVersion \"{tok}\" pecahan {} digit",
                            frac.len()
                        ));
                    }
                }
                i = j;
            }
            i += 1;
        }
    }
    assert!(
        hits.is_empty(),
        "{} typeVersion desimal >=2 digit di korpus (guard minor<10 #1176, P1 #1188):\n  {}",
        hits.len(),
        hits.join("\n  ")
    );
}

#[test]
fn r2_semua_node_masuk_grafik_opaque_dicatat() {
    for f in corpus_files() {
        let label = f.file_name().unwrap().to_string_lossy().to_string();
        let w = parse_workflow_path(&f).expect(&label);

        // hitung komunitas/unknown per file dari sumber asli via node2
        let json: serde_json::Value = serde_json::from_slice(&std::fs::read(&f).unwrap()).unwrap();
        let mut expect_opaque = 0usize;
        if let Some(arr) = json.get("nodes").and_then(|n| n.as_array()) {
            for raw in arr {
                let t = raw.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let origin = rosetta::catalog::origin_of(t);
                if origin.is_opaque() {
                    expect_opaque += 1;
                }
            }
        }

        assert_eq!(
            w.opaque_count(),
            expect_opaque,
            "{label}: jumlah catatan opaque tidak cocok hitungan sumber"
        );

        // tiap node sumber harus ada di grafik dgn type persis
        let arr = json.get("nodes").and_then(|n| n.as_array()).unwrap();
        assert_eq!(w.node_count(), arr.len(), "{label}: jumlah node berubah");
        for (i, raw) in arr.iter().enumerate() {
            let t = raw.get("type").and_then(|v| v.as_str()).unwrap();
            let name = raw.get("name").and_then(|v| v.as_str()).unwrap();
            let node = w
                .nodes
                .iter()
                .find(|n| n.name == name)
                .unwrap_or_else(|| panic!("{label}: node ke-{i} `{name}` hilang"));
            assert_eq!(node.type_full, t, "{label}: type {name} berubah");
            let expect_opaque = rosetta::catalog::origin_of(t).is_opaque();
            assert_eq!(
                node.origin.is_opaque(),
                expect_opaque,
                "{label}: klasifikasi {name} salah"
            );
            // parameters preserved byte-identik (tidak dinormalisasi di M1)
            let src_params = raw
                .get("parameters")
                .cloned()
                .unwrap_or_else(|| serde_json::Value::Object(Default::default()));
            assert_eq!(
                node.parameters, src_params,
                "{label}: parameters {name} berubah"
            );
        }
    }
}

#[test]
fn r8_determinisme_korpus() {
    // 2× parse dari DISK (jalur nyata) pada sampel penuh — identik
    for f in corpus_files() {
        let label = f.file_name().unwrap().to_string_lossy().to_string();
        let a = parse_workflow_path(&f).unwrap_or_else(|e| panic!("{label}: {e}"));
        let b = parse_workflow_path(&f).unwrap_or_else(|e| panic!("{label}: {e}"));
        assert_eq!(a, b, "R-8 gagal pada {label}");
    }
}

#[test]
fn statistik_korpus_tercatat() {
    // statistik penting utk laporan; asersi longgar (bukan gate keras),
    // supaya korpus bertambah tanpa mematahkan test ini.
    let mut nodes = 0usize;
    let mut sticky = 0usize;
    let mut community = 0usize;
    for f in corpus_files() {
        let w = parse_workflow_path(&f).unwrap();
        nodes += w.node_count();
        for n in &w.nodes {
            if n.type_full == "n8n-nodes-base.stickyNote" {
                sticky += 1;
            }
            if n.origin == NodeOrigin::Community {
                community += 1;
            }
        }
    }
    eprintln!("korpus: {nodes} node total; {sticky} stickyNote; {community} komunitas");
    assert!(
        nodes > 2000,
        "korpus harus >2000 node total (aktual {nodes})"
    );
}
