//! M6 — manifest node WCB-guest (RFC AGENT6-WCB-NODE-MANIFEST v0.3 e2e2587e,
//! [CONSENSUS-REACHED]): model JSON + validasi struktur (MV-1) + type-check
//! parameter node thd manifest.params (MV-2, arah Rosetta).
//!
//! Prinsip:
//! - Nol vocab tipe paralel: manifest dipetakan ke tipe kernel NYATA di tepi
//!   integrasi (ParameterSchema/NodeDescriptor — agent1/agent6); crate ini
//!   hanya membaca & memvalidasi kontrak JSON v0.3.
//! - `module_sha256` = identitas tipe node + checksum berkas (NC-1); modul
//!   berubah → versi node baru (W4 §10 silent-swap).
//! - `schema_version` >1 → ditolak (policy invalidate-on-upgrade, #958).
//! - Node komunitas tanpa manifest tetap OPAQUE (R-2) — manifest TIDAK
//!   pernah menebak.
//! - Field `n8n_type` (opsional, NON-RFC) = usul jembatan resolver
//!   workflow-level utk amend RFC v0.4 (agent6): node workflow → manifest.
//!   Tanpa jembatan, manifest tetap valid & terpakai runtime.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Type node manifest (RFC v0.3).
pub const MANIFEST_NODE_KIND: &str = "wcb-guest";
/// Schema version yang diterima.
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Model (per contoh JSON RFC v0.3 — field opsional diberi default)
// ---------------------------------------------------------------------------

/// Tipe nilai param manifest (subset yang dipakai type-check Rosetta).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WcbParamType {
    /// string
    String,
    /// integer
    Integer,
    /// number (float)
    Number,
    /// boolean
    Boolean,
    /// json (objek/array bebas)
    Json,
}

/// Satu parameter deklaratif node WCB.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WcbParam {
    /// Nama parameter (kunci di `parameters`).
    pub name: String,
    /// Tipe nilai.
    #[serde(rename = "type")]
    pub ty: WcbParamType,
    /// Wajib diisi.
    #[serde(default)]
    pub required: bool,
    /// Nilai default.
    #[serde(default)]
    pub default: Option<Value>,
    /// maps_to: nama field ParameterSchema/NodeDescriptor kernel (NC-2) —
    /// dipertahankan sebagai string, interpretasi = di tepi integrasi.
    #[serde(default)]
    pub maps_to: Option<String>,
    /// Enumerasi nilai valid (type-check).
    #[serde(default)]
    pub options: Vec<Value>,
    /// Batas bawah (integer/number).
    #[serde(default)]
    pub min: Option<f64>,
    /// Batas atas (integer/number).
    #[serde(default)]
    pub max: Option<f64>,
}

/// ABI (ekspor guest + batas memori).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AbiSpec {
    /// Nama fungsi yang diekspor.
    #[serde(default)]
    pub exports: Vec<String>,
    /// Halaman memori maksimum (64KiB/page).
    #[serde(default)]
    pub memory_max_pages: Option<u32>,
}

/// Allowlist impor guest (deny-by-default).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ImportSpec {
    /// namespace env (mis. wcb_net_request).
    #[serde(default)]
    pub env: Vec<String>,
    /// namespace record — OPSIONAL (NC-3): pure-compute cukup kosong.
    #[serde(default)]
    pub record: Vec<String>,
}

/// Deskripsi output (referensi kernel, interpretasi di tepi).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OutputSpec {
    /// kernel Item (item.rs).
    #[serde(default)]
    pub item_shape: Option<String>,
    /// ItemList::Inline|Spilled.
    #[serde(default)]
    pub item_list: Option<String>,
}

/// Kebijakan egress (deny-by-default).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EgressPolicy {
    /// default: deny.
    #[serde(default = "default_deny")]
    pub default: String,
    /// Allowlist host.
    #[serde(default)]
    pub allowlist_hosts: Vec<String>,
    /// Skema yang diizinkan.
    #[serde(default)]
    pub allow_schemes: Vec<String>,
}

fn default_deny() -> String {
    "deny".into()
}

