//! TRIAGE + TRANSLATE (spec paragraf 1 tahap 3-4; tabel 1.2 & 1.3).
//!
//! Legenda IR=>ParameterKind (v0.5.3 N2, agent1 #1049):
//!   StringField=>kind::String, NumberField=>kind::Number,
//!   BooleanField=>kind::Boolean, EnumField=>kind::Options,
//!   JsonField=>kind::Json (termasuk array-of-string sampai N1 diputuskan).
//!
//! SideEffect (kernel node.rs:226, varian riil: None|Idempotent|NonIdempotent):
//!   GET/HEAD/OPTIONS/PUT/DELETE => Idempotent (PUT/DELETE per D60);
//!   POST/PATCH => NonIdempotent.
//!
//! DUA MODE (tegangan 1.4 "gagal build" vs manifest ops_rejected diselesaikan
//! eksplisit, dilaporkan ke matt/agent4/agent1):
//!   - translate()        : STRICT fail-loud (default; rejection-corpus INTEG-02).
//!   - translate_collect(): kegagalan LEVEL-OPERASI dicatat ke ops_rejected dan
//!     operasi dilewati (mode golden INTEG-01 — manifest auditable deterministik);
//!     kegagalan LEVEL-SPEC (tanpa operationId) tetap fail-loud.

use crate::error::{codes, CodegenError};
use crate::ingest::SpecDoc;
use crate::resolve::Resolver;
use crate::validate::Operation;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum IrKind {
    String,
    Number,
    Boolean,
    Options,
    Json,
}

#[derive(Debug, Clone)]
pub struct IrField {
    pub name: String,
    pub display_name: String,
    pub kind: IrKind,
    pub required: bool,
    pub default: Option<Value>,
    /// Hanya untuk IrKind::Options: (name, value).
    pub options: Option<Vec<(String, Value)>>,
}

#[derive(Debug, Clone)]
pub struct IrCredential {
    /// httpBasicAuth | httpHeaderAuth | httpQueryAuth (kredensial generik, spec 1.5).
    pub kind: String,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriSideEffect {
    Idempotent,
    NonIdempotent,
}

#[derive(Debug, Clone)]
pub struct TranslatedOp {
    pub operation: Operation,
    pub fields: Vec<IrField>,
    pub credentials: Vec<IrCredential>,
    pub side_effect: TriSideEffect,
    /// op.summary (untuk display name node).
    pub summary: Option<String>,
    /// op.description (untuk deskripsi node; fallback summary).
    pub description: Option<String>,
    /// Content-type body terpilih (deterministik: preferensi lalu urutan kunci terurut).
    pub body_content_type: Option<String>,
    /// Nama path parameter (segmen `{var}`) — untuk routing.
    pub path_params: Vec<String>,
}

/// Operasi yang ditolak pada mode collect (manifest ops_rejected).
#[derive(Debug, Clone)]
pub struct RejectedOp {
    pub operation_id: String,
    pub code: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct TranslateReport {
    pub ops: Vec<TranslatedOp>,
    pub rejected: Vec<RejectedOp>,
}

const METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

/// Mode STRICT: gagal-loud pada kegagalan pertama (level operasi maupun spec).
pub fn translate(doc: &SpecDoc) -> Result<Vec<TranslatedOp>, CodegenError> {
    let mut out = Vec::new();
    for_each_op(doc, |op| {
        out.push(op?);
        Ok(())
    })?;
    if out.is_empty() {
        return Err(CodegenError::new(codes::NOT_OPENAPI, &doc.path, "0 operasi ditemukan"));
    }
    out.sort_by(|a, b| a.operation.operation_id.cmp(&b.operation.operation_id));
    Ok(out)
}

/// Mode COLLECT (golden): kegagalan level-operasi -> ops_rejected; level-spec -> Err.
pub fn translate_collect(doc: &SpecDoc) -> Result<TranslateReport, CodegenError> {
    let mut ops = Vec::new();
    let mut rejected = Vec::new();
    for_each_op(doc, |op| {
        match op {
            Ok(t) => ops.push(t),
            Err(e) => {
                if e.operation.is_none() {
                    return Err(e); // level-spec: tetap fail-loud
                }
                rejected.push(RejectedOp {
                    operation_id: e.operation.clone().unwrap_or_default(),
                    code: e.code.to_string(),
                    reason: e.reason.clone(),
                });
            }
        }
        Ok(())
    })?;
    if ops.is_empty() && rejected.is_empty() {
        return Err(CodegenError::new(codes::NOT_OPENAPI, &doc.path, "0 operasi ditemukan"));
    }
    ops.sort_by(|a, b| a.operation.operation_id.cmp(&b.operation.operation_id));
    rejected.sort_by(|a, b| a.operation_id.cmp(&b.operation_id));
    Ok(TranslateReport { ops, rejected })
}

/// Iterasi seluruh operasi (path-item x method) dan panggil `f` per operasi.
fn for_each_op(
    doc: &SpecDoc,
    mut f: impl FnMut(Result<TranslatedOp, CodegenError>) -> Result<(), CodegenError>,
) -> Result<(), CodegenError> {
    let resolver = Resolver::new(&doc.root, &doc.path);
    let paths = doc
        .root
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| CodegenError::new(codes::NOT_OPENAPI, &doc.path, "paths hilang"))?;
    let ctx = Ctx {
        doc,
        r: &resolver,
        global_security: doc.root.get("security").cloned().unwrap_or(Value::Null),
    };
    for (path, item) in paths {
        for m in METHODS {
            let Some(op) = item.get(m) else { continue };
            let operation_id = op
                .get("operationId")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| derive_id(m, path));
            let result = translate_one(&ctx, item, op, m, path, &operation_id);
            f(result)?;
        }
    }
    Ok(())
}

