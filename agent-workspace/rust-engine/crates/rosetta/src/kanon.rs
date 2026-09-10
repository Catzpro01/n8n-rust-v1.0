//! Kanon normalisasi (M5 — bagian agent4 per pembagian #168): N-01, N-02,
//! N-03, N-04, N-05, N-07, N-11 + N-20. Tiap aturan = 1 fungsi bernama +
//! test (AGENT4-NORMALIZER-DESIGN.md §3). Aturan N-06/N-08/N-09/N-10/N-12/
//! N-21/N-22 = domain agent1/agent3 (execution harness) — di luar crate ini.
//!
//! Output: `Value` terkanon + digest sha256 (kanon dijamin deterministik;
//! digest memakai sha2 =0.10.8 — preseden RULING 7: crate di luar kernel
//! boleh depend sendiri).
//!
//! Versi aturan TIDAK dikunci di digest (framing u32-BE, formula
//! input_canon_key = sha256(norm_version ∥ norm_fn_id ∥ canon_fields ∥
//! payload) — #988/#990): versi tinggal DI DALAM kunci pemanggil, bukan
//! di sini.

use crate::model::Workflow;

/// Versi normalizer (bagian ini). Pemanggil memasukkannya ke kunci kanon.
pub const KANON_VERSION: u32 = 1;

/// Kunci root workflow yang dipertahankan (allowlist — N-01/N-20).
pub const ROOT_ALLOWLIST: &[&str] = &[
    "name",
    "nodes",
    "connections",
    "settings",
    "pinData",
    "meta",
];

// ---------------------------------------------------------------------------
// N-01/N-20 — strip kunci root non-n8n
// ---------------------------------------------------------------------------
/// Hanya pertahankan kunci root dalam allowlist (N-01 utk diff; N-20 utk
/// input tercemar). Kunci lain dihapus.
pub fn n01_strip_root_extra(wf: &Workflow) -> serde_json::Value {
    let mut root = serde_json::Map::new();
    if let Some(name) = &wf.name {
        root.insert("name".into(), serde_json::Value::String(name.clone()));
    }
    for k in ["settings", "pinData", "meta"] {
        if let Some(v) = wf.extra.get(k) {
            root.insert(k.into(), v.clone());
        }
    }
    root.insert("nodes".into(), serde_json::Value::Null); // diisi pemanggil
    root.insert("connections".into(), wf.connections.clone());
    serde_json::Value::Object(root)
}

// ---------------------------------------------------------------------------
// N-02 — hapus id internal node yang berubah antar run
// ---------------------------------------------------------------------------
fn n02_strip_node_id(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(n02_strip_node_id).collect())
        }
        serde_json::Value::Object(mut map) => {
            map.remove("id");
            for (_, v) in map.iter_mut() {
                if let serde_json::Value::Object(inner) = v {
                    inner.remove("id");
                }
            }
            // lanjut rekursif utk struktur lain
            for (_, v) in map.iter_mut() {
                *v = n02_strip_node_id(v.clone());
            }
            serde_json::Value::Object(map)
        }
        other => other,
    }
}

// ---------------------------------------------------------------------------
// N-07 — number: -0 → 0; float bulat → integer
// ---------------------------------------------------------------------------
fn n07_normalize_number(v: f64) -> serde_json::Value {
    if v == 0.0 {
        return serde_json::json!(0); // -0.0 → 0
    }
    if v.fract() == 0.0 && v.abs() < 9.007_199_254_740_992e15 {
        // float dgn nilai integer → integer (1.0 ≡ 1)
        return serde_json::json!(v as i64);
    }
    serde_json::json!(v)
}

fn n07_walk(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(n07_walk).collect())
        }
        serde_json::Value::Object(map) => {
            serde_json::Value::Object(map.into_iter().map(|(k, v)| (k, n07_walk(v))).collect())
        }
        serde_json::Value::Number(n) => n
            .as_f64()
            .map(n07_normalize_number)
            .unwrap_or(serde_json::Value::Number(n)),
        other => other,
    }
}

