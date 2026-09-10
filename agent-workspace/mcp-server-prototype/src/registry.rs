//! Registri URI JIT (AGENT7-JIT-ROUTING-REGISTRY v0.5): 9 URI n8n:// + 4 URI hub://.
//! Prototipe: konten deterministik embed (ringkas) + metadata; pemilik per baris.
//! Aturan: server TIDAK serve kode RESERVED; receipt-outdated flag (H-7).

use serde_json::{json, Value};
use crate::diag::RESERVED_CODES;

pub struct UriRow {
    pub pattern: &'static str,
    pub owner: &'static str,
    pub content_version: &'static str,
    pub note: &'static str,
}

pub const URI_ROWS: &[UriRow] = &[
    UriRow { pattern: "n8n://nodes/{name}/schema", owner: "agent4", content_version: "0.2", note: "schema compiler v0.2" },
    UriRow { pattern: "n8n://nodes/{type}/alias", owner: "agent4", content_version: "1.0", note: "alias deprecation V-1/V-2" },
    UriRow { pattern: "n8n://nodes/cron/mapping", owner: "agent4", content_version: "1.0", note: "cron v1→v2 D1-D4" },
    UriRow { pattern: "n8n://expression-rules", owner: "agent3", content_version: "2", note: "DRAFT v1.0 cv2" },
    UriRow { pattern: "n8n://errors/{code}", owner: "agent1", content_version: "1.0", note: "E-* + MIG-*/PARTIAL agent4" },
    UriRow { pattern: "n8n://deviation/{DEV-*}", owner: "agent5", content_version: "1.0", note: "DEVIATION-CATALOG" },
    UriRow { pattern: "n8n://limits", owner: "agent2", content_version: "1.0", note: "budget/guardrail/allowlist $env" },
    UriRow { pattern: "n8n://workflow/{id}/receipt", owner: "agent4", content_version: "1.0", note: "receipt Rosetta" },
    UriRow { pattern: "n8n://templates/{id}", owner: "agent4", content_version: "1.0", note: "3 pilar + ?lang (agent2)" },
    UriRow { pattern: "n8n://templates/{id}?lang=*", owner: "agent2", content_version: "1.0", note: "i18n_rev" },
    UriRow { pattern: "hub://skills/{hub_id}/manifest", owner: "agent4", content_version: "0.5", note: "manifest Hub (SEC-HUB-01)" },
    UriRow { pattern: "hub://skills/{hub_id}/receipt", owner: "agent4", content_version: "0.5", note: "receipt Rosetta ringkas" },
    UriRow { pattern: "hub://skills/{hub_id}/manifest?lang=", owner: "agent4", content_version: "0.5", note: "i18n (agent2)" },
    UriRow { pattern: "hub://intel-types", owner: "agent9", content_version: "1.2", note: "katalog intel" },
];

pub fn lookup_owner(uri: &str) -> Option<&'static UriRow> {
    URI_ROWS.iter().find(|r| {
        let p = r.pattern;
        if p.contains("{") {
            // pola: awalan sampai '{' harus cocok; sisanya wildcard per-segmen
            let prefix = &p[..p.find('{').unwrap()];
            if !uri.starts_with(prefix) { return false; }
            let suffix = &p[p.find('}').unwrap() + 1..];
            uri.ends_with(suffix) || suffix.is_empty()
        } else {
            uri == p || uri.starts_with(&format!("{p}?"))
        }
    })
}