/// Konteks bersama per-dokumen (mengurangi argumen per-operasi).
struct Ctx<'a> {
    doc: &'a SpecDoc,
    r: &'a Resolver<'a>,
    global_security: Value,
}

fn translate_one(
    ctx: &Ctx,
    item: &Value,
    op: &Value,
    method: &str,
    path: &str,
    operation_id: &str,
) -> Result<TranslatedOp, CodegenError> {
    let mut fields = translate_params(ctx.r, item, op, operation_id)?;
    let credentials = translate_security(ctx.doc, op, &ctx.global_security, operation_id)?;
    let (body_field, content_type) = translate_body(ctx.r, op, operation_id)?;
    if let Some(b) = body_field {
        fields.push(b);
    }
    Ok(TranslatedOp {
        operation: Operation {
            method: method.to_string(),
            path: path.to_string(),
            operation_id: operation_id.to_string(),
        },
        fields,
        credentials,
        side_effect: side_effect(method),
        summary: op
            .get("summary")
            .and_then(Value::as_str)
            .map(str::to_string),
        description: op
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string),
        body_content_type: content_type,
        path_params: path_params(path),
    })
}

fn side_effect(method: &str) -> TriSideEffect {
    match method {
        "post" | "patch" => TriSideEffect::NonIdempotent,
        _ => TriSideEffect::Idempotent,
    }
}

fn path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|s| s.strip_prefix('{').and_then(|s| s.strip_suffix('}')).map(str::to_string))
        .collect()
}

/// Gabungkan parameter level path-item dan level operasi (operasi menang per name+in).
fn translate_params(
    r: &Resolver,
    item: &Value,
    op: &Value,
    op_id: &str,
) -> Result<Vec<IrField>, CodegenError> {
    let mut merged: Vec<(String, String, Value)> = Vec::new(); // (in, name, raw)
    for source in [item.get("parameters"), op.get("parameters")] {
        let Some(arr) = source.and_then(Value::as_array) else { continue };
        for raw in arr {
            let v = r.resolve(raw).map_err(|e| e.op(op_id))?;
            let name = v.get("name").and_then(Value::as_str).unwrap_or("").to_string();
            let in_ = v.get("in").and_then(Value::as_str).unwrap_or("").to_string();
            if name.is_empty() || in_.is_empty() {
                return Err(CodegenError::new(
                    codes::SCHEMA_UNMAPPABLE,
                    r.spec_name(),
                    "parameter tanpa name/in",
                ).op(op_id));
            }
            if let Some(pos) = merged.iter().position(|(i, n, _)| *i == in_ && *n == name) {
                merged[pos] = (in_, name, v); // override level-operasi
            } else {
                merged.push((in_, name, v));
            }
        }
    }
    let mut fields = Vec::new();
    for (in_, _name, v) in merged {
        fields.push(map_parameter(r, &v, &in_, op_id)?);
    }
    Ok(fields)
}

