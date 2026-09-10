//! n8n-cli: `validate` / `run` / `nodes`.
//! Argumen diurai manual (tanpa clap — ringan).

use n8n_core::Workflow;
use n8n_engine::{Engine, Registry};
use n8n_nodes::register_all;
use std::process::ExitCode;

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") => cmd_validate(args.get(2)),
        Some("run") => cmd_run(args.get(2)),
        Some("nodes") => cmd_nodes(),
        _ => {
            usage();
            Err("perintah tak dikenal".to_string())
        }
    }
}

fn usage() {
    eprintln!("pakai: n8n-cli validate <workflow.json> | run <workflow.json> | nodes");
}

fn need_path(arg: Option<&String>) -> Result<&str, String> {
    arg.map(String::as_str)
        .ok_or_else(|| "kurang argumen: path workflow.json".to_string())
}

fn load_workflow(path: &str) -> Result<Workflow, String> {
    let raw =
        std::fs::read_to_string(path).map_err(|e| format!("baca '{path}' gagal: {e}"))?;
    Workflow::from_json(&raw).map_err(|e| format!("parse '{path}' gagal: {e}"))
}

fn full_registry() -> Registry {
    let mut registry = Registry::default();
    register_all(&mut registry);
    registry
}

fn cmd_validate(arg: Option<&String>) -> Result<(), String> {
    let wf = load_workflow(need_path(arg)?)?;
    println!(
        "OK: '{}' ({} node, {} edge)",
        wf.name,
        wf.nodes.len(),
        wf.edge_count()
    );
    for n in &wf.nodes {
        let flag = if n.disabled { " [disabled]" } else { "" };
        println!("  - '{}' [{}]{flag}", n.name, n.node_type);
    }
    Ok(())
}

fn cmd_run(arg: Option<&String>) -> Result<(), String> {
    let wf = load_workflow(need_path(arg)?)?;
    let report = Engine::run(&wf, &full_registry()).map_err(|e| e.to_string())?;
    println!("urutan: {}", report.order.join(" -> "));
    let out = serde_json::to_string_pretty(&report.outputs).map_err(|e| e.to_string())?;
    println!("{out}");
    Ok(())
}

fn cmd_nodes() -> Result<(), String> {
    for t in full_registry().types() {
        println!("{t}");
    }
    Ok(())
}
