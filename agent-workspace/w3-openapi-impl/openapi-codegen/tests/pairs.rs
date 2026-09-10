//! INTEG-02 (rejection corpus, >=20 kasus) + INTEG-03 (translation pairs):
//! tiap baris SUPPORTED tabel 1.2 punya pasangan terima (+kind eksak),
//! tiap polan REJECT punya kasus dengan kode CG-E eksak.

use openapi_codegen::error::CodegenError;
use openapi_codegen::ingest::SpecDoc;
use openapi_codegen::translate::{translate, IrKind};
use serde_json::{json, Value};

/// SpecDoc inline (tanpa file) — path kanonik dummy.
fn doc(paths: Value) -> SpecDoc {
    SpecDoc {
        path: "inline.json".into(),
        sha256_hex: "0".repeat(64),
        size_bytes: 0,
        root: json!({"openapi": "3.0.0", "info": {"title": "t", "version": "1"}, "paths": paths}),
    }
}

fn one_param(schema: Value) -> Result<Vec<openapi_codegen::translate::TranslatedOp>, CodegenError> {
    let d = doc(json!({"/x": {"get": {"operationId": "op",
        "parameters": [{"name": "p", "in": "query", "schema": schema}],
        "responses": {"200": {"description": "ok"}}}}}));
    translate(&d)
}

fn kind_of(ops: &[openapi_codegen::translate::TranslatedOp]) -> IrKind {
    ops[0].fields[0].kind.clone()
}

// ============ INTEG-03: pasangan TERIMA (baris SUPPORTED tabel 1.2) ============

#[test]
fn pair_row1_string() {
    assert_eq!(kind_of(&one_param(json!({"type": "string"})).unwrap()), IrKind::String);
}

#[test]
fn pair_row2_number() {
    assert_eq!(kind_of(&one_param(json!({"type": "number"})).unwrap()), IrKind::Number);
}

#[test]
fn pair_row3_integer() {
    assert_eq!(kind_of(&one_param(json!({"type": "integer"})).unwrap()), IrKind::Number);
}

#[test]
fn pair_row4_boolean() {
    assert_eq!(kind_of(&one_param(json!({"type": "boolean"})).unwrap()), IrKind::Boolean);
}

#[test]
fn pair_row5_enum() {
    let ops = one_param(json!({"type": "string", "enum": ["a", "b", "c"]})).unwrap();
    assert_eq!(kind_of(&ops), IrKind::Options);
    assert_eq!(ops[0].fields[0].options.as_ref().unwrap().len(), 3);
}

#[test]
fn pair_row6_object() {
    assert_eq!(kind_of(&one_param(json!({"type": "object"})).unwrap()), IrKind::Json);
}

#[test]
fn pair_row7_array_of_string() {
    // N1 #1038: opaque Json sampai varian kernel string-array diputuskan.
    assert_eq!(
        kind_of(&one_param(json!({"type": "array", "items": {"type": "string"}})).unwrap()),
        IrKind::Json
    );
}

#[test]
fn pair_row8_array_of_number() {
    assert_eq!(
        kind_of(&one_param(json!({"type": "array", "items": {"type": "number"}})).unwrap()),
        IrKind::Json
    );
}

#[test]
fn pair_row15_ref_infile() {
    let d = doc(json!({"/x": {"get": {"operationId": "op",
        "parameters": [{"name": "p", "in": "query", "schema": {"$ref": "#/x-schemas/S"}}],
        "responses": {"200": {"description": "ok"}}}}}));
    // sisipkan components ke root
    let mut root = d.root;
    root["x-schemas"] = json!({"S": {"type": "string"}});
    let d = SpecDoc { root, ..d };
    assert_eq!(kind_of(&translate(&d).unwrap()), IrKind::String);
}

#[test]
fn pair_row13_all_of_merge() {
    assert_eq!(
        kind_of(&one_param(json!({"allOf": [{"type": "number"}, {"minimum": 0}]})).unwrap()),
        IrKind::Number
    );
}

#[test]
fn pair_serialization_accept_styles() {
    for style in ["form", "spaceDelimited", "pipeDelimited"] {
        let d = doc(json!({"/x": {"get": {"operationId": "op",
            "parameters": [{"name": "p", "in": "query", "style": style,
                "schema": {"type": "array", "items": {"type": "string"}}}],
            "responses": {"200": {"description": "ok"}}}}}));
        assert!(translate(&d).is_ok(), "style {style} harus diterima");
    }
}

#[test]
fn pair_body_json_accepted() {
    let d = doc(json!({"/x": {"post": {"operationId": "op",
        "requestBody": {"content": {"application/json": {"schema": {"type": "object"}}}},
        "responses": {"200": {"description": "ok"}}}}}));
    let ops = translate(&d).unwrap();
    assert_eq!(ops[0].fields[0].name, "body");
    assert_eq!(ops[0].fields[0].kind, IrKind::Json);
    assert_eq!(ops[0].body_content_type.as_deref(), Some("application/json"));
}

// ============ INTEG-03: pasangan TOLAK (kode eksak) ============

