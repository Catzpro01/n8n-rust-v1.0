//! Mini ekspresi `={{ }}` (subset n8n, tanpa dependensi).
//!
//! v0.3.0 didukung:
//! - path: `$json.a.b`, `$json["a"]`, `$json.arr[0]`
//! - antar-node: `$node["Nama"].json.a.b` (+`.first()`), cabang output 0
//! - operator: `==` `!=` `>` `<` `>=` `<=` (angka numerik, sisanya string)
//! - fungsi satu-argumen: `len(x)`, `upper(x)`, `lower(x)`
//! - literal: `"str"`, `123`, `1.5`, `true`, `false`, `null`
//! - selain itu → Null lunak (tak ada operator unary, tak ada escape `\"`).
//!
//! Render: bila SELURUH string adalah satu `={{...}}`, nilai asli
//! dipertahankan tipenya; bila template campuran, interpolasi jadi string
//! (missing → string kosong).

use serde_json::{json, Value};
use std::collections::HashMap;

pub struct ExprContext<'a> {
    pub item: &'a Value,
    /// Output per nama node → per cabang → items. `$node` membaca cabang 0.
    pub outputs: &'a HashMap<String, Vec<Vec<Value>>>,
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

/// Render rekursif satu nilai: string → template, array/object → per elemen.
pub fn render_value(v: &Value, ctx: &ExprContext) -> Value {
    match v {
        Value::String(s) => render(s, ctx),
        Value::Array(a) => Value::Array(a.iter().map(|x| render_value(x, ctx)).collect()),
        Value::Object(o) => Value::Object(
            o.iter()
                .map(|(k, x)| (k.clone(), render_value(x, ctx)))
                .collect(),
        ),
        other => other.clone(),
    }
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
    if let Some((l, op, r)) = split_operator(e) {
        return apply_op(l, op, r, ctx);
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
            .and_then(|branches| branches.first())
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
    if e.ends_with(')') {
        if let Some(open) = e.find('(') {
            let name = e[..open].trim();
            if !name.is_empty()
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                let arg = e[open + 1..e.len() - 1].trim();
                let v = eval(arg, ctx);
                return apply_func(name, &v);
            }
        }
    }
    serde_json::from_str(e).unwrap_or(Value::Null)
}

/// Belah di operator perbandingan level-0 pertama (di luar kutip/kurung).
/// Operator 2-huruf dicek dulu supaya `>=` tak terbaca sebagai `>`.
fn split_operator(e: &str) -> Option<(&str, &str, &str)> {
    let bytes = e.as_bytes();
    let mut quote: Option<u8> = None;
    let mut depth = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' => {
                quote = Some(c);
                i += 1;
            }
            b'(' | b'[' => {
                depth += 1;
                i += 1;
            }
            b')' | b']' => {
                depth -= 1;
                i += 1;
            }
            _ => {
                if depth == 0 {
                    let rest = &e[i..];
                    for op in ["==", "!=", ">=", "<=", ">", "<"] {
                        if rest.starts_with(op) {
                            return Some((e[..i].trim(), op, e[i + op.len()..].trim()));
                        }
                    }
                }
                i += 1;
            }
        }
    }
    None
}

fn apply_op(l: &str, op: &str, r: &str, ctx: &ExprContext) -> Value {
    let lv = eval(l, ctx);
    let rv = eval(r, ctx);
    let b = match op {
        "==" => lv == rv,
        "!=" => lv != rv,
        _ => compare_order(&lv, &rv, op),
    };
    Value::Bool(b)
}

fn compare_order(lv: &Value, rv: &Value, op: &str) -> bool {
    match (lv, rv) {
        (Value::Number(a), Value::Number(b)) => {
            let (x, y) = (
                a.as_f64().unwrap_or(f64::NAN),
                b.as_f64().unwrap_or(f64::NAN),
            );
            match op {
                ">" => x > y,
                "<" => x < y,
                ">=" => x >= y,
                "<=" => x <= y,
                _ => false,
            }
        }
        _ => {
            let (x, y) = (stringify(lv), stringify(rv));
            match op {
                ">" => x > y,
                "<" => x < y,
                ">=" => x >= y,
                "<=" => x <= y,
                _ => false,
            }
        }
    }
}

fn apply_func(name: &str, v: &Value) -> Value {
    match name {
        "len" => match v {
            Value::String(s) => json!(s.chars().count()),
            Value::Array(a) => json!(a.len()),
            Value::Object(o) => json!(o.len()),
            _ => Value::Null,
        },
        "upper" => v
            .as_str()
            .map(|s| json!(s.to_uppercase()))
            .unwrap_or(Value::Null),
        "lower" => v
            .as_str()
            .map(|s| json!(s.to_lowercase()))
            .unwrap_or(Value::Null),
        _ => Value::Null,
    }
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

    fn ctx<'a>(item: &'a Value, outputs: &'a HashMap<String, Vec<Vec<Value>>>) -> ExprContext<'a> {
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
        let outputs = HashMap::from([("Up".to_string(), vec![vec![json!({"x": 7})]])]);
        let c = ctx(&item, &outputs);
        assert_eq!(render("={{ $node[\"Up\"].json.x }}", &c), json!(7));
        assert_eq!(
            render("={{ $node['Up'].first().json.x }}", &c),
            json!(7)
        );
    }

    #[test]
    fn node_reference_reads_branch_zero_only() {
        let item = json!({});
        let outputs = HashMap::from([(
            "If".to_string(),
            vec![vec![json!({"x": 7})], vec![json!({"x": 9})]],
        )]);
        assert_eq!(
            render("={{ $node[\"If\"].json.x }}", &ctx(&item, &outputs)),
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

    #[test]
    fn comparison_operators() {
        let item = json!({"age": 20, "name": "udi"});
        let outputs = HashMap::new();
        let c = ctx(&item, &outputs);
        assert_eq!(render("={{ $json.age > 18 }}", &c), json!(true));
        assert_eq!(render("={{ $json.age < 18 }}", &c), json!(false));
        assert_eq!(render("={{ $json.age >= 20 }}", &c), json!(true));
        assert_eq!(render("={{ $json.age <= 19 }}", &c), json!(false));
        assert_eq!(render("={{ $json.age == 20 }}", &c), json!(true));
        assert_eq!(render("={{ $json.age != 20 }}", &c), json!(false));
        assert_eq!(render("={{ $json.name == \"udi\" }}", &c), json!(true));
        assert_eq!(render("={{ $json.name > \"a\" }}", &c), json!(true));
    }

    #[test]
    fn functions_len_upper_lower() {
        let item = json!({"name": "udi", "arr": [1, 2, 3]});
        let outputs = HashMap::new();
        let c = ctx(&item, &outputs);
        assert_eq!(render("={{ len($json.arr) }}", &c), json!(3));
        assert_eq!(render("={{ len($json.name) }}", &c), json!(3));
        assert_eq!(render("={{ upper($json.name) }}", &c), json!("UDI"));
        assert_eq!(render("={{ lower(\"A B\") }}", &c), json!("a b"));
        assert_eq!(render("={{ len($json.arr) > 2 }}", &c), json!(true));
        assert_eq!(render("={{ nope($json.name) }}", &c), Value::Null);
    }
}
