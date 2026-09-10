//! n8n-cli: validate / explain / run / nodes — premium UX.
//! Tanpa clap (ringan), tapi dengan colored output, help, version.

use n8n_core::Workflow;
use n8n_engine::{Engine, Level, Registry};
use n8n_nodes::register_all;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("\x1b[31merror:\x1b[0m {message}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") => cmd_validate(args.get(2)),
        Some("explain") => cmd_explain(args.get(2)),
        Some("run") => {
            let save = args
                .iter()
                .position(|a| a == "--save")
                .and_then(|i| args.get(i + 1).map(String::as_str));
            let verbose = args.iter().any(|a| a == "--verbose" || a == "-v");
            cmd_run(args.get(2), save, verbose)
        }
        Some("nodes") => cmd_nodes(),
        Some("--version") | Some("-V") | Some("version") => {
            println!("n8n-cli {VERSION}");
            Ok(())
        }
        Some("--help") | Some("-h") | Some("help") | None => {
            usage();
            if args.get(1).is_none() {
                Ok(())
            } else {
                Err("perintah tak dikenal".to_string())
            }
        }
        _ => {
            usage();
            Err("perintah tak dikenal".to_string())
        }
    }
}

fn usage() {
    eprintln!(
        r#"
\x1b[1mn8n-cli {VERSION}\x1b[0m — Rust port of n8n, ringan & cepat

\x1b[33mUSAGE:\x1b[0m
  n8n-cli validate <workflow.json>              Lint workflow (error + warning)
  n8n-cli explain <workflow.json>               Rencana urutan eksekusi (dry-run)
  n8n-cli run <workflow.json> [--save <report.json>] [-v]  Eksekusi + laporan
  n8n-cli nodes                                 Daftar tipe node terdaftar
  n8n-cli --version / --help

\x1b[33mEXAMPLES:\x1b[0m
  n8n-cli validate n8n-rust/fixtures/manual-to-set.json
  n8n-cli run n8n-rust/fixtures/manual-to-set.json --save report.json -v
  n8n-cli nodes | grep http

\x1b[33mENV:\x1b[0m
  RUST_LOG=info  (future: tracing)
"#
    );
}

fn need_path(arg: Option<&String>) -> Result<&str, String> {
    arg.map(String::as_str)
        .ok_or_else(|| "kurang argumen: path workflow.json".to_string())
}

fn load_workflow(path: &str) -> Result<Workflow, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("baca '{path}' gagal: {e}"))?;
    Workflow::from_json(&raw).map_err(|e| format!("parse '{path}' gagal: {e}"))
}

fn full_registry() -> Registry {
    let mut registry = Registry::default();
    register_all(&mut registry);
    registry
}

fn cmd_validate(arg: Option<&String>) -> Result<(), String> {
    let wf = load_workflow(need_path(arg)?)?;
    let diags = Engine::lint(&wf, &full_registry());
    let errors = diags.iter().filter(|d| d.level == Level::Error).count();
    let warns = diags.iter().filter(|d| d.level == Level::Warning).count();

    if diags.is_empty() {
        println!(
            "\x1b[32m✓ OK\x1b[0m: '{}' bersih ({} node, {} edge)",
            wf.name,
            wf.nodes.len(),
            wf.edge_count()
        );
    } else {
        for d in &diags {
            let (tag, color) = match d.level {
                Level::Error => ("ERROR", "\x1b[31m"),
                Level::Warning => ("WARN ", "\x1b[33m"),
            };
            let node = d
                .node
                .as_ref()
                .map(|n| format!(" \x1b[2m({n})\x1b[0m"))
                .unwrap_or_default();
            println!("{color}[{tag}]\x1b[0m {}{node}", d.message);
        }
        println!(
            "\n\x1b[1m{} error, {} warning\x1b[0m — workflow '{}' ({} node, {} edge)",
            errors,
            warns,
            wf.name,
            wf.nodes.len(),
            wf.edge_count()
        );
    }

    if errors > 0 {
        return Err(format!("{errors} error"));
    }
    Ok(())
}

fn cmd_explain(arg: Option<&String>) -> Result<(), String> {
    let wf = load_workflow(need_path(arg)?)?;
    match Engine::explain(&wf) {
        Ok(order) => {
            println!("\x1b[32m✓ rencana:\x1b[0m {}", order.join(" \x1b[2m→\x1b[0m "));
            for (i, name) in order.iter().enumerate() {
                println!("  {}. {}", i + 1, name);
            }
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

fn cmd_run(arg: Option<&String>, save: Option<&str>, verbose: bool) -> Result<(), String> {
    let wf = load_workflow(need_path(arg)?)?;
    println!(
        "\x1b[2m▶ running '{}' ({} node, {} edge)...\x1b[0m",
        wf.name,
        wf.nodes.len(),
        wf.edge_count()
    );
    let t0 = std::time::Instant::now();
    let report = Engine::run(&wf, &full_registry()).map_err(|e| e.to_string())?;
    let wall = t0.elapsed().as_millis();

    println!(
        "\x1b[32m✓ urutan:\x1b[0m {}",
        report.order.join(" \x1b[2m→\x1b[0m ")
    );
    let per: Vec<String> = report
        .order
        .iter()
        .map(|n| {
            format!(
                "{}={}ms",
                n,
                report.durations_ms.get(n).copied().unwrap_or(0)
            )
        })
        .collect();
    println!(
        "\x1b[2mwaktu:\x1b[0m {} (total {}ms, wall {}ms)",
        per.join(", "),
        report.total_ms,
        wall
    );

    if verbose {
        let out = serde_json::to_string_pretty(&report.outputs).map_err(|e| e.to_string())?;
        println!("\n{out}");
    } else {
        // summary
        for name in &report.order {
            if let Some(branches) = report.outputs.get(name) {
                let count: usize = branches.iter().map(|b| b.len()).sum();
                println!("  \x1b[2m• {name}: {count} item(s)\x1b[0m");
            }
        }
        println!("\n\x1b[2m(gunakan -v untuk full output)\x1b[0m");
    }

    if let Some(path) = save {
        let pretty = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
        std::fs::write(path, pretty).map_err(|e| format!("simpan '{path}' gagal: {e}"))?;
        println!("\x1b[32m✓ tersimpan:\x1b[0m {path}");
    }
    Ok(())
}

fn cmd_nodes() -> Result<(), String> {
    let reg = full_registry();
    println!("\x1b[1m{} node types:\x1b[0m", reg.len());
    for t in reg.types() {
        println!("  • {t}");
    }
    Ok(())
}