#[test]
fn pair_row12_oneof_rejected() {
    assert_eq!(one_param(json!({"oneOf": [{"type": "string"}, {"type": "number"}]})).unwrap_err().code, "CG-E-201");
}

#[test]
fn pair_row12_anyof_rejected() {
    assert_eq!(one_param(json!({"anyOf": [{"type": "string"}, {"type": "boolean"}]})).unwrap_err().code, "CG-E-201");
}

#[test]
fn pair_row13_all_of_conflict_rejected() {
    assert_eq!(
        one_param(json!({"allOf": [{"type": "string"}, {"type": "number"}]})).unwrap_err().code,
        "CG-E-201"
    );
}

#[test]
fn pair_no_type_rejected() {
    assert_eq!(one_param(json!({"description": "tanpa type"})).unwrap_err().code, "CG-E-201");
}

#[test]
fn pair_style_unknown_rejected() {
    let d = doc(json!({"/x": {"get": {"operationId": "op",
        "parameters": [{"name": "p", "in": "query", "style": "label", "schema": {"type": "string"}}],
        "responses": {"200": {"description": "ok"}}}}}));
    assert_eq!(translate(&d).unwrap_err().code, "CG-E-201");
}

#[test]
fn pair_deepobject_rejected() {
    let d = doc(json!({"/x": {"get": {"operationId": "op",
        "parameters": [{"name": "p", "in": "query", "style": "deepObject", "schema": {"type": "object"}}],
        "responses": {"200": {"description": "ok"}}}}}));
    assert_eq!(translate(&d).unwrap_err().code, "CG-E-203");
}

#[test]
fn pair_cookie_rejected() {
    let d = doc(json!({"/x": {"get": {"operationId": "op",
        "parameters": [{"name": "p", "in": "cookie", "schema": {"type": "string"}}],
        "responses": {"200": {"description": "ok"}}}}}));
    assert_eq!(translate(&d).unwrap_err().code, "CG-E-204");
}

#[test]
fn pair_body_binary_rejected() {
    let d = doc(json!({"/x": {"post": {"operationId": "op",
        "requestBody": {"content": {"application/octet-stream": {"schema": {"type": "string"}}}},
        "responses": {"200": {"description": "ok"}}}}}));
    assert_eq!(translate(&d).unwrap_err().code, "CG-E-205");
}

#[test]
fn pair_security_undefined_rejected() {
    let d = doc(json!({"/x": {"get": {"operationId": "op", "security": [{"hantu": []}],
        "responses": {"200": {"description": "ok"}}}}}));
    assert_eq!(translate(&d).unwrap_err().code, "CG-E-301");
}

#[test]
fn pair_default_enum_mismatch_rejected() {
    assert_eq!(
        one_param(json!({"type": "string", "enum": ["a", "b"], "default": "z"})).unwrap_err().code,
        "CG-E-302"
    );
}

// ============ INTEG-02: rejection corpus file (>=20 kasus, kode eksak) ============

#[test]
fn integ02_rejection_corpus() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let corpus: &[(&str, &str)] = &[
        ("not-json.txt", "CG-E-101"),
        ("wrong-version.json", "CG-E-101"),
        ("no-paths.json", "CG-E-101"),
        ("server-var-no-default.json", "CG-E-102"),
        ("dup-operation-id.json", "CG-E-103"),
        ("depth-65.json", "CG-E-104"),
        ("param-oneof.json", "CG-E-201"),
        ("param-anyof.json", "CG-E-201"),
        ("all-of-conflict.json", "CG-E-201"),
        ("schema-no-type.json", "CG-E-201"),
        ("param-style-label.json", "CG-E-201"),
        ("in-unknown.json", "CG-E-201"),
        ("cyclic-ref.json", "CG-E-202"),
        ("external-ref-param.json", "CG-E-202"),
        ("param-deepobject.json", "CG-E-203"),
        ("param-cookie.json", "CG-E-204"),
        ("body-multipart.json", "CG-E-205"),
        ("body-image.json", "CG-E-205"),
        ("security-oauth.json", "CG-E-301"),
        ("security-undefined.json", "CG-E-301"),
        ("default-mismatch.json", "CG-E-302"),
        ("default-enum-mismatch.json", "CG-E-302"),
    ];
    assert!(corpus.len() >= 20, "INTEG-02 minta >=20, kini {}", corpus.len());
    for (file, expected) in corpus {
        let path = format!("{manifest_dir}/../tests/fixtures/{file}");
        let err = match openapi_codegen::ingest::ingest(&path) {
            Err(e) => e, // gagal level INGEST (mis. not-json) juga bagian corpus
            Ok(doc) => match openapi_codegen::validate::validate(&doc) {
                Err(e) => e, // urutan pipeline: VALIDATE sebelum TRANSLATE
                Ok(_) => match translate(&doc) {
                    Err(e) => e,
                    Ok(_) => panic!("fixture {file} harus DITOLAK"),
                },
            },
        };
        assert_eq!(&err.code, expected, "fixture {file}: kode salah");
        // format pesan eksak (spec 1.4)
        assert!(err.to_string().starts_with("spec:"), "format pesan 1.4: {}", err);
    }
}
