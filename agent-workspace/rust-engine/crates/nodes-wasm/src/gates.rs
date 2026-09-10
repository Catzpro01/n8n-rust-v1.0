//! Slice-2 gate logic (RFC #950 v0.3 §5) — lapis keputusan PURE + pemetaan tipe kernel.
//!
//! Dipisah dari lapis host (wasmtime) supaya bisa diuji tanpa runtime. Semua fungsi
//! deterministic & dependency-light. MV-1/MV-3 di sini: porsi statis (audit impor atas
//! daftar nyata; checksum sha256); eksekusi thd modul .wasm nyata ada di lapis host.
//!
//! Canonical-bound crate ini memakai tipe kernel via path-dep read-only `kernel`.

use kernel::item::Item;
use kernel::params::{ParameterKind, ParameterSchema};
use serde_json::Value;

use crate::manifest::{EgressPolicy, Manifest, ParamSpec, ParamType};

// ---------------------------------------------------------------------------
// MV-1 (statis): deny-by-default atas impor guest.
// ---------------------------------------------------------------------------

/// Audit impor nyata sebuah modul terhadap allowlist manifest.
/// Kembalikan daftar impor yang TIDAK di-allowlist (harus kosong utk lolos).
pub fn audit_imports(actual: &[(String, String)], allowed: &crate::manifest::ImportsAllowlist) -> Vec<(String, String)> {
    actual
        .iter()
        .filter(|(module, name)| {
            match module.as_str() {
                "env" => !allowed.env.iter().any(|n| n == name),
                "record" => !allowed.record.iter().any(|n| n == name),
                _ => true, // namespace tak dikenal => deny-by-default
            }
        })
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// MV-2: manifest params <-> ParameterSchema kernel cocok (NC-2 #961).
// ---------------------------------------------------------------------------

fn kernel_kind(t: ParamType) -> ParameterKind {
    match t {
        ParamType::String => ParameterKind::String,
        ParamType::Integer => ParameterKind::Number,
        ParamType::Boolean => ParameterKind::Boolean,
    }
}

/// Periksa konformansi setiap ParamSpec manifest thd schema kernel.
/// Arah: nama nyata = field schema kernel (atau `maps_to`); tidak ada vocab paralel.
pub fn params_conform(schema: &ParameterSchema, params: &[ParamSpec]) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();

    for p in params {
        let field = schema
            .lookup(&p.name)
            .or_else(|| p.maps_to.as_deref().and_then(|m| schema.lookup(m)));

        let f = match field {
            Some(f) => f,
            None => {
                errs.push(format!("manifest param '{}' has no kernel schema field (name or maps_to)", p.name));
                continue;
            }
        };

        let expected = kernel_kind(p.ty);
        let ok = match expected {
            ParameterKind::String => matches!(f.kind, ParameterKind::String | ParameterKind::Text),
            ParameterKind::Number => matches!(f.kind, ParameterKind::Number),
            ParameterKind::Boolean => matches!(f.kind, ParameterKind::Boolean),
            _ => false,
        };
        if !ok {
            errs.push(format!(
                "manifest param '{}' type {:?} != kernel field '{}' kind {:?}",
                p.name, p.ty, f.name, f.kind
            ));
        }
        if matches!(expected, ParameterKind::Number) {
            if let (Some(min), Some(max)) = (p.min, p.max) {
                if min > max {
                    errs.push(format!("manifest param '{}' min>max", p.name));
                }
            }
        }
    }

    // Arah balik: field kernel yang wajib & terlihat harus punya manifest param.
    for f in &schema.fields {
        if f.required && f.display_if.is_none() {
            let in_manifest = params
                .iter()
                .any(|p| p.name == f.name || p.maps_to.as_deref() == Some(f.name.as_str()));
            if !in_manifest {
                errs.push(format!(
                    "kernel schema requires field '{}' but manifest omits it",
                    f.name
                ));
            }
        }
    }

    if errs.is_empty() { Ok(()) } else { Err(errs) }
}

// ---------------------------------------------------------------------------
// MV-4: egress deny-by-default. allowlist_hosts kosong => nol jaringan.
// ---------------------------------------------------------------------------