/// Guard eksekusi.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GuardsSpec {
    /// > window → spill + truncated (konsensus #922 §B; DR-8: 262144).
    #[serde(default)]
    pub body_window_bytes: Option<u64>,
    /// polite delay [min,max].
    #[serde(default)]
    pub polite_delay_ms: Option<[u64; 2]>,
    /// enforce|log.
    #[serde(default)]
    pub robots: Option<String>,
    /// pii_redaction.
    #[serde(default)]
    pub pii_redaction: Option<String>,
}

/// Manifest WCB-guest lengkap (RFC v0.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WcbManifest {
    /// Skema versi (1 sekarang; >1 ditolak).
    pub schema_version: u32,
    /// "wcb-guest".
    pub node_kind: String,
    /// 64-hex sha256 berkas .wasm — identitas tipe node (NC-1).
    pub module_sha256: String,
    /// 64-hex blake3 modul (integritas; SHA-256 vs BLAKE3 dua peran).
    #[serde(default)]
    pub module_digest_blake3: Option<String>,
    /// ABI.
    #[serde(default)]
    pub abi: AbiSpec,
    /// Impor (allowlist).
    #[serde(default)]
    pub imports: ImportSpec,
    /// Granularity.
    #[serde(default)]
    pub granularity: Vec<String>,
    /// Parameter deklaratif.
    #[serde(default)]
    pub params: Vec<WcbParam>,
    /// Output.
    #[serde(default)]
    pub output: OutputSpec,
    /// Egress.
    #[serde(default)]
    pub egress_policy: EgressPolicy,
    /// Guard.
    #[serde(default)]
    pub guards: GuardsSpec,
    /// Determinisme (dokumentatif di sisi Rosetta).
    #[serde(default)]
    pub determinism: serde_json::Map<String, Value>,
    /// USUL JEMBATAN RESOLVER (amend RFC v0.4 — disetujui agent6 #1140):
    /// type n8n penuh node komunitas yang dilayani manifest ini (mis.
    /// "@blotato/n8n-nodes-blotato.blotato").
    /// Syarat (agent6): (i) opsional & plain identifier non-kosong — BUKAN
    /// sumbu identitas kedua (module_sha256 tetap identitas tipe node;
    /// n8n_type hanya utk resolver workflow→manifest, TIDAK utk digest/
    /// dedup); (ii) konsumsi di Rosetta/Hub, bukan di kernel; (iii)
    /// one-to-many boleh (1 manifest utk >1 alias tipe) asal resolver
    /// deterministik — ditulis di spec v0.4.
    #[serde(default, rename = "n8n_type")]
    pub n8n_type: Option<String>,
}

// ---------------------------------------------------------------------------
// Parse & validasi
// ---------------------------------------------------------------------------

/// Parse manifest dari JSON.
pub fn parse_manifest(s: &str) -> Result<WcbManifest, String> {
    // Guard kedalaman di BATAS API crate (RULING 50b/50c) — fail-closed sebelum
    // serde_json; kernel::json_depth_exceeded = SATU lintasan (R41/47b).
    if let Some(found) = kernel::json_depth_exceeded(s.as_bytes(), crate::parse::MAX_JSON_DEPTH) {
        return Err(format!(
            "manifest kedalaman JSON {found} melebihi batas {} (RULING 39b/50b; fail-closed)",
            crate::parse::MAX_JSON_DEPTH
        ));
    }
    let m: WcbManifest =
        serde_json::from_str(s).map_err(|e| format!("manifest bukan JSON valid: {e}"))?;
    Ok(m)
}

/// Validasi struktur (MV-1 arah Rosetta). Mengembalikan daftar masalah
/// (kosong = valid).
pub fn validate_manifest(m: &WcbManifest) -> Vec<String> {
    let mut errs = Vec::new();

    if m.schema_version != MANIFEST_SCHEMA_VERSION {
        errs.push(format!(
            "schema_version {} tidak didukung (harus {}) — policy invalidate-on-upgrade",
            m.schema_version, MANIFEST_SCHEMA_VERSION
        ));
    }
    if m.node_kind != MANIFEST_NODE_KIND {
        errs.push(format!("node_kind `{}` ≠ `wcb-guest`", m.node_kind));
    }
    let sha = m.module_sha256.trim();
    if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        errs.push("module_sha256 harus 64 hex digit (SHA-256 berkas fisik)".into());
    }
    if let Some(b) = &m.module_digest_blake3 {
        let b = b.trim();
        if b.len() != 64 || !b.chars().all(|c| c.is_ascii_hexdigit()) {
            errs.push("module_digest_blake3 harus 64 hex digit (BLAKE3-256)".into());
        }
    }
    if m.egress_policy.default != "deny" {
        errs.push("egress_policy.default harus \"deny\" (deny-by-default, SEC-WCB)".into());
    }
    if let Some(w) = m.guards.body_window_bytes {
        if w == 0 {
            errs.push("guards.body_window_bytes harus >0".into());
        }
    }
    if let Some(t) = &m.n8n_type {
        if t.trim().is_empty() {
            errs.push(
                "n8n_type (jika ada) harus identifier non-kosong (syarat amend v0.4 (i))".into(),
            );
        }
    }
    for p in &m.params {
        if p.name.trim().is_empty() {
            errs.push("ada param tanpa `name`".into());
        }
        if p.required && p.default.is_some() {
            errs.push(format!(
                "param `{}`: required + default kontradiktif",
                p.name
            ));
        }
        if !p.options.is_empty() {
            for o in &p.options {
                if !p.ty.accepts(o) {
                    errs.push(format!(
                        "param `{}`: option {o:?} tidak cocok tipe {}",
                        p.name,
                        p.ty.type_name()
                    ));
                }
            }
        }
        if (p.min.is_some() || p.max.is_some())
            && !matches!(p.ty, WcbParamType::Integer | WcbParamType::Number)
        {
            errs.push(format!(
                "param `{}`: min/max hanya utk integer/number",
                p.name
            ));
        }
    }
    errs
}

impl WcbParamType {
    /// Nama tipe utk pesan.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Json => "json",
        }
    }

    /// Apakah nilai JSON diterima tipe ini.
    pub fn accepts(&self, v: &Value) -> bool {
        match self {
            Self::String => v.is_string(),
            Self::Integer => v.is_i64() || v.is_u64(),
            Self::Number => v.is_f64() || v.is_i64() || v.is_u64(),
            Self::Boolean => v.is_boolean(),
            Self::Json => v.is_object() || v.is_array(),
        }
    }
}

/// Satu masalah type-check (MV-2, arah Rosetta).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParamIssue {
    /// Nama parameter bermasalah.
    pub param: String,
    /// Deskripsi masalah.
    pub message: String,
}

/// Type-check parameter node thd manifest (MV-2): required/type/options/
/// min/max. Hanya memeriksa field yang DEKLARATIF di manifest; parameter
/// tambahan di node tidak ditolak (n8n param ekstra umum).
pub fn type_check_params(m: &WcbManifest, params: &Value) -> Vec<ParamIssue> {
    let mut issues = Vec::new();
    let obj = match params.as_object() {
        Some(o) => o,
        None => {
            issues.push(ParamIssue {
                param: "<root>".into(),
                message: "parameters harus objek JSON".into(),
            });
            return issues;
        }
    };

    for p in &m.params {
        match obj.get(&p.name) {
            None => {
                if p.required {
                    issues.push(ParamIssue {
                        param: p.name.clone(),
                        message: format!("param wajib `{}` tidak ada", p.name),
                    });
                }
            }
            Some(v) => {
                if !p.ty.accepts(v) {
                    issues.push(ParamIssue {
                        param: p.name.clone(),
                        message: format!(
                            "`{}`: harap {} — dapat {}",
                            p.name,
                            p.ty.type_name(),
                            json_type_name(v)
                        ),
                    });
                    continue;
                }
                if !p.options.is_empty() && !p.options.contains(v) {
                    issues.push(ParamIssue {
                        param: p.name.clone(),
                        message: format!(
                            "`{}`: nilai {v:?} tidak ada di opsi {options:?}",
                            p.name,
                            options = p.options
                        ),
                    });
                }
                if let Some(n) = v.as_f64() {
                    if let Some(min) = p.min {
                        if n < min {
                            issues.push(ParamIssue {
                                param: p.name.clone(),
                                message: format!("`{}`: {n} < min {min}", p.name),
                            });
                        }
                    }
                    if let Some(max) = p.max {
                        if n > max {
                            issues.push(ParamIssue {
                                param: p.name.clone(),
                                message: format!("`{}`: {n} > max {max}", p.name),
                            });
                        }
                    }
                }
            }
        }
    }
    issues
}