/// Paket konten untuk URI — deterministik, ≤500 token (dipotong di frame layer bila perlu).
pub fn build_content(uri: &str) -> Result<Value, String> {
    let row = lookup_owner(uri).ok_or_else(|| format!("URI tidak dikenal: {uri}"))?;

    // Kode RESERVED tidak pernah di-serve (aturan registri agent1 §4 + #715/#682)
    if let Some(code) = uri.strip_prefix("n8n://errors/") {
        let code = code.split('?').next().unwrap_or("");
        if RESERVED_CODES.contains(&code) {
            return Err(format!("kode {code} RESERVED — server tidak melayani konten kode RESERVED"));
        }
    }

    let (payload, extra): (Value, Value) = if uri.starts_with("n8n://expression-rules") {
        (json!({
            "summary": "Aturan ekspresi E-3/W-5 final (F-6 #503); 215 kasus; tabel penuh di sumber agent3 DRAFT v1.0 cv2.",
            "errors": ["E-EXPR-REFERENCE", "E-EXPR-SCOPE", "E-EXPR-CREDENTIAL"],
            "warnings": ["W-EXPR-FLOAT-PRECISION", "W-EXPR-SURROGATE", "W-EXPR-OBJECT-ORDER", "W-EXPR-UNDEFINED-NULL", "W-EXPR-ARRAY-METHODS"],
            "note": "autofix 3-level #446/#449; finalisasi integrasi agent7"
        }), json!({}))
    } else if let Some(code) = uri.strip_prefix("n8n://errors/") {
        let code = code.split('?').next().unwrap_or("").to_string();
        let entry = match code.as_str() {
            "E-REF-DANGLING" => json!({"meaning":"referensi node tidak dikenal","example":"$node[\"Ghost\"]","autofix":"periksa nama node"}),
            "E-DAG-CYCLE" => json!({"meaning":"cycle koneksi","autofix":"arahkan ulang"}),
            "E-DUP-NODE-NAME" => json!({"meaning":"nama duplikat","autofix":"rename"}),
            "E-EXPR-CREDENTIAL" => json!({"meaning":"nilai kredensial di expression","autofix":"refs-only (D-6)"}),
            "E-HUB-INPUT-OVERSIZE" => json!({"meaning":"RESERVED untuk Hub","reserved":true}),
            _ => json!({"meaning":"kode terdaftar di registri agent1/agent4","known":true}),
        };
        (json!({"code": code, "entry": entry}), json!({}))
    } else if uri.starts_with("n8n://limits") {
        (json!({
            "budget": {"ram_mb": 500, "per_request_kb": 4},
            "guardrail": ["SSRF block", "polite delay 2-5s", "robots.txt"],
            "allowlist_env": "daftar $env di dokumen owner agent2 (AGENT1-MCP-SKILL-SPEC §6)",
            "credentials": "refs-only — nilai tidak pernah disajikan"
        }), json!({}))
    } else if let Some(rest) = uri.strip_prefix("n8n://templates/") {
        // templates/{id} atau templates/{id}?lang=...
        let (id, lang) = match rest.find('?') {
            Some(p) => (&rest[..p], Some(&rest[p + 6..])), // setelah "?lang="
            None => (rest, None),
        };
        if rest.contains("?lang=*") || (lang.is_some() && lang.unwrap().is_empty()) {
            return Ok(json!({
                "templates_languages": ["en", "id"],
                "coverage": {"en": "100% (source)", "id": "node-notes inti"},
                "i18n_rev": "0.2", "owner": "agent2"
            }));
        }
        // konten deterministik (embed mini)
        let base = json!({
            "id": id, "verdict": "deterministic-ok",
            "pillars": {"cara_kerja": "...", "fungsi": "...", "tujuan": "..."},
            "lang_served": lang.unwrap_or("en"),
            "i18n_fallback": lang.is_some() && lang.unwrap() != "en" && lang.unwrap() != "id",
            "source_ref": "corpus-171/manifest"
        });
        (base, json!({}))
    } else if let Some(hub) = uri.strip_prefix("hub://skills/") {
        let hub_id = hub.split('/').next().unwrap_or("").to_string();
        if hub.ends_with("/receipt") {
            let manifest_cv = "1.0.0"; // simulasi manifest saat ini
            let stored_cv = "0.9.0";   // simulasi receipt lokal
            (json!({
                "hub_id": hub_id, "template_sha256": "abc123def456",
                "content_version": stored_cv,
                "receipt_outdated": stored_cv != manifest_cv,  // H-7
                "hint": if stored_cv != manifest_cv { "re-execute needed (content_version < manifest)" } else { "current" }
            }), json!({}))
        } else if hub.contains("manifest") {
            (json!({
                "hub_id": hub_id, "manifest_version": "v0.5",
                "review_status": "verified-tofu", "est_token": 120,
                "pin": "katalog single-source (SEC-HUB-01)"
            }), json!({}))
        } else {
            (json!({"hub_id": hub_id}), json!({}))
        }
    } else if uri.starts_with("hub://intel-types") {
        (json!({
            "kinds": ["rss", "http_json", "sse", "scrape"],
            "katalog": "hub-intel-types v1.2 (033924f6)", "owner": "agent9"
        }), json!({}))
    } else if let Some(id) = uri.strip_prefix("n8n://workflow/") {
        let id = id.trim_end_matches("/receipt");
        (json!({"workflow_id": id, "receipt": {"rosetta": "ok", "deviation_ids": [], "ts": "2026-09-09T00:00:00Z"}}), json!({}))
    } else if uri.starts_with("n8n://deviation/") {
        (json!({"entry": uri.trim_start_matches("n8n://deviation/"), "catalog": "DEVIATION-CATALOG (agent5)"}), json!({}))
    } else {
        (json!({"uri": uri, "note": "konten minimal — owner: "}), json!({"owner": row.owner}))
    };

    Ok(json!({
        "uri": uri,
        "owner": row.owner,
        "content_version": row.content_version,
        "i18n_rev": "0.2",
        "updated_at": "2026-09-09T00:00:00Z",
        "source_ref": row.note,
        "taint": "INTERNAL",
        "payload": payload,
        "_extra": extra
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-50..59: resources
    #[test]
    fn tc_resolve_all_rows() {
        for uri in ["n8n://expression-rules", "n8n://limits", "n8n://errors/E-REF-DANGLING",
                    "n8n://templates/crypto-ticker?lang=id", "hub://intel-types",
                    "hub://skills/bm-macro/manifest", "hub://skills/bm-macro/receipt",
                    "n8n://nodes/HTTP Request/schema", "n8n://workflow/wf-1/receipt"] {
            assert!(lookup_owner(uri).is_some(), "missing {uri}");
        }
    }
    #[test]
    fn tc_reserved_not_served() {
        let r = build_content("n8n://errors/E-WCB-TRAP");
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("RESERVED"));
    }
    #[test]
    fn tc_receipt_outdated_flag() {
        let v = build_content("hub://skills/bm-macro/receipt").unwrap();
        assert_eq!(v["payload"]["receipt_outdated"], true);
        assert_eq!(v["payload"]["hint"], "re-execute needed (content_version < manifest)");
    }
    #[test]
    fn tc_lang_fallback_flag() {
        let v = build_content("n8n://templates/crypto-ticker?lang=id").unwrap();
        assert_eq!(v["payload"]["lang_served"], "id");
        assert_eq!(v["payload"]["i18n_fallback"], false);
        let v2 = build_content("n8n://templates/crypto-ticker?lang=zz").unwrap();
        assert_eq!(v2["payload"]["i18n_fallback"], true);
    }
    #[test]
    fn tc_metadata_present() {
        let v = build_content("n8n://expression-rules").unwrap();
        assert_eq!(v["owner"], "agent3");
        assert_eq!(v["content_version"], "2");
        assert_eq!(v["taint"], "INTERNAL");
    }
}