/// Evaluasi satu permintaan egress. `host` = hostname RESOLVED (resolve-then-check
/// jalur terpisah, agent9 Q-2 #1031 — bukan tanggung jawab fungsi ini).
pub fn egress_allowed(scheme: &str, host: &str, p: &EgressPolicy) -> Result<(), String> {
    match p.default {
        crate::manifest::DenyMode::Allow => return Ok(()), // hanya jika manifest tegas allow
        crate::manifest::DenyMode::Deny => {}
    }
    if !p.allow_schemes.iter().any(|s| s == scheme) {
        return Err(format!("scheme '{scheme}' not in allow_schemes"));
    }
    if !p.allowlist_hosts.iter().any(|h| h == host) {
        return Err(format!("host '{host}' not in allowlist_hosts (deny-by-default)"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// MV-5: wall-clock TIDAK boleh masuk Item/digest (agent9 R-2 #933, RFC #950).
// ---------------------------------------------------------------------------

/// Kunci JSON level-1 yang dicadangkan kernel/witness dan TIDAK boleh lolos sbg
/// isi output (Item) — wall-clock/durasi observasi non-deterministik.
pub const BANNED_TIMING_KEYS: &[&str] = &[
    "took_ms", "duration_ms", "wall_clock", "wall_clock_ms", "started_ms",
    "ended_ms", "elapsed_ms", "request_ms", "response_ms",
];

/// Deteksi field wall-clock di dalam Item (sebelum masuk digest / spill).
/// Kembalikan daftar kunci bermasalah; kosong = bersih.
pub fn wallclock_fields_in_item(item: &Item) -> Vec<String> {
    let mut hits = Vec::new();
    if let Value::Object(map) = &item.json {
        for key in map.keys() {
            let lk = key.to_ascii_lowercase();
            if BANNED_TIMING_KEYS.iter().any(|b| lk.contains(b)) {
                hits.push(key.clone());
            }
        }
    }
    hits
}

// ---------------------------------------------------------------------------
// MV-6: body_window 256 KiB (#922 §B) -> spill + truncated.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BodyWindowPlan {
    pub body_bytes: u64,
    pub window_bytes: u64,
    /// true bila body melebihi window -> materialisasi SPILL (SpilledList) 
    pub spill: bool,
    /// true bila isi terpotong dari sudut guest (guest lihat window saja; DR-8).
    pub truncated: bool,
    pub bytes_in_window: u64,
    pub bytes_out_of_window: u64,
}

/// Keputusan murni windowing; hasil `spill:true` memilih cabang ItemList::Spilled.
pub fn plan_body_window(body_bytes: u64, window_bytes: u64) -> BodyWindowPlan {
    assert!(window_bytes > 0, "body_window_bytes harus > 0");
    if body_bytes <= window_bytes {
        BodyWindowPlan {
            body_bytes,
            window_bytes,
            spill: false,
            truncated: false,
            bytes_in_window: body_bytes,
            bytes_out_of_window: 0,
        }
    } else {
        BodyWindowPlan {
            body_bytes,
            window_bytes,
            spill: true,
            truncated: true,
            bytes_in_window: window_bytes,
            bytes_out_of_window: body_bytes - window_bytes,
        }
    }
}

/// Peta hasil windowing ke varian kernel (tipe NYATA): Inline(..) vs Spilled(..).
pub fn kernel_item_list_shape(plan: &BodyWindowPlan) -> &'static str {
    if plan.spill {
        "ItemList::Spilled(SpilledList{path: SpillPath, len, total_bytes, codec})"
    } else {
        "ItemList::Inline(Vec<Item>)"
    }
}

// ---------------------------------------------------------------------------
// MV-7: ReplayRecord membawa seq/attempt utk korelasi timeline (Mitra-C v0.3).
// ---------------------------------------------------------------------------

/// Struktur replay (kontrak AGENT4-AGENT6-DETERM-REPLAY-INTERFACE v0.3).
/// node_id IMPLISIT (capability-binding instance) — bukan param, bukan uuid.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplayRecord {
    /// digest kanonik input host-side (sha256) sebelum guest (RFC #950 §2).
    pub input_canon_sha256: String,
    /// nomor urut eksekusi utk korelasi timeline (host-log seam exec.retry).
    pub seq: u64,
    /// attempt saat ini (otoritas task.attempt engine — satu sumber penomoran).
    pub attempt: u64,
}

impl ReplayRecord {
    pub fn new(input_canon_sha256: impl Into<String>, seq: u64, attempt: u64) -> Self {
        Self { input_canon_sha256: input_canon_sha256.into(), seq, attempt }
    }

    /// Kunci korelasi ke baris timeline.
    pub fn key(&self) -> (u64, u64) {
        (self.seq, self.attempt)
    }
}

/// MV-7: sebuah catatan replay valid hanya bila seq & attempt konsisten dgn konteks.
pub fn replay_consistent(rec: &ReplayRecord, expected_seq: u64, expected_attempt: u64) -> bool {
    rec.seq == expected_seq && rec.attempt == expected_attempt
}

// ---------------------------------------------------------------------------
// MV-3: checksum berkas modul (SHA-256) vs manifest.
// ---------------------------------------------------------------------------

/// sha256 hex — dipakai utk MV-3 & input_digest. (sha2 =0.10.8 preseden Ruling 7;
/// utk crate KANONIK menunggu persetujuan dep @matt — shadow/dev diaktifkan.)
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    out.iter().map(|b| format!("{b:02x}")).collect()
}

