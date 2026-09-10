//! Mini ekspresi `={{ }}` (subset n8n, tanpa dependensi).
//!
//! Bentuk yang didukung:
//! - `$json.a.b` / `$json["a"].b` / `$json.arr[0]` — field item saat ini
//! - `$node["Nama"].json.a.b` (boleh `.first()` setelah nama) — item PERTAMA
//!   output node lain
//! - literal: `"str"`, `123`, `1.5`, `true`, `false`, `null`
//! - selain itu (termasuk operator `==`/`>` dan fungsi) → Null, lunak.
//!   n8n asli melempar error; subset ini memilih lunak + terdokumentasi.
//!
//! Render: bila SELURUH string adalah satu `={{...}}`, nilai asli
//! dipertahankan tipenya; bila template campuran, interpolasi jadi string
//! (missing → string kosong).

use serde_json::Value;
use std::collections::HashMap;

pub struct ExprContext<'a> {
    pub item: &'a Value,
    pub outputs: &'a HashMap<String, Vec<Value>>,
}

pub fn render(template: &str, ctx: &ExprContext) -> Value {
    let t = template.trim();
    if let Some(inner) = single_placeholder(t) {
        return eval(inner, ctx);
    }
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("={{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 3..];
        match after.find("}}") {
            Some(end) => {
                let v = eval(after[..end].trim(), ctx);
                out.push_str(&stringify(&v));
                rest = &after[end + 2..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    Value::String(out)
}

fn single_placeholder(t: &str) -> Option<&str> {
    if t.starts_with("={{") && t.ends_with("}}") {
        let inner = &t[3..t.len() - 2];
        if !inner.contains("={{") {
            return Some(inner.trim());
        }
    }
    None
}

fn eval(expr: &str, ctx: &ExprContext) -> Value {
    let e = expr.trim();
    if e.is_empty() {
        return Value::Null;
    }
    if let Some(rest) = e.strip_prefix("$json") {
        return drill(ctx.item, rest);
    }
    if let Some(rest) = e.strip_prefix("$node") {
        let (name, rest) = match parse_bracket(rest.trim_start()) {
            Some(x) => x,
            None => return Value::Null,
        };
        let first = ctx
            .outputs
            .get(&name)
            .and_then(|items| items.first())
            .unwrap_or(&Value::Null);
        let r = rest.trim_start();
        let r = r
            .strip_prefix(".first()")
            .map(str::trim_start)
            .unwrap_or(r);
        let r = match r.strip_prefix(".json") {
            Some(x) => x,
            None => return Value::Null,
        };
        return drill(first, r);
    }
    serde_json::from_str(e).unwrap_or(Value::Null)
}

/// `["Nama"]...` / `['Nama']...` → (Nama, sisa). Butuh kurung tutup.
fn parse_bracket(s: &str) -> Option<(String, &str)> {
    let s = s.strip_prefix('[')?.trim_start();
    let q = s.chars().next()?;
    if q != '"' && q != '\'' {
        return None;
    }
    let end = s[1..].find(q)?;
    let name = s[1..1 + end].to_string();
    let rest = s[1 + end + 1..].trim_start().strip_prefix(']')?;
    Some((name, rest))
}

/// Telusuri `base` lewat segmen `.ident`, `["quoted"]`, `[indeks]`.
/// String kosong → base itu sendiri. Segmen tak dikenal → Null.
fn drill(base: &Value, rest: &str) -> Value {
    let mut cur = base.clone();
    let mut r = rest.trim_start();
    if r.is_empty() {
        return cur;
    }
    loop {
        r = r.trim_start();
        if r.is_empty() {
            return cur;
        }
        if let Some(after) = r.strip_prefix('.') {
            let after = after.trim_start();
            let len: usize = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .map(|c| c.len_utf8())
                .sum();
            if len == 0 {
                return Value::Null;
            }
            let key = &after[..len];
            cur = cur.get(key).cloned().unwrap_or(Value::Null);
            r = &after[len..];
        } else if r.starts_with('[') {
            let inner = r[1..].trim_start();
            if inner
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                let len: usize = inner
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .map(|c| c.len_utf8())
                    .sum();
                let idx: usize = inner[..len].parse().unwrap_or(usize::MAX);
                match inner[len..].trim_start().strip_prefix(']') {
                    Some(rr) => {
                        cur = cur.get(idx).cloned().unwrap_or(Value::Null);
                        r = rr;
                    }
                    None => return Value::Null,
                }
            } else {
                match parse_bracket(r) {
                    Some((key, rest2)) => {
                        cur = cur.get(key.as_str()).cloned().unwrap_or(Value::Null);
                        r = rest2;
                    }
                    None => return Value::Null,
                }
            }
        } else {
            return Value::Null;
        }
    }
}

fn stringify(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx<'a>(item: &'a Value, outputs: &'a HashMap<String, Vec<Value>>) -> ExprContext<'a> {
        ExprContext { item, outputs }
    }

    #[test]
    fn plain_string_passes_through() {
        let item = json!({"a": 1});
        let outputs = HashMap::new();
        assert_eq!(render("halo", &ctx(&item, &outputs)), json!("halo"));
    }

    #[test]
    fn single_placeholder_preserves_type() {
        let item = json!({"n": 41});
        let outputs = HashMap::new();
        assert_eq!(
            render("={{ $json.n }}", &ctx(&item, &outputs)),
            json!(41)
        );
    }

    #[test]
    fn missing_path_is_null_not_error() {
        let item = json!({"a": 1});
        let outputs = HashMap::new();
        assert_eq!(
            render("={{ $json.b.c }}", &ctx(&item, &outputs)),
            Value::Null
        );
    }

    #[test]
    fn node_reference_reads_first_item() {
        let item = json!({});
        let outputs = HashMap::from([("Up".to_string(), vec![json!({"x": 7})])]);
        let c = ctx(&item, &outputs);
        assert_eq!(render("={{ $node[\"Up\"].json.x }}", &c), json!(7));
        assert_eq!(
            render("={{ $node['Up'].first().json.x }}", &c),
            json!(7)
        );
    }

    #[test]
    fn interpolation_mixes_text_and_values() {
        let item = json!({"who": "udi"});
        let outputs = HashMap::new();
        assert_eq!(
            render("halo={{ $json.who }}!", &ctx(&item, &outputs)),
            json!("halo=udi!")
        );
    }

    #[test]
    fn literals_brackets_and_index() {
        let item = json!({"arr": [10, 20]});
        let outputs = HashMap::new();
        let c = ctx(&item, &outputs);
        assert_eq!(render("={{ 1 }}", &c), json!(1));
        assert_eq!(render("={{ \"s\" }}", &c), json!("s"));
        assert_eq!(render("={{ $json.arr[1] }}", &c), json!(20));
        assert_eq!(render("={{ $json[\"arr\"][0] }}", &c), json!(10));
    }
}
