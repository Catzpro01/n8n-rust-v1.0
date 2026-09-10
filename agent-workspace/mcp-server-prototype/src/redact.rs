//! Redaksi oracle (#394) di batas facade (A7-6) — transport tidak boleh
//! melewati lapisan redaksi. Juga taint 3-level (TOOLS-PLAN amendemen v0.2).

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaintLevel {
    External,   // data dari input/eksternal — TAINTED-EXTERNAL
    Sanitized,  // sudah lewat sanitasi agent5
    Internal,   // data internal engine
}

pub fn taint_label(t: TaintLevel) -> &'static str {
    match t {
        TaintLevel::External => "TAINTED-EXTERNAL",
        TaintLevel::Sanitized => "SANITIZED",
        TaintLevel::Internal => "INTERNAL",
    }
}

/// Redaksi: email, token (bearer/ghp_/sk-), telepon → placeholder.
/// Diterapkan sekali di facade (A7-6). Versi prototipe: regex-free scanner.
pub fn redact(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut i = 0;
    while i < n {
        // email sederhana: lokal@domain.tld
        if i + 3 < n && chars[i].is_ascii_alphanumeric() {
            let mut j = i;
            while j < n && (chars[j].is_ascii_alphanumeric() || matches!(chars[j], '.' | '_' | '%' | '+' | '-')) { j += 1; }
            if j > i && j + 1 < n && chars[j] == '@' {
                let mut k = j + 1;
                while k < n && (chars[k].is_ascii_alphanumeric() || matches!(chars[k], '.' | '-')) { k += 1; }
                if k > j + 1 && chars[k - 1] != '.' && s[i..k].contains('.') && k - (j + 1) >= 2 {
                    out.push_str("[EMAIL]");
                    i = k;
                    continue;
                }
            }
        }
        // token: ghp_36char / sk- / Bearer xxx
        if chars[i].is_ascii_alphabetic() {
            let rest: String = chars[i..].iter().collect();
            if rest.starts_with("ghp_") {
                out.push_str("[TOKEN]"); i = n; continue;
            }
            let lower = rest.to_lowercase();
            if lower.starts_with("bearer ") || lower.starts_with("bearer\t") {
                // potong sampai spasi/akhir (1 token)
                out.push_str("[TOKEN]");
                let mut k = i + 7;
                while k < n && !chars[k].is_whitespace() { k += 1; }
                i = k;
                continue;
            }
            if rest.starts_with("sk-") && rest.len() > 22 {
                out.push_str("[TOKEN]"); i = n; continue;
            }
        }
        // telepon: pola longgar digit 8-15 dgn spasi/titik/dash/+ separators
        if chars[i].is_ascii_digit() || chars[i] == '+' {
            let mut j = i;
            let mut digits = 0usize;
            let mut seps = 0usize;
            while j < n && j < i + 20 {
                let c = chars[j];
                if c.is_ascii_digit() { digits += 1; }
                else if matches!(c, ' ' | '.' | '-' | '(' | ')') { seps += 1; }
                else { break; }
                j += 1;
            }
            if digits >= 8 && digits <= 15 && (seps >= 1 || digits >= 11) {
                out.push_str("[PHONE]"); i = j; continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Redaksi atas seluruh string dalam struktur (rekursif, kunci dipertahankan).
pub fn redact_value(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::String(redact(s)),
        Value::Array(a) => Value::Array(a.iter().map(redact_value).collect()),
        Value::Object(o) => {
            let mut m = serde_json::Map::new();
            for (k, val) in o { m.insert(k.clone(), redact_value(val)); }
            Value::Object(m)
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-60..64: taint/redaksi
    #[test]
    fn tc_redact_email() {
        let out = redact("hubungi a.b@mail.test segera");
        assert!(!out.contains("a.b@mail.test"));
        assert!(out.contains("[EMAIL]"));
    }
    #[test]
    fn tc_redact_token_ghp() {
        let tok = "ghp_0123456789012345678901234567890123456789";
        let out = redact(&format!("token={tok} end"));
        assert!(!out.contains("ghp_"));
        assert!(out.contains("[TOKEN]"));
    }
    #[test]
    fn tc_redact_bearer() {
        let out = redact("Authorization: Bearer abc123XYZ secret");
        assert!(!out.contains("abc123XYZ"));
        assert!(out.contains("[TOKEN]"));
    }
    #[test]
    fn tc_redact_phone() {
        let out = redact("hp +62 812-3456-7890 oke");
        assert!(!out.contains("812-3456"));
        assert!(out.contains("[PHONE]"));
    }
    #[test]
    fn tc_redact_no_false_positive_short() {
        assert_eq!(redact("a1b2c3"), "a1b2c3");
        assert_eq!(redact("tahun 2026"), "tahun 2026");
    }
    #[test]
    fn tc_redact_value_deep() {
        let v = serde_json::json!({"nested": {"email": "x@y.id", "arr": ["p: 081234567890"]}});
        let out = redact_value(&v);
        let s = out.to_string();
        assert!(!s.contains("x@y.id"));
        assert!(s.contains("[EMAIL]"));
    }
    #[test]
    fn tc_taint_label() {
        assert_eq!(taint_label(TaintLevel::External), "TAINTED-EXTERNAL");
    }
}
