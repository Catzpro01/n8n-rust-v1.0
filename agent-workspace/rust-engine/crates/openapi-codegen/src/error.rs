//! Kode kegagalan codegen — AGENT9-INTEGRATION-SPEC v0.5.3 paragraf 1.6.
//! Format pesan (spec 1.4): `spec:<file> operation:<operationId> kode:<CG-E-xxx> alasan:<eksak>`.
//! Kode terdaftar: tidak boleh digunakan ulang; boleh bertambah.

use std::fmt;

#[derive(Debug, Clone)]
pub struct CodegenError {
    pub code: &'static str,
    pub spec: String,
    pub operation: Option<String>,
    pub reason: String,
}

impl CodegenError {
    pub fn new(code: &'static str, spec: impl Into<String>, reason: impl Into<String>) -> Self {
        Self { code, spec: spec.into(), operation: None, reason: reason.into() }
    }

    /// Lampirkan operationId (untuk kegagalan level-operasi).
    pub fn op(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.operation {
            Some(op) => write!(
                f,
                "spec:{} operation:{} kode:{} alasan:{}",
                self.spec, op, self.code, self.reason
            ),
            None => write!(f, "spec:{} kode:{} alasan:{}", self.spec, self.code, self.reason),
        }
    }
}

impl std::error::Error for CodegenError {}

/// Registri kode (spec 1.6). Tidak boleh digunakan ulang.
pub mod codes {
    /// Dokumen bukan OpenAPI 3.0.x/3.1.x valid (parse/struktural).
    pub const NOT_OPENAPI: &str = "CG-E-101";
    /// servers ber-template variable tanpa default/enum.
    pub const SERVER_VAR_NO_DEFAULT: &str = "CG-E-102";
    /// operationId hilang/duplikat dan tak dapat diderivasi deterministik.
    pub const OPERATION_ID_UNRESOLVABLE: &str = "CG-E-103";
    /// Kedalaman nesting JSON melewati batas (Ruling 39b; fail-closed).
    pub const JSON_DEPTH_EXCEEDED: &str = "CG-E-104";
    /// Skema parameter tidak dapat dipetakan eksak (tabel 1.2 baris 12-13).
    pub const SCHEMA_UNMAPPABLE: &str = "CG-E-201";
    /// $ref siklik / lintas-file tak terdaftar.
    pub const REF_CYCLIC_OR_EXTERNAL: &str = "CG-E-202";
    /// Serialisasi deepObject tidak didukung.
    pub const SERIALIZATION_DEEPOBJECT: &str = "CG-E-203";
    /// Serialisasi cookie tidak didukung.
    pub const SERIALIZATION_COOKIE: &str = "CG-E-204";
    /// Body binary/multipart tidak didukung.
    pub const SERIALIZATION_BINARY: &str = "CG-E-205";
    /// securityScheme tak terpetakan.
    pub const SECURITY_UNMAPPED: &str = "CG-E-301";
    /// Default non-finit / tak konsisten tipe.
    pub const DEFAULT_INVALID: &str = "CG-E-302";
}