// ---------------------------------------------------------------------------
// N-11 — null vs missing: kunci bernilai null dihapus (setara absen)
// ---------------------------------------------------------------------------
fn n11_drop_null(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(n11_drop_null).collect())
        }
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, n11_drop_null(v)))
                .collect(),
        ),
        other => other,
    }
}

// ---------------------------------------------------------------------------
// N-03 — timestamp ISO-8601 → epoch ms (UTC)
// ---------------------------------------------------------------------------
/// Konversi tanggal Gregorian → hari sejak 1970-01-01 (algoritma
/// days-from-civil, Hinnant). Murni, tanpa dependensi.
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719_468
}

/// Parse offset zona waktu: `+07:00`, `+0700`, `-05:00`.
fn parse_offset(off: &str) -> Option<(i64, i64)> {
    if off.contains(':') {
        let mut p = off.splitn(2, ':');
        Some((p.next()?.parse().ok()?, p.next()?.parse().ok()?))
    } else if off.len() == 4 {
        Some((off[..2].parse().ok()?, off[2..].parse().ok()?))
    } else {
        None
    }
}

/// Normalisasi timestamp ISO-8601 (UTC/offset) → `EPOCH:<ms>` (N-03).
/// String bukan timestamp dibiarkan. Offset ±HH:MM / ±HHMM diperhitungkan;
/// tanpa offset dianggap UTC (Z). Presisi ms.
pub fn n03_iso_to_epoch(s: &str) -> String {
    // pola: date T time (Z | ±HH:MM | ±HHMM | kosong)
    let bytes = s.as_bytes();
    if bytes.len() < 19 || bytes[4] != b'-' || bytes[7] != b'-' {
        return s.to_string();
    }
    let (Ok(y), Ok(mo), Ok(d)) = (
        s[0..4].parse::<i64>(),
        s[5..7].parse::<i64>(),
        s[8..10].parse::<i64>(),
    ) else {
        return s.to_string();
    };
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return s.to_string();
    }
    if s.as_bytes().get(10) != Some(&b'T') {
        return s.to_string();
    }
    let time_part = &s[11..];

    // ambil HH:MM[:SS[.fff]] lalu offset
    let parse_time = |t: &str| -> Option<(i64, i64, i64, i64)> {
        // t = HH:MM[:SS[.fraction]]
        let (hms, frac) = match t.find('.') {
            Some(i) => (&t[..i], &t[i + 1..]),
            None => (t, ""),
        };
        let mut parts = hms.splitn(3, ':');
        let h: i64 = parts.next()?.parse().ok()?;
        let m: i64 = parts.next()?.parse().ok()?;
        let sec: i64 = match parts.next() {
            Some(x) => x.parse().ok()?,
            None => 0,
        };
        let ms: i64 = if frac.is_empty() {
            0
        } else {
            let mut f = frac.to_string();
            while f.len() < 3 {
                f.push('0');
            }
            f[..3].parse().unwrap_or(0)
        };
        if !(0..=23).contains(&h) || !(0..=59).contains(&m) || !(0..=60).contains(&sec) {
            return None;
        }
        Some((h, m, sec, ms))
    };

    // cari offset di akhir
    let (body, offset_ms): (&str, i64) = {
        let after = time_part;
        if after.ends_with('Z') {
            (after.strip_suffix('Z').unwrap_or(after), 0)
        } else if let Some(plus) = after.rfind('+') {
            let off = &after[plus + 1..];
            let Some((oh, om)) = parse_offset(off) else {
                return s.to_string();
            };
            if oh > 14 || om > 59 {
                return s.to_string();
            }
            (&after[..plus], (oh * 60 + om) * 60_000)
        } else if let Some(minus) = after.rfind('-') {
            let off = &after[minus + 1..];
            let Some((oh, om)) = parse_offset(off) else {
                return s.to_string();
            };
            if oh > 14 || om > 59 {
                return s.to_string();
            }
            (&after[..minus], -(oh * 60 + om) * 60_000)
        } else {
            (after, 0)
        }
    };

    let Some((h, m, sec, ms)) = parse_time(body) else {
        return s.to_string();
    };
    let epoch =
        days_from_civil(y, mo, d) * 86_400_000 + (h * 3600 + m * 60 + sec) * 1000 + ms - offset_ms;
    format!("EPOCH:{epoch}")
}

