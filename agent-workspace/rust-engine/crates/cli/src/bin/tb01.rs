// tb01: TB-01 tracer binary (Seam 1). Reads a workflow JSON file, prints the
// final items as JSON on stdout. Errors go to stderr with non-zero exit;
// stdout is protocol-pure JSON.
#[tokio::main]
async fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: tb01 <workflow.json>");
        std::process::exit(2);
    });
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("tb01: read {path}: {e}");
        std::process::exit(1);
    });
    match executor::run_workflow_json(&text).await {
        Ok(items) => println!("{}", serde_json::to_string(&items).unwrap()),
        Err(e) => {
            eprintln!("tb01: {e}");
            std::process::exit(1);
        }
    }
}
