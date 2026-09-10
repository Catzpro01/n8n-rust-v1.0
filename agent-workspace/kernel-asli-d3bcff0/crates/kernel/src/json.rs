//! Pure JSON depth guard — RULING 41 (#1345), API diresmikan RULING 47b (#1420).
//!
//! # Why this lives in the kernel
//!
//! Several implementations of a depth checker would drift like kernel copies
//! did (closed in ebfa533): one fixes escape handling, another does not; one
//! uses `u32`, another `u64`. So there is exactly ONE scanner, here, as a
//! **pure function with no policy**. Each consumer sets its own limit constant,
//! because the limit is policy:
//!
//! - `nodes-wasm` (WCB external artefacts): `MAX_JSON_DEPTH = 64`
//! - storage lineage `inputs_*` ingest: `MAX_JSON_DEPTH = 64`
//! - openapi-codegen ingest: `MAX_JSON_DEPTH = 64` (or per-consumer reason)
//! - `mcp` JSON-RPC frames: `MAX_JSON_DEPTH = 64`
//!
//! The check must run BEFORE `serde_json` parses: serde itself recurses on
//! deeply nested documents and would exhaust the stack before a post-parse
//! check ever ran.

/// Single scanner: returns `None` while the JSON structure in `bytes` stays
/// within `max` nesting; returns `Some(found)` the moment it exceeds it, where
/// `found` is the depth reached (fail-closed). Callers get a truthful error
/// message with the actual number in ONE pass — no re-scan for the message
/// (a double scan was the defect in `mcp/frame.rs:19-20`, Ruling 47b).
///
/// Iterative — no recursion — so it cannot overflow the stack on the very input
/// it exists to reject. Brackets inside strings and escaped quotes are ignored.
/// Pure: no IO, no allocation, no policy.
///
/// # Byte-vs-char invariant (agent10 #1428, Catatan A)
///
/// This scans `&[u8]`, not `&str`/chars. That is safe because every structural
/// JSON character (`{ } [ ] " \` and whitespace) is ASCII, and UTF-8 never
/// embeds an ASCII byte inside a multi-byte sequence — continuation bytes are
/// all `>= 0x80`. So byte scanning cannot miscount on CJK/emoji content, and
/// the `&str`/chars and `&[u8]`/byte forms agree on valid UTF-8. If a future
/// caller ever feeds UTF-16 or Latin-1, this assumption breaks without a
/// signal — hence it is documented here, where the owner is the kernel.
pub fn json_depth_exceeded(bytes: &[u8], max: u32) -> Option<u32> {
    let mut depth: u32 = 0;
    let mut in_str = false;
    let mut esc = false;
    for &b in bytes {
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' | b'[' => {
                // saturating_add: a u32 wrap after 4G openers would turn a reject
                // into a silent fail-open in release (agent1 #1361 micro-fix).
                depth = depth.saturating_add(1);
                if depth > max {
                    return Some(depth);
                }
            }
            b'}' | b']' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    None
}

/// Boolean convenience view over the single scanner (NOT a second
/// implementation — same loop, `.is_none()`). Use when the reached depth is
/// not needed; prefer [`json_depth_exceeded`] for error paths that must report
/// the offending depth.
pub fn json_depth_within(bytes: &[u8], max: u32) -> bool {
    json_depth_exceeded(bytes, max).is_none()
}

#[cfg(test)]
mod tests {
    use super::{json_depth_exceeded, json_depth_within};

    fn open(c: u8, n: u32) -> Vec<u8> {
        vec![c; n as usize]
    }

    #[test]
    fn empty_document_ok() {
        assert_eq!(json_depth_exceeded(b"", 64), None);
        assert_eq!(json_depth_exceeded(b"[]", 64), None);
        assert_eq!(json_depth_exceeded(b"{}", 64), None);
        assert!(json_depth_within(b"[]", 64));
    }

    #[test]
    fn flat_object_ok() {
        let doc = br#"{"a":1,"b":[1,2,3],"c":{"d":null}}"#;
        assert_eq!(json_depth_exceeded(doc, 64), None);
        assert_eq!(json_depth_exceeded(doc, 2), None); // max depth of the doc is 2
    }

    #[test]
    fn depth_equal_to_max_accepted() {
        // 64 levels of '[' then one scalar then 64 ']' -> depth 64 == max -> ok
        let mut doc = open(b'[', 64);
        doc.push(b'0');
        doc.extend(open(b']', 64));
        assert_eq!(json_depth_exceeded(&doc, 64), None);
    }

    #[test]
    fn depth_over_max_rejected_with_reached_depth() {
        let mut doc = open(b'[', 65);
        doc.push(b'0');
        doc.extend(open(b']', 65));
        assert_eq!(json_depth_exceeded(&doc, 64), Some(65));
        assert!(!json_depth_within(&doc, 64));
    }

    #[test]
    fn deep_74_rejected() {
        // Returns at the FIRST depth that exceeds max, so found == 65 here
        // even though the document nests to 74.
        let mut doc = open(b'[', 74);
        doc.push(b'0');
        doc.extend(open(b']', 74));
        assert_eq!(json_depth_exceeded(&doc, 64), Some(65));
        assert!(!json_depth_within(&doc, 64));
    }

    #[test]
    fn brackets_inside_string_ignored() {
        // "[[" / "{}}" inside strings must not count toward depth.
        // Real nesting: root {1, "c":[ 2, { 3 -> max 3.
        let doc = br#"{"a":"[[[[","b":"{}}","c":[1,{"x":"]{"}]}"#;
        assert_eq!(json_depth_exceeded(doc, 64), None);
        assert_eq!(json_depth_exceeded(doc, 3), None);
        assert_eq!(json_depth_exceeded(doc, 2), Some(3));
    }

    #[test]
    fn escaped_quotes_handled() {
        // String with escaped quotes containing bracket-looking text.
        let doc = br#"{"a":"{\"[[\":0}","b":{"nested":[1,2]}}"#;
        assert_eq!(json_depth_exceeded(doc, 64), None);
        assert_eq!(json_depth_exceeded(doc, 3), None);
        assert_eq!(json_depth_exceeded(doc, 2), Some(3)); // doc nests to 3
    }

    #[test]
    fn unterminated_string_ignores_rest() {
        // After an opening quote with no closing one, brackets are string data.
        let doc = br#"{"a":"[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[["#;
        assert_eq!(json_depth_exceeded(doc, 64), None);
    }

    #[test]
    fn boundary_63_64_65() {
        // max=63: 64-deep doc -> Some(64); max=64: 64-deep ok (None), 65-deep -> Some(65).
        let d64 = {
            let mut doc = open(b'[', 64);
            doc.push(b'0');
            doc.extend(open(b']', 64));
            doc
        };
        let d65 = {
            let mut doc = open(b'[', 65);
            doc.push(b'0');
            doc.extend(open(b']', 65));
            doc
        };
        assert_eq!(json_depth_exceeded(&d64, 63), Some(64));
        assert_eq!(json_depth_exceeded(&d64, 64), None);
        assert_eq!(json_depth_exceeded(&d65, 64), Some(65));
    }

    #[test]
    fn zero_max_accepts_only_scalars() {
        // depth > max is checked on EVERY opener, so at max=0 any bracket fails.
        assert_eq!(json_depth_exceeded(b"", 0), None);
        assert_eq!(json_depth_exceeded(b"42", 0), None);
        assert_eq!(json_depth_exceeded(br#""42""#, 0), None); // JSON string scalar
        assert_eq!(json_depth_exceeded(b"[]", 0), Some(1));
        assert_eq!(json_depth_exceeded(b"{}", 0), Some(1));
        assert_eq!(json_depth_exceeded(b"[[]]", 0), Some(1));
    }
}
