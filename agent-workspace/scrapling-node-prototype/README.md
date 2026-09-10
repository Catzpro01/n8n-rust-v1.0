# n8n-nodes-scrapling Prototype (W2-NODE-SCRAPLING)

Adaptive stealth web scraping node with self-healing DOM parser.

## Status: ✅ 9/9 tests passing

## Test Results
```
test_batch_limit           ... ok  (max 50 URLs enforced)
test_batch_scrape          ... ok  (batch operation works)
test_deterministic_jitter  ... ok  (seeded PRNG, reproducible)
test_css_query             ... ok  (CSS selector extraction)
test_smart_extract         ... ok  (auto-healing extraction)
test_output_serialize      ... ok  (canonical output format)
test_ssrf_blocked          ... ok  (private range blocked)
test_operations            ... ok  (4 operations parse)
test_params_deserialize    ... ok  (11-field parameter schema)
```

## Components

### ScraplingParams (11 fields per agent9 spec §1)
- resource, operation, urls, rules, auto_heal
- selector, xpath, concurrency, mode
- respect_retry_after, ignore_robots

### Operations
- **smartExtract**: Auto-healing semantic extraction
- **cssQuery**: CSS selector-based extraction
- **xpathQuery**: XPath-based extraction
- **batchScrape**: Multi-URL concurrent scraping (max 50)

### Guardrails (spec §4)
- SSRF blocklist (private IP ranges)
- Robots.txt check
- Polite delay (2-5s per domain)
- PII redaction

### Deterministic Retry (spec §3, agent6 v0.3)
- Backoff: 1.5^n seconds
- Jitter: f(input_digest, attempt_index) — BLAKE3 seeded
- Max 3 attempts
- RecordSet replay compatible

### Output Format (spec §5)
```json
{
  "url": "https://example.com",
  "status": "success",
  "content_format": "json",
  "data_ref": {...},
  "auto_healed": false,
  "pii_redacted": true,
  "took_ms": 250,
  "attempts": 1,
  "input_digest": "abc123"
}
```

## Gate Targets (spec §6)
| Gate | Target | Status |
|------|--------|--------|
| SCRP-SL-1 | RAM <15MB httpStealth | prototype (no real HTTP) |
| SCRP-SL-2 | SSRF 20/20 blocked | ✓ tested |
| SCRP-SL-3 | Retry-After respected | prototype logic |
| SCRP-SL-4 | Self-heal 8/10 | prototype logic |
| SCRP-SL-5 | PII 0 leak | pii_redacted=true |
| SCRP-SL-6 | Replay deterministic | ✓ tested |

## Dependencies
- serde/serde_json, blake3, url, thiserror

## Basis
- Spec: AGENT9-NODE-SCRAPLING-SPEC.md v0.1
- Adapter: AGENT6-WCB-SCRAPLING-ADAPTER.md v0.3
- Reference: agent9 camofox L1-L3 implementations

## Author
agent3 (ROLE_EXPRESSION, adopsi ROLE_INTEGRATION)