fn map_parameter(r: &Resolver, v: &Value, in_: &str, op_id: &str) -> Result<IrField, CodegenError> {
    let name = v.get("name").and_then(Value::as_str).unwrap_or("").to_string();
    let required = v.get("required").and_then(Value::as_bool).unwrap_or(false);
    let style = v.get("style").and_then(Value::as_str);

    if in_ == "cookie" {
        return Err(CodegenError::new(
            codes::SERIALIZATION_COOKIE,
            r.spec_name(),
            format!("parameter '{name}' in=cookie: routing n8n tidak mengirim cookie"),
        ).op(op_id));
    }
    match in_ {
        "query" => {
            if style == Some("deepObject") {
                return Err(CodegenError::new(
                    codes::SERIALIZATION_DEEPOBJECT,
                    r.spec_name(),
                    format!("parameter '{name}' style=deepObject: tidak ada padanan routing"),
                ).op(op_id));
            }
            if let Some(s) = style {
                if !matches!(s, "form" | "spaceDelimited" | "pipeDelimited") {
                    return Err(CodegenError::new(
                        codes::SCHEMA_UNMAPPABLE,
                        r.spec_name(),
                        format!("parameter '{name}' style='{s}' tidak didukung (hanya form/spaceDelimited/pipeDelimited)"),
                    ).op(op_id));
                }
            }
        }
        "path" | "header" => {
            if let Some(s) = style {
                if s != "simple" {
                    return Err(CodegenError::new(
                        codes::SCHEMA_UNMAPPABLE,
                        r.spec_name(),
                        format!("parameter '{name}' style='{s}' tidak didukung (hanya simple)"),
                    ).op(op_id));
                }
            }
        }
        other => {
            return Err(CodegenError::new(
                codes::SCHEMA_UNMAPPABLE,
                r.spec_name(),
                format!("parameter '{name}' in='{other}' tidak dikenal"),
            ).op(op_id));
        }
    }

    let schema = v.get("schema").cloned().unwrap_or(Value::Null);
    let mut field = map_schema(r, &schema, &name, op_id)?;
    field.required = required;
    if let Some(d) = v.get("default") {
        check_default(r, &field, d, op_id)?;
        field.default = Some(d.clone());
    }
    Ok(field)
}

/// Tabel 1.2: pemetaan skema -> IR. Baris 12-13: oneOf/anyOf & merge konflik = CG-E-201.
fn map_schema(r: &Resolver, schema: &Value, name: &str, op_id: &str) -> Result<IrField, CodegenError> {
    let display = capitalize_words(name);
    let schema = r.resolve(schema).map_err(|e| e.op(op_id))?;

    if schema.get("oneOf").is_some() || schema.get("anyOf").is_some() {
        return Err(unmappable(r, name, "oneOf/anyOf varian campuran — kurasi manual", op_id));
    }
    // allOf: merge tanpa konflik (baris 13).
    if let Some(all) = schema.get("allOf").and_then(Value::as_array) {
        let mut ty: Option<String> = None;
        let mut enum_: Option<Vec<Value>> = None;
        for sub in all {
            let sub = r.resolve(sub).map_err(|e| e.op(op_id))?;
            let st = sub.get("type").and_then(Value::as_str).map(str::to_string);
            match (&ty, &st) {
                (Some(a), Some(b)) if a != b => {
                    return Err(unmappable(r, name, &format!("allOf konflik tipe {a} vs {b}"), op_id));
                }
                (None, Some(b)) => ty = Some(b.clone()),
                _ => {}
            }
            if let Some(e) = sub.get("enum").and_then(Value::as_array) {
                enum_ = Some(match enum_.take() {
                    None => e.clone(),
                    Some(prev) => prev.into_iter().chain(e.clone()).collect(),
                });
            }
        }
        let kind = kind_from_type(&ty, enum_.is_some(), r, name, op_id)?;
        return Ok(IrField {
            name: name.to_string(),
            display_name: display,
            kind,
            required: false,
            default: schema.get("default").cloned(),
            options: enum_.map(|e| opts(&e)),
        });
    }

    let ty = schema.get("type").and_then(Value::as_str).map(str::to_string);
    let enum_ = schema.get("enum").and_then(Value::as_array);
    let kind = kind_from_type(&ty, enum_.is_some(), r, name, op_id)?;
    let default = schema.get("default").cloned();
    let field = IrField {
        name: name.to_string(),
        display_name: display,
        kind,
        required: false,
        default,
        options: enum_.map(|e| opts(e)),
    };
    if let Some(d) = field.default.clone() {
        check_default(r, &field, &d, op_id)?;
    }
    Ok(field)
}