/// Verifikasi identitas tipe node: sha256 BERKAS fisik == nilai manifest.
pub fn module_checksum_ok(manifest: &Manifest, module_bytes: &[u8]) -> bool {
    sha256_hex(module_bytes) == manifest.module_sha256
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// RULING 39b (#1317) / RULING 41 (#1345) / RULING 47b (#1420):
// input eksternal tak-tepercaya -> IKAT KEDALAMAN, fail-closed SEBELUM serde.
// Pemeriksa kedalaman JSON pasca-landing (commit 5bc8ea8) ada SATU di kernel;
// crate ini TIDAK memuat scanner/wrapper (nol implementasi kedua) — hanya
// konstanta kebijakan konsumen + delegasi langsung ke kernel::json_depth_*.
// ---------------------------------------------------------------------------

/// Batas kedalaman artefak luar jalur WCB (kebijakan KONSUMEN; jauh di atas
/// kebutuhan nyata, jauh di bawah ambang stack 2 MiB).
pub const MAX_JSON_DEPTH: u32 = 64;

/// Cek kedalaman manifest/params/body eksternal (delegasi kernel, view bool).
pub fn untrusted_json_ok(bytes: &[u8]) -> bool {
    kernel::json_depth_within(bytes, MAX_JSON_DEPTH)
}

// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use kernel::params::ParameterField;
    use serde_json::json;

    #[test]
    fn mv1_audit_denies_unknown_import_and_namespace() {
        let allowed = crate::manifest::ImportsAllowlist {
            env: vec!["wcb_net_request".into()],
            record: vec![],
        };
        let actual = vec![
            ("env".to_string(), "wcb_net_request".to_string()),
            ("env".to_string(), "wasi_snapshot_preview1".to_string()),
            ("record".to_string(), "lookup".to_string()),
            ("mystery".to_string(), "x".to_string()),
        ];
        let bad = audit_imports(&actual, &allowed);
        assert_eq!(bad.len(), 3);
        assert!(!bad.iter().any(|(m, n)| m == "env" && n == "wcb_net_request"));
    }

    fn sample_schema() -> ParameterSchema {
        ParameterSchema::empty()
            .field(ParameterField::new("url", "URL", ParameterKind::String).required())
            .field(ParameterField::new("mode", "Mode", ParameterKind::String))
            .field(ParameterField::new("concurrency", "Concurrency", ParameterKind::Number))
            .field(ParameterField::new("count", "Count", ParameterKind::Number).required())
    }

    #[test]
    fn mv2_conform_ok() {
        let params = vec![
            ParamSpec { name: "url".into(), ty: ParamType::String, required: true, default: None, options: None, min: None, max: None, maps_to: None },
            ParamSpec { name: "mode".into(), ty: ParamType::String, required: false, default: Some(json!("smart")), options: None, min: None, max: None, maps_to: None },
            ParamSpec { name: "concurrency".into(), ty: ParamType::Integer, required: false, default: None, options: None, min: Some(1), max: Some(5), maps_to: None },
            ParamSpec { name: "c".into(), ty: ParamType::Integer, required: true, default: None, options: None, min: None, max: None, maps_to: Some("count".into()) },
        ];
        assert!(params_conform(&sample_schema(), &params).is_ok());
    }

    #[test]
    fn mv2_reports_type_mismatch_and_missing_required() {
        let params = vec![
            ParamSpec { name: "url".into(), ty: ParamType::Integer, required: true, default: None, options: None, min: None, max: None, maps_to: None },
        ];
        let errs = params_conform(&sample_schema(), &params).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("url")));
        assert!(errs.iter().any(|e| e.contains("count"))); // required schema field omitted
    }

    #[test]
    fn mv4_empty_allowlist_is_zero_network() {
        let p = EgressPolicy {
            default: crate::manifest::DenyMode::Deny,
            allowlist_hosts: vec![],
            allow_schemes: vec!["http".into(), "https".into()],
        };
        assert!(egress_allowed("https", "example.com", &p).is_err());
        assert!(egress_allowed("file", "example.com", &p).is_err());
    }

    #[test]
    fn mv4_allowlisted_host_ok_scheme_gate_still_applies() {
        let p = EgressPolicy {
            default: crate::manifest::DenyMode::Deny,
            allowlist_hosts: vec!["api.example.com".into()],
            allow_schemes: vec!["https".into()],
        };
        assert!(egress_allowed("https", "api.example.com", &p).is_ok());
        assert!(egress_allowed("http", "api.example.com", &p).is_err()); // scheme bukan allowlist
        assert!(egress_allowed("https", "evil.com", &p).is_err());
    }

    #[test]
    fn mv5_flags_wallclock_in_item_json() {
        let clean = Item::new(json!({ "content": "ok" }));
        assert!(wallclock_fields_in_item(&clean).is_empty());

        let dirty = Item::new(json!({ "text": "x", "duration_ms": 42, "took_ms": 7, "ok": 1 }));
        let hits = wallclock_fields_in_item(&dirty);
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn mv6_window_plan_inline_and_spill() {
        let w = 262_144u64;
        let small = plan_body_window(100_000, w);
        assert!(!small.spill && !small.truncated && small.bytes_out_of_window == 0);

        let big = plan_body_window(300_000, w);
        assert!(big.spill && big.truncated);
        assert_eq!(big.bytes_in_window, 262_144);
        assert_eq!(big.bytes_out_of_window, 300_000 - 262_144);
        assert!(kernel_item_list_shape(&big).starts_with("ItemList::Spilled"));
        assert!(kernel_item_list_shape(&small).starts_with("ItemList::Inline"));
    }

    #[test]
    fn mv7_replay_record_carries_seq_attempt() {
        let r = ReplayRecord::new("abc", 3, 2);
        assert_eq!(r.key(), (3, 2));
        assert!(replay_consistent(&r, 3, 2));
        assert!(!replay_consistent(&r, 3, 3));
    }

    #[test]
    fn mv3_sha256_matches_manifest_identity() {
        // digest known: sha256("") 
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let mut m = crate::manifest::tests::example_manifest();
        m.module_sha256 = sha256_hex(b""); // samakan identitas dgn berkas uji kosong
        assert!(module_checksum_ok(&m, b""));
        m.module_sha256 = "00".repeat(32);
        assert!(!module_checksum_ok(&m, b""));
    }

    #[test]
    fn r39b_deep_untrusted_json_rejected_but_normal_ok() {
        // Sumber guard = kernel (R41, commit 5bc8ea8). Delegasi ini menguji jalur
        // KONSUMEN: kalau pemanggil lupa memanggil guard, test ini merah.
        // normal (manifest contoh) -> ok
        let m = crate::manifest::tests::example_manifest();
        let ok_bytes = serde_json::to_vec(&m).unwrap();
        assert!(untrusted_json_ok(&ok_bytes));

        // kurung dalam string + string ter-escape tidak menambah kedalaman
        let tricky = br#"{"a":"{","b":"[}\\]\"","c":[1,2,{"d":"x"}]}"#;
        assert!(untrusted_json_ok(tricky));

        // struktur bersarang > max -> tolak (fail-closed via kernel)
        let mut deep = String::new();
        for _ in 0..(MAX_JSON_DEPTH + 10) {
            deep.push('[');
        }
        deep.push('0');
        for _ in 0..(MAX_JSON_DEPTH + 10) {
            deep.push(']');
        }
        assert!(!untrusted_json_ok(deep.as_bytes()));

        // F1 (#1330): error jalur parse menyebut kedalaman (DepthExceeded{max,found})
        // diverifikasi di manifest::parse_manifest. Kernel menjamin found = kedalaman
        // pertama yang melewati batas (test kernel boundary 63/64/65).
        assert_eq!(
            kernel::json_depth_exceeded(deep.as_bytes(), MAX_JSON_DEPTH),
            Some(MAX_JSON_DEPTH + 1)
        );
    }
}