//! W1-WCB-IMPL Slice-2 — manifest node WCB-guest (schema v0.1 per RFC #950 v0.3).
//!
//! Scaffold awal (shadow /home/agent6/W1-WCB-IMPL/crates/nodes-wasm/src/).
//! Prinsip: JSON wire manifest; NOL vocab paralel; nama kernel di `maps_to`
//! hanya sitasi (NC-2 #961). Tipe kernel dipakai pada lapis gate MV-2/MV-3.

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Wire schema (RFC #950 v0.3 §2) — self-contained, tanpa dep luar kernel.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub schema_version: u32,
    pub node_kind: NodeKind,
    /// Jembatan resolver workflow-level (agent4 amend v0.4 #1126): tipe n8n node komunitas
    /// di workflow -> manifest mana. OPSIONAL, plain identifier; BUKAN sumbu identitas kedua —
    /// identitas tipe node tetap module_sha256 (NC-1). Dibaca Rosetta/Hub, bukan kernel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n8n_type: Option<String>,
    pub module_sha256: String,        // SHA-256 BERKAS .wasm fisik (NC-1; BAHASA-BERSAMA §2)
    pub module_digest_blake3: String, // BLAKE3 32B digest integritas modul
    pub abi: Abi,
    pub imports: ImportsAllowlist,    // allowlist deny-by-default; nol impor lain
    #[serde(default)]
    pub granularity: Vec<String>,
    #[serde(default)]
    pub params: Vec<ParamSpec>,
    pub output: OutputShape,
    pub egress_policy: EgressPolicy,
    pub guards: Guards,
    pub determinism: Determinism,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    WcbGuest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Abi {
    pub exports: Vec<String>,
    /// Halaman memori linear maks. Halaman WASM = 64 KiB, jadi 32 MiB = 512 halaman.
    /// (Erratum: contoh v0.3 sempat menulis 2048 dgn klaim "32MiB" — 2048 = 128MiB, keliru.)
    pub memory_max_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ImportsAllowlist {
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub record: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParamSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: ParamType,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
    /// Sitasi nama field ParameterSchema/NodeDescriptor KERNEL persis (NC-2 #961).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maps_to: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    String,
    Integer,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputShape {
    pub item_shape: String,  // "Item" (item.rs) — tipe kernel
    pub item_list: String,   // "ItemList::Inline|Spilled" — tipe kernel
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EgressPolicy {
    #[serde(default = "default_deny")]
    pub default: DenyMode,
    #[serde(default)]
    pub allowlist_hosts: Vec<String>, // MV-4: kosong => nol jaringan
    #[serde(default)]
    pub allow_schemes: Vec<String>,   // skema-allowlist (agent9 R-1 #933)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DenyMode {
    Deny,
    Allow,
}

fn default_deny() -> DenyMode {
    DenyMode::Deny
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Guards {
    /// body_window 256 KiB (konsensus #922 §B) -> >window = spill + truncated.
    pub body_window_bytes: u64,
    pub polite_delay_ms: (u64, u64),
    pub robots: RobotsMode,
    pub pii_redaction: String, // "facade-1x"
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RobotsMode {
    Enforce,
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Determinism {
    pub input_digest: String,        // host-side sha256 sebelum guest (#905 Q2)
    pub jitter_seed: String,         // input_digest:attempt (adapter v0.3)
    pub replay: String,              // record::lookup (no re-fetch) (#777-E)
    pub wall_clock_in_item: bool,    // MV-5: false; took_ms/duration_ms DILARANG di Item/digest
}

// ---------------------------------------------------------------------------
// Validasi awal — deny-by-default + bentuk param (MV-1 porsi statis).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    UnsupportedVersion(u32),
    UnknownNodeKind(String),
    ModuleSha256Malformed(String),
    ImportNotAllowlisted { module: String, name: String },
    ParamNameEmpty,
    ParamTypeUnknown(String),
    DuplicateParamName(String),
    BodyWindowZero,
    MemoryPagesOverLimit(u32), // > 512 halaman (=32MiB linear) ditolak (sandbox)
    EmptyN8nType,
    /// Ruling 39b (#1317): kedalaman JSON eksternal melebihi batas (fail-closed
    /// sebelum serde parse; error TERLIHAT menyebut dokumen & kedalaman).
    DepthExceeded { max: u32, found: u32 },
    /// JSON tak valid secara sintaks.
    Parse(String),
}

/// Batas keras memori linear node WCB: 32 MiB = 512 halaman (mandat #1074, pooling wasmtime).
pub const MAX_MEMORY_PAGES: u32 = 512;

pub fn validate(m: &Manifest) -> Result<(), ManifestError> {
    if m.schema_version != SCHEMA_VERSION {
        return Err(ManifestError::UnsupportedVersion(m.schema_version));
    }
    if m.abi.memory_max_pages > MAX_MEMORY_PAGES {
        return Err(ManifestError::MemoryPagesOverLimit(m.abi.memory_max_pages));
    }
    if m.guards.body_window_bytes == 0 {
        return Err(ManifestError::BodyWindowZero);
    }
    if !(m.module_sha256.len() == 64 && m.module_sha256.chars().all(|c| c.is_ascii_hexdigit())) {
        return Err(ManifestError::ModuleSha256Malformed(m.module_sha256.clone()));
    }
    if let Some(t) = &m.n8n_type {
        if t.trim().is_empty() {
            return Err(ManifestError::EmptyN8nType);
        }
    }
    // deny-by-default pada sisi impor: env + record adalah SATU-SATUNYA namespace yang boleh.
    // (audit impor WASM nyata thd allowlist ini dilakukan di lapis host saat load — MV-1 penuh.)
    let mut seen = std::collections::HashSet::new();
    for p in &m.params {
        if p.name.is_empty() {
            return Err(ManifestError::ParamNameEmpty);
        }
        if !seen.insert(p.name.clone()) {
            return Err(ManifestError::DuplicateParamName(p.name.clone()));
        }
    }
    Ok(())
}

/// Parsing produksi manifest dari byte EKSTERNAL: periksa kedalaman dulu
/// (Ruling 39b/41/47b via kernel::json_depth_exceeded, fail-closed), baru serde.
/// F1 agent1 #1330: error DepthExceeded{max,found} TERLIHAT.
pub fn parse_manifest(bytes: &[u8]) -> Result<Manifest, ManifestError> {
    // Ruling 39b / 41 / 47b: SATU scanner di kernel (commit 5bc8ea8). Fail-closed
    // SEBELUM serde parse; error TERLIHAT menyebut kedalaman yang dicapai (F1 #1330).
    if let Some(found) =
        kernel::json_depth_exceeded(bytes, crate::gates::MAX_JSON_DEPTH)
    {
        return Err(ManifestError::DepthExceeded {
            max: crate::gates::MAX_JSON_DEPTH,
            found,
        });
    }
    serde_json::from_slice(bytes).map_err(|e| ManifestError::Parse(e.to_string()))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("../fixtures/manifest.example.json");

    /// Fixture utk dipakai silang oleh module gates (cfg(test) saja).
    pub(crate) fn example_manifest() -> Manifest {
        serde_json::from_str(EXAMPLE).expect("example valid")
    }

    #[test]
    fn parses_example_manifest() {
        let m = example_manifest();
        assert_eq!(m.schema_version, 1);
        assert_eq!(m.abi.memory_max_pages, 512);
        assert!(m.egress_policy.allowlist_hosts.is_empty());
        assert_eq!(m.guards.body_window_bytes, 262144);
        assert!(!m.determinism.wall_clock_in_item);
        assert!(validate(&m).is_ok());
    }

    #[test]
    fn parse_manifest_gates_depth_and_rejects_bad_json() {
        // normal -> ok
        let m = parse_manifest(EXAMPLE.as_bytes()).expect("example valid");
        assert_eq!(m.schema_version, 1);

        // bersarang terlalu dalam -> DepthExceeded (fail-closed, dengan angka)
        let mut deep = String::new();
        for _ in 0..(crate::gates::MAX_JSON_DEPTH + 5) {
            deep.push('[');
        }
        deep.push('0');
        for _ in 0..(crate::gates::MAX_JSON_DEPTH + 5) {
            deep.push(']');
        }
        match parse_manifest(deep.as_bytes()) {
            Err(ManifestError::DepthExceeded { max, found }) => {
                assert!(found > max);
            }
            other => panic!("harus DepthExceeded, dapat {other:?}"),
        }

        // sintaks buruk -> Parse
        assert!(matches!(
            parse_manifest(br#"{"schema_version":"x""#),
            Err(ManifestError::Parse(_))
        ));
    }

    #[test]
    fn rejects_unknown_schema_version() {
        let mut m: Manifest = serde_json::from_str(EXAMPLE).unwrap();
        m.schema_version = 2;
        assert_eq!(validate(&m), Err(ManifestError::UnsupportedVersion(2)));
    }

    #[test]
    fn rejects_memory_over_32mib() {
        let mut m: Manifest = serde_json::from_str(EXAMPLE).unwrap();
        m.abi.memory_max_pages = MAX_MEMORY_PAGES + 1;
        assert_eq!(
            validate(&m),
            Err(ManifestError::MemoryPagesOverLimit(MAX_MEMORY_PAGES + 1))
        );
    }

    #[test]
    fn rejects_malformed_sha256() {
        let mut m: Manifest = serde_json::from_str(EXAMPLE).unwrap();
        m.module_sha256 = "zz".repeat(32);
        assert!(matches!(validate(&m), Err(ManifestError::ModuleSha256Malformed(_))));
    }

    #[test]
    fn n8n_type_optional_and_must_be_nonempty() {
        let m = example_manifest();
        assert_eq!(m.n8n_type.as_deref(), Some("community.wcb.extract"));
        let mut m2 = m.clone();
        m2.n8n_type = Some("  ".into());
        assert_eq!(validate(&m2), Err(ManifestError::EmptyN8nType));
    }

    #[test]
    fn rejects_duplicate_param() {
        let mut m: Manifest = serde_json::from_str(EXAMPLE).unwrap();
        let p = m.params[0].clone();
        m.params.push(p);
        assert!(matches!(validate(&m), Err(ManifestError::DuplicateParamName(_))));
    }
}