fn kind_from_type(
    ty: &Option<String>,
    has_enum: bool,
    r: &Resolver,
    name: &str,
    op_id: &str,
) -> Result<IrKind, CodegenError> {
    match (ty.as_deref(), has_enum) {
        (Some("string"), true) => Ok(IrKind::Options),
        (Some("string"), false) => Ok(IrKind::String),
        (Some("number"), _) | (Some("integer"), _) => Ok(IrKind::Number),
        (Some("boolean"), _) => Ok(IrKind::Boolean),
        (Some("object"), _) => Ok(IrKind::Json),
        (Some("array"), _) => Ok(IrKind::Json), // N1 #1038: opaque sampai varian kernel string-array
        (None, _) => Err(unmappable(r, name, "skema tanpa type/enum", op_id)),
        (Some(other), _) => Err(unmappable(r, name, &format!("tipe '{other}' tidak dipetakan"), op_id)),
    }
}

fn opts(enum_: &[Value]) -> Vec<(String, Value)> {
    enum_
        .iter()
        .map(|v| {
            let label = match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            (label, v.clone())
        })
        .collect()
}

/// CG-E-302: default wajib konsisten tipe & finit; enum default wajib anggota.
fn check_default(r: &Resolver, field: &IrField, d: &Value, op_id: &str) -> Result<(), CodegenError> {
    let bad = |reason: String| {
        CodegenError::new(codes::DEFAULT_INVALID, r.spec_name(), reason).op(op_id)
    };
    match field.kind {
        IrKind::String | IrKind::Options => {
            if !d.is_string() {
                return Err(bad(format!("default field '{}' bukan string", field.name)));
            }
            if let Some(opts) = &field.options {
                let dv = d.as_str().unwrap();
                if !opts.iter().any(|(l, _)| l.as_str() == dv) {
                    return Err(bad(format!("default '{dv}' bukan anggota enum field '{}'", field.name)));
                }
            }
        }
        IrKind::Number => {
            if !d.is_number() {
                return Err(bad(format!("default field '{}' bukan number", field.name)));
            }
            if let Some(f) = d.as_f64() {
                if !f.is_finite() {
                    return Err(bad(format!("default field '{}' non-finit", field.name)));
                }
            }
        }
        IrKind::Boolean => {
            if !d.is_boolean() {
                return Err(bad(format!("default field '{}' bukan boolean", field.name)));
            }
        }
        IrKind::Json => {}
    }
    Ok(())
}

/// requestBody -> SATU field Json (spec 1.3). Binary/multipart = CG-E-205.
fn translate_body(
    r: &Resolver,
    op: &Value,
    op_id: &str,
) -> Result<(Option<IrField>, Option<String>), CodegenError> {
    let Some(body) = op.get("requestBody") else { return Ok((None, None)) };
    let body = r.resolve(body).map_err(|e| e.op(op_id))?;
    let Some(content) = body.get("content").and_then(Value::as_object) else {
        return Ok((None, None));
    };
    // Deterministik: preferensi lalu kunci terurut (serde_json Map = BTreeMap).
    const PREFERENCE: [&str; 3] = ["application/json", "application/x-www-form-urlencoded", "text/plain"];
    let mut keys: Vec<&String> = content.keys().collect();
    keys.sort();
    let chosen = PREFERENCE
        .iter()
        .find_map(|p| keys.iter().find(|k| k.as_str() == *p).cloned())
        .or_else(|| keys.first().cloned());
    let Some(ct) = chosen else { return Ok((None, None)) };
    if ct.starts_with("multipart/") || ct.starts_with("image/") || ct == "application/octet-stream" {
        return Err(CodegenError::new(
            codes::SERIALIZATION_BINARY,
            r.spec_name(),
            format!("requestBody content-type '{ct}' binary/multipart: butuh binary path data-plane (PRD-1 8.2)"),
        ).op(op_id));
    }
    let required = body.get("required").and_then(Value::as_bool).unwrap_or(false);
    Ok((
        Some(IrField {
            name: "body".to_string(),
            display_name: "Body".to_string(),
            kind: IrKind::Json,
            required,
            default: None,
            options: None,
        }),
        Some(ct.clone()),
    ))
}

