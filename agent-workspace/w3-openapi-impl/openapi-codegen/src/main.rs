//! CLI openapi-codegen.
//! Pemakaian:
//!   openapi-codegen --spec <file.json>             (M1: INGEST+VALIDATE)
//!   openapi-codegen --spec <file.json> --translate (M2: + TRIAGE/TRANSLATE)
//! EMIT menyusul M3.

#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let mut spec: Option<String> = None;
    let mut do_translate = false;
    let mut emit_dir: Option<String> = None;
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--spec" => spec = it.next().cloned(),
            "--translate" => do_translate = true,
            "--emit" => emit_dir = it.next().cloned(),
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("argumen tak dikenal: {other}");
                print_help();
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(spec) = spec else {
        print_help();
        return ExitCode::FAILURE;
    };
    match run(&spec, do_translate, emit_dir.as_deref()) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run(spec: &str, do_translate: bool, emit_dir: Option<&str>) -> Result<String, openapi_codegen::CodegenError> {
    let doc = openapi_codegen::ingest::ingest(spec)?;
    let ops = openapi_codegen::validate::validate(&doc)?;
    if !do_translate {
        return Ok(format!(
            "spec: {}\n  openapi: {}\n  sha256(raw-bytes): {}\n  ukuran: {} bytes\n  operasi: {} (valid, terurut kanonik)",
            doc.path,
            doc.root.get("openapi").and_then(|v| v.as_str()).unwrap_or("?"),
            doc.sha256_hex,
            doc.size_bytes,
            ops.len(),
        ));
    }
    let rep = openapi_codegen::translate::translate_collect(&doc)?;
    if let Some(dir) = emit_dir {
        // VALIDATE dulu (servers/dup-id) sebelum EMIT menulis artefak.
        openapi_codegen::validate::validate(&doc)?;
        let art = openapi_codegen::emit::emit(
            &doc,
            &rep,
            env!("CARGO_PKG_VERSION"),
        )?;
        let stem = art.stem.clone();
        std::fs::create_dir_all(dir)
            .map_err(|e| openapi_codegen::CodegenError::new("CG-E-101", spec, format!("buat dir {dir}: {e}")))?;
        let nodes_path = format!("{dir}/{stem}.rs");
        let manifest_path = format!("{dir}/{stem}.manifest.json");
        std::fs::write(&nodes_path, &art.nodes_rs)
            .map_err(|e| openapi_codegen::CodegenError::new("CG-E-101", spec, format!("tulis {nodes_path}: {e}")))?;
        std::fs::write(&manifest_path, &art.manifest_json)
            .map_err(|e| openapi_codegen::CodegenError::new("CG-E-101", spec, format!("tulis {manifest_path}: {e}")))?;
        return Ok(format!(
            "EMIT: {nodes_path} ({} bytes) + {manifest_path} ({} bytes)\n  operasi OK: {} | ditolak: {}",
            art.nodes_rs.len(), art.manifest_json.len(), rep.ops.len(), rep.rejected.len()
        ));
    }
    let tops = &rep.ops;
    let n_fields: usize = tops.iter().map(|t| t.fields.len()).sum();
    let n_cred: usize = tops.iter().filter(|t| !t.credentials.is_empty()).count();
    let n_nonidem: usize = tops.iter().filter(|t| t.side_effect == openapi_codegen::TriSideEffect::NonIdempotent).count();
    let mut codes: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for r in &rep.rejected {
        *codes.entry(&r.code).or_default() += 1;
    }
    let codes_str = if codes.is_empty() {
        "-".to_string()
    } else {
        codes.iter().map(|(c, n)| format!("{c}x{n}")).collect::<Vec<_>>().join(", ")
    };
    Ok(format!(
        "spec: {}\n  sha256(raw-bytes): {}\n  operasi OK: {} (terurut kanonik) | DITOLAK: {} ({})\n  field IR total: {}\n  operasi berkredensial: {}\n  operasi non-idempotent (POST/PATCH): {}\n  contoh: {} [{}] fields={} creds={} side={:?}",
        doc.path,
        doc.sha256_hex,
        tops.len(),
        rep.rejected.len(),
        codes_str,
        n_fields,
        n_cred,
        n_nonidem,
        tops.first().map(|t| t.operation.operation_id.as_str()).unwrap_or("-"),
        tops.first().map(|t| t.operation.method.as_str()).unwrap_or("-"),
        tops.first().map(|t| t.fields.len()).unwrap_or(0),
        tops.first().map(|t| t.credentials.len()).unwrap_or(0),
        tops.first().map(|t| t.side_effect).unwrap_or(openapi_codegen::TriSideEffect::Idempotent),
    ))
}

fn print_help() {
    println!("openapi-codegen (W3-OPENAPI-IMPL, agent9) v0.1-M2");
    println!("  --spec <file.json>   proses spec (INGEST+VALIDATE)");
    println!("  --translate          tambah TRIAGE+TRANSLATE");
    println!("  --emit <dir>         tulis kode node + manifest verifikasi ke <dir>");
}
