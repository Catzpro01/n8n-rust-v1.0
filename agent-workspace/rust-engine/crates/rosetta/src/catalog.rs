//! Klasifikasi asal node berdasarkan namespace type (M1).
//!
//! M1 hanya membedakan namespace: base, first-party `@n8n/*`, komunitas, dan
//! unknown. Verifikasi per-tipe terhadap katalog 694 node = R-6 (milestone
//! lanjut, setelah DEVIATION-CATALOG/DEVIATION-CATALOG dan penerimaan katalog
//! #418) — tidak ada klaim "seluruh 694 dikenal" sebelum itu.
//!
//! Referensi: AGENT4-NODE-ALIAS-DEPRECATION.md, AGENT4_SCHEMA_COMPILER_SPEC.md F-C3.

use crate::model::NodeOrigin;

/// Prefix node inti n8n (bundle default).
pub const N8N_BASE_PREFIX: &str = "n8n-nodes-base.";

/// Prefix first-party lain milik n8n (mis. `@n8n/n8n-nodes-langchain.*`).
pub const N8N_FIRST_PARTY_PREFIXES: &[&str] = &["@n8n/"];

/// Awalan yang TIDAK kita klaim dikenal tanpa verifikasi lanjut:
/// tipe dengan awalan lain (komunitas `@scope/pkg.node`, `n8n-nodes-*.custom`)
/// atau tanpa titik namespace.
pub fn origin_of(type_full: &str) -> NodeOrigin {
    if type_full.starts_with(N8N_BASE_PREFIX) {
        NodeOrigin::N8nBase
    } else if N8N_FIRST_PARTY_PREFIXES
        .iter()
        .any(|p| type_full.starts_with(p))
    {
        NodeOrigin::N8nFirstParty
    } else if type_full.contains('.') {
        NodeOrigin::Community
    } else {
        NodeOrigin::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_namespaces() {
        assert_eq!(origin_of("n8n-nodes-base.cron"), NodeOrigin::N8nBase);
        assert_eq!(
            origin_of("@n8n/n8n-nodes-langchain.lmChatOpenAi"),
            NodeOrigin::N8nFirstParty
        );
        assert_eq!(
            origin_of("@blotato/n8n-nodes-blotato.blotato"),
            NodeOrigin::Community
        );
        assert_eq!(origin_of("noNamespace"), NodeOrigin::Unknown);
    }
}