/// security: alternatif mappable pertama (deterministik); oauth2/openIdConnect/mutualTLS
/// tanpa alternatif mappable = CG-E-301 (kurasi manual, F-C5).
fn translate_security(
    doc: &SpecDoc,
    op: &Value,
    global: &Value,
    op_id: &str,
) -> Result<Vec<IrCredential>, CodegenError> {
    let sec = op.get("security").unwrap_or(global);
    let Some(alts) = sec.as_array() else {
        return Ok(Vec::new());
    };
    if alts.is_empty() {
        return Ok(Vec::new()); // security: [] = opsional tanpa auth
    }
    let empty = serde_json::Map::new();
    let schemes = doc
        .root
        .get("components")
        .and_then(|c| c.get("securitySchemes"))
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    for alt in alts {
        let Some(alt_obj) = alt.as_object() else { continue };
        for (scheme_name, _) in alt_obj {
            let Some(def) = schemes.get(scheme_name) else {
                return Err(CodegenError::new(
                    codes::SECURITY_UNMAPPED,
                    &doc.path,
                    format!("securityScheme '{scheme_name}' tidak terdefinisi di components.securitySchemes"),
                ).op(op_id));
            };
            let ty = def.get("type").and_then(Value::as_str).unwrap_or("");
            let kind = match ty {
                "http" => match def.get("scheme").and_then(Value::as_str).unwrap_or("") {
                    "basic" => Some("httpBasicAuth"),
                    "bearer" => Some("httpHeaderAuth"),
                    _ => None,
                },
                "apiKey" => match def.get("in").and_then(Value::as_str).unwrap_or("") {
                    "header" => Some("httpHeaderAuth"),
                    "query" => Some("httpQueryAuth"),
                    _ => None,
                },
                _ => None,
            };
            if let Some(kind) = kind {
                return Ok(vec![IrCredential { kind: kind.to_string(), required: true }]);
            }
        }
    }
    Err(CodegenError::new(
        codes::SECURITY_UNMAPPED,
        &doc.path,
        "semua alternatif security tak terpetakan (oauth2/openIdConnect/mutualTLS) — kurasi manual (F-C5)",
    ).op(op_id))
}

fn unmappable(r: &Resolver, name: &str, why: &str, op_id: &str) -> CodegenError {
    CodegenError::new(
        codes::SCHEMA_UNMAPPABLE,
        r.spec_name(),
        format!("skema parameter '{name}': {why}"),
    ).op(op_id)
}

fn capitalize_words(s: &str) -> String {
    let mut out = String::new();
    for (i, part) in s.split(['_', '-']).enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let mut c = part.chars();
        if let Some(f) = c.next() {
            out.extend(f.to_uppercase());
            out.push_str(c.as_str());
        }
    }
    out
}