fn n03_walk(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(n03_walk).collect())
        }
        serde_json::Value::Object(map) => {
            serde_json::Value::Object(map.into_iter().map(|(k, v)| (k, n03_walk(v))).collect())
        }
        serde_json::Value::String(s) => serde_json::Value::String(n03_iso_to_epoch(&s)),
        other => other,
    }
}

// ---------------------------------------------------------------------------
// N-04 — $execution.id & resumeUrl → token deterministik
// ---------------------------------------------------------------------------
fn n04_tokenize(s: &str) -> String {
    s.replace("$execution.id", "{{EXEC_ID}}")
}

fn n04_walk(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(n04_walk).collect())
        }
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if k == "resumeUrl" {
                    out.insert(k, serde_json::Value::String("{{EXEC_ID}}".into()));
                } else {
                    out.insert(k, n04_walk(v));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::String(s) => serde_json::Value::String(n04_tokenize(&s)),
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Pipeline
// ---------------------------------------------------------------------------
/// Kanonikasi penuh urutan tetap: N-01/20 → N-02 → N-04 → N-03 → N-11 →
/// N-07 → (N-05 otomatis: serde_json Map = BTreeMap → serialize terurut).
/// Murni + deterministik (R-8).
pub fn canonicalize(wf: &Workflow) -> serde_json::Value {
    let mut root = n01_strip_root_extra(wf);
    if let serde_json::Value::Object(map) = &mut root {
        let nodes_out: Vec<serde_json::Value> = wf
            .nodes
            .iter()
            .map(|n| {
                let mut nm = serde_json::Map::new();
                nm.insert("name".into(), serde_json::Value::String(n.name.clone()));
                nm.insert(
                    "type".into(),
                    serde_json::Value::String(n.type_full.clone()),
                );
                nm.insert(
                    "typeVersion".into(),
                    n.type_version
                        .map(|tv| tv.to_json_value())
                        .unwrap_or_else(|| serde_json::json!(1)),
                );
                if let Some([x, y]) = n.position {
                    nm.insert("position".into(), serde_json::json!([x, y]));
                }
                nm.insert("parameters".into(), n.parameters.clone());
                for (k, v) in &n.extra {
                    nm.insert(k.clone(), v.clone());
                }
                nm.remove("id"); // N-02
                serde_json::Value::Object(nm)
            })
            .collect();
        // N-02 tambahan utk struktur tersarang (jaga-jaga)
        let nodes = n02_strip_node_id(serde_json::Value::Array(nodes_out));
        map.insert("nodes".into(), nodes);
    }
    // urutan tetap: N-04 token sebelum N-03 timestamp (resumeUrl berisi
    // $execution.id, hindari sisa token)
    let v = n04_walk(root);
    let v = n03_walk(v);
    let v = n11_drop_null(v);
    n07_walk(v)
}

/// Serialisasi kanonik deterministik (kunci terurut — serde_json Map).
pub fn canonical_json_bytes(wf: &Workflow) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&canonicalize(wf))
}