fn json_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const MANIFEST_OK: &str = r#"{
        "schema_version": 1,
        "node_kind": "wcb-guest",
        "n8n_type": "@blotato/n8n-nodes-blotato.blotato",
        "module_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "module_digest_blake3": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "abi": {"exports": ["alloc","run","out_ptr","out_len"], "memory_max_pages": 2048},
        "imports": {"env": ["wcb_net_request"], "record": ["lookup","put"]},
        "granularity": ["cssQuery"],
        "params": [
            {"name":"url","type":"string","required":true,"maps_to":"url_field"},
            {"name":"mode","type":"string","default":"smart","options":["smart","css","xpath"]},
            {"name":"concurrency","type":"integer","min":1,"max":5}
        ],
        "output": {"item_shape":"Item","item_list":"ItemList::Inline|Spilled"},
        "egress_policy": {"default":"deny","allowlist_hosts":[],"allow_schemes":["http","https"]},
        "guards": {"body_window_bytes":262144,"polite_delay_ms":[2000,5000],"robots":"enforce"},
        "determinism": {"wall_clock_in_item": false}
    }"#;

    #[test]
    fn parse_dan_validasi_ok() {
        let m = parse_manifest(MANIFEST_OK).unwrap();
        assert_eq!(
            m.n8n_type.as_deref(),
            Some("@blotato/n8n-nodes-blotato.blotato")
        );
        assert!(validate_manifest(&m).is_empty());
    }

    #[test]
    fn validasi_menolak_sha_salah_dan_schema_baru() {
        let mut m = parse_manifest(MANIFEST_OK).unwrap();
        m.module_sha256 = "abc".into();
        assert!(validate_manifest(&m).iter().any(|e| e.contains("64 hex")));
        m.module_sha256 = "a".repeat(64);
        m.schema_version = 2;
        let errs = validate_manifest(&m);
        assert!(errs.iter().any(|e| e.contains("schema_version")));
        // egress default bukan deny
        let mut m2 = parse_manifest(MANIFEST_OK).unwrap();
        m2.egress_policy.default = "allow".into();
        assert!(validate_manifest(&m2).iter().any(|e| e.contains("deny")));
    }

    #[test]
    fn n8n_type_kosong_ditolak_amend_v04() {
        let mut m = parse_manifest(MANIFEST_OK).unwrap();
        m.n8n_type = Some("   ".into());
        assert!(validate_manifest(&m).iter().any(|e| e.contains("n8n_type")));
        m.n8n_type = None;
        assert!(
            validate_manifest(&m).is_empty(),
            "opsional → tanpa n8n_type valid"
        );
    }

    #[test]
    fn type_check_required_tipe_opsi_batas() {
        let m = parse_manifest(MANIFEST_OK).unwrap();

        // bagus
        let ok = type_check_params(
            &m,
            &json!({"url": "https://x", "mode": "css", "concurrency": 2}),
        );
        assert!(ok.is_empty(), "{ok:?}");

        // required hilang
        let issues = type_check_params(&m, &json!({"mode": "css"}));
        assert!(issues
            .iter()
            .any(|i| i.param == "url" && i.message.contains("wajib")));

        // tipe salah
        let issues = type_check_params(&m, &json!({"url": 42}));
        assert!(issues
            .iter()
            .any(|i| i.param == "url" && i.message.contains("string")));

        // di luar opsi
        let issues = type_check_params(&m, &json!({"url": "u", "mode": "magic"}));
        assert!(issues
            .iter()
            .any(|i| i.param == "mode" && i.message.contains("opsi")));

        // di luar min/max
        let issues = type_check_params(&m, &json!({"url": "u", "concurrency": 9}));
        assert!(issues
            .iter()
            .any(|i| i.param == "concurrency" && i.message.contains("max")));
    }

    #[test]
    fn param_ekstra_node_tidak_ditolak() {
        let m = parse_manifest(MANIFEST_OK).unwrap();
        let issues = type_check_params(&m, &json!({"url": "u", "extraParam": "x"}));
        assert!(
            issues.is_empty(),
            "param ekstra di node tidak ditolak (n8n umum)"
        );
    }
}