fn derive_id(method: &str, path: &str) -> String {
    let canonical: String = path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|seg| {
            let seg = seg.trim_start_matches('{').trim_end_matches('}');
            let mut c = seg.chars();
            match c.next() {
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect();
    let m = capitalize_words(method).replace(' ', "");
    format!("{}{}", m, canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::ingest;

    fn fixture(name: &str) -> String {
        format!("{}/../tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
    }

    fn tr(name: &str) -> Vec<TranslatedOp> {
        let doc = ingest(&fixture(name)).unwrap();
        translate(&doc).unwrap()
    }

    fn err_of(name: &str) -> CodegenError {
        let doc = ingest(&fixture(name)).unwrap();
        translate(&doc).unwrap_err()
    }

    #[test]
    fn happy_path_types() {
        let ops = tr("happy-path.json");
        assert_eq!(ops.len(), 1);
        let op = &ops[0];
        assert_eq!(op.operation.operation_id, "createThing");
        assert_eq!(op.side_effect, TriSideEffect::NonIdempotent); // POST
        assert_eq!(op.fields.len(), 6); // q, n, b, e, tags, body
        assert_eq!(op.fields[0].kind, IrKind::String); // q
        assert_eq!(op.fields[1].kind, IrKind::Number); // n
        assert_eq!(op.fields[2].kind, IrKind::Boolean); // b
        assert_eq!(op.fields[3].kind, IrKind::Options); // e (enum)
        assert_eq!(op.fields[3].options.as_ref().unwrap().len(), 2);
        assert_eq!(op.fields[4].kind, IrKind::Json); // tags: array string (N1)
        assert_eq!(op.fields[5].name, "body");
        assert_eq!(op.body_content_type.as_deref(), Some("application/json"));
        // bearer -> httpHeaderAuth
        assert_eq!(op.credentials.len(), 1);
        assert_eq!(op.credentials[0].kind, "httpHeaderAuth");
    }

    #[test]
    fn get_is_idempotent_path_params() {
        let ops = tr("get-with-path.json");
        assert_eq!(ops[0].side_effect, TriSideEffect::Idempotent);
        assert_eq!(ops[0].path_params, vec!["id"]);
        assert_eq!(ops[0].fields[0].name, "id");
        assert!(ops[0].fields[0].required);
    }

    #[test]
    fn reject_oneof() {
        let e = err_of("param-oneof.json");
        assert_eq!(e.code, "CG-E-201");
        assert!(e.operation.is_some());
    }

    #[test]
    fn reject_deepobject() {
        let e = err_of("param-deepobject.json");
        assert_eq!(e.code, "CG-E-203");
    }

    #[test]
    fn reject_cookie() {
        let e = err_of("param-cookie.json");
        assert_eq!(e.code, "CG-E-204");
    }

    #[test]
    fn reject_multipart_body() {
        let e = err_of("body-multipart.json");
        assert_eq!(e.code, "CG-E-205");
    }

    #[test]
    fn reject_oauth_only() {
        let e = err_of("security-oauth.json");
        assert_eq!(e.code, "CG-E-301");
        assert!(e.reason.contains("kurasi manual"));
    }

    #[test]
    fn reject_default_mismatch() {
        let e = err_of("default-mismatch.json");
        assert_eq!(e.code, "CG-E-302");
    }

    #[test]
    fn reject_cyclic_ref_in_param() {
        let e = err_of("cyclic-ref.json");
        assert_eq!(e.code, "CG-E-202");
        assert!(e.reason.contains("siklik"));
    }

    #[test]
    fn all_of_merge_ok() {
        let ops = tr("all-of-ok.json");
        assert_eq!(ops[0].fields[0].kind, IrKind::Number);
    }

    #[test]
    fn collect_mode_skips_and_records() {
        // 2 op: satu sehat, satu oneOf -> collect melewati yang rusak + mencatat.
        let doc = ingest(&fixture("collect-mixed.json")).unwrap();
        let rep = translate_collect(&doc).unwrap();
        assert_eq!(rep.ops.len(), 1);
        assert_eq!(rep.rejected.len(), 1);
        assert_eq!(rep.rejected[0].code, "CG-E-201");
        assert_eq!(rep.ops[0].operation.operation_id, "listOk");
    }

    #[test]
    fn frankfurter_golden_translates() {
        let path = format!("{}/../specs/frankfurter.openapi.json", env!("CARGO_MANIFEST_DIR"));
        let doc = ingest(&path).unwrap();
        let ops = translate(&doc).unwrap();
        assert_eq!(ops.len(), 5);
    }

    #[test]
    fn github_golden_strict_fails_on_oneof_param() {
        // Tegangan 1.4 vs manifest: strict GAGAL pada param oneOf (bukti kebutuhan mode collect).
        let path = format!("{}/../specs/github-rest.openapi.json", env!("CARGO_MANIFEST_DIR"));
        let doc = ingest(&path).unwrap();
        let err = translate(&doc).unwrap_err();
        assert_eq!(err.code, "CG-E-201");
    }

    #[test]
    fn github_golden_collect_reports() {
        let path = format!("{}/../specs/github-rest.openapi.json", env!("CARGO_MANIFEST_DIR"));
        let doc = ingest(&path).unwrap();
        let rep = translate_collect(&doc).unwrap();
        // Angka dipatok setelah pengukuran pertama (deterministik: kunci BTreeMap + sort).
        assert_eq!(rep.ops.len() + rep.rejected.len(), 1225);
        assert!(!rep.rejected.is_empty()); // param oneOf dll dicatat, bukan didiamkan
        // ditolak = deterministik & terurut
        assert!(rep.rejected.windows(2).all(|w| w[0].operation_id <= w[1].operation_id));
        // operasi yang lolos tetap terurut kanonik
        assert!(rep.ops.windows(2).all(|w| w[0].operation.operation_id <= w[1].operation.operation_id));
    }
}