/// Digest sha256 kanon (sha2 =0.10.8; preseden RULING 7). Framing u32-BE
/// utk versi (C-02) — versi DI DALAM kunci, sesuai formula input_canon_key.
pub fn canonical_digest(wf: &Workflow) -> Result<[u8; 32], serde_json::Error> {
    use sha2::Digest;
    let bytes = canonical_json_bytes(wf)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(KANON_VERSION.to_be_bytes()); // framing u32-BE (C-02)
    hasher.update(&bytes);
    Ok(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_workflow_str;

    #[test]
    fn n03_epoch_dasar() {
        // 1970-01-01T00:00:00Z
        assert_eq!(n03_iso_to_epoch("1970-01-01T00:00:00.000Z"), "EPOCH:0");
        // 2026-09-09T04:00:00Z = 1788480000000 (hitungan manual, verifikasi)
        // 2026-09-09 04:00:00 UTC
        assert_eq!(
            n03_iso_to_epoch("2026-09-09T04:00:00Z"),
            "EPOCH:1788926400000"
        );
        // offset +07
        assert_eq!(
            n03_iso_to_epoch("2026-09-09T11:00:00+07:00"),
            "EPOCH:1788926400000"
        );
        // offset +0700 (tanpa titik dua)
        assert_eq!(
            n03_iso_to_epoch("2026-09-09T11:00:00+0700"),
            "EPOCH:1788926400000"
        );
        // tanpa offset → UTC
        assert_eq!(
            n03_iso_to_epoch("2026-09-09T04:00:00"),
            "EPOCH:1788926400000"
        );
        // bukan timestamp → dibiarkan
        assert_eq!(n03_iso_to_epoch("hello 2026-09-09"), "hello 2026-09-09");
    }

    #[test]
    fn n07_neg_zero_dan_float_bulat() {
        assert_eq!(n07_normalize_number(-0.0), serde_json::json!(0));
        assert_eq!(n07_normalize_number(0.0), serde_json::json!(0));
        assert_eq!(n07_normalize_number(1.0), serde_json::json!(1));
        assert_eq!(n07_normalize_number(1.5), serde_json::json!(1.5));
        let v = n07_walk(serde_json::json!({"a": -0.0, "b": [1.0], "c": 1.5}));
        assert_eq!(v, serde_json::json!({"a": 0, "b": [1], "c": 1.5}));
    }

    #[test]
    fn n11_null_hilang_n02_id_hilang() {
        let wf = parse_workflow_str(
            r#"{"nodes":[{"name":"N","type":"n8n-nodes-base.code","id":"abc","parameters":{"jsCode":"x","opt":null}}],"connections":{},"_corpus_meta":{"x":1}}"#,
            "t",
        )
        .unwrap();
        let c = canonicalize(&wf);
        let s = serde_json::to_string(&c).unwrap();
        assert!(!s.contains("\"id\""), "N-02: id harus hilang: {s}");
        assert!(!s.contains("_corpus_meta"), "N-01: meta harus hilang");
        assert!(!s.contains("\"opt\":null"), "N-11: null harus hilang");
    }

    #[test]
    fn n05_kunci_terurut() {
        let a = parse_workflow_str(
            r#"{"nodes":[{"name":"N","parameters":{"z":1,"a":2,"m":{"y":1,"b":2}},"type":"n8n-nodes-base.code"}],"connections":{}}"#,
            "t",
        )
        .unwrap();
        let s = canonical_json_bytes(&a).unwrap();
        let t = String::from_utf8(s).unwrap();
        // serde_json Map (BTreeMap) → terurut: a sebelum m sebelum z
        assert!(t.contains("\"a\":2"));
        let ia = t.find("\"a\"").unwrap();
        let im = t.find("\"m\"").unwrap();
        let iz = t.find("\"z\"").unwrap();
        assert!(ia < im && im < iz, "{t}");
    }

    #[test]
    fn n04_token() {
        let v = n04_walk(serde_json::json!({
            "a": "{{ $execution.id }}",
            "b": "${$execution.id}",
            "resumeUrl": "https://x/resume/abc"
        }));
        let s = serde_json::to_string(&v).unwrap();
        assert!(!s.contains("$execution.id"));
        assert!(s.contains("{{EXEC_ID}}"));
    }

    #[test]
    fn digest_dan_determinisme() {
        let wf = parse_workflow_str(
            r#"{"nodes":[{"name":"N","type":"n8n-nodes-base.code","parameters":{"jsCode":"x"}}],"connections":{}}"#,
            "t",
        )
        .unwrap();
        let d1 = canonical_digest(&wf).unwrap();
        let d2 = canonical_digest(&wf).unwrap();
        assert_eq!(d1, d2, "digest deterministik");
        let wf2 = parse_workflow_str(
            r#"{"nodes":[{"name":"N","type":"n8n-nodes-base.code","parameters":{"jsCode":"y"}}],"connections":{}}"#,
            "t",
        )
        .unwrap();
        assert_ne!(
            d1,
            canonical_digest(&wf2).unwrap(),
            "isi beda → digest beda"
        );
    }
}
