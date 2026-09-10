//! n8n-server: REST API + UI single-file embedded + webhook hooks.
//!
//! State in-memory (personal, sekali jalan): hooks terdaftar + ring 50
//! ringkasan run terakhir. Restart = state hilang (terdokumentasi).

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Html,
    routing::{delete, get, post},
    Json, Router,
};
use n8n_core::Workflow;
use n8n_engine::{Diagnostic, Engine, Registry, RunReport};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    registry: Arc<Registry>,
    hooks: Arc<RwLock<HashMap<String, Workflow>>>,
    runs: Arc<Mutex<VecDeque<RunSummary>>>,
    next_id: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Serialize)]
struct RunSummary {
    id: u64,
    at_epoch: u64,
    workflow: String,
    ok: bool,
    order: Vec<String>,
    total_ms: u128,
}

#[derive(Debug, Deserialize)]
struct HookReg {
    path: String,
    workflow: Workflow,
}

#[tokio::main]
async fn main() {
    let mut registry = Registry::default();
    n8n_nodes::register_all(&mut registry);
    let state = AppState {
        registry: Arc::new(registry),
        hooks: Arc::new(RwLock::new(HashMap::new())),
        runs: Arc::new(Mutex::new(VecDeque::new())),
        next_id: Arc::new(AtomicU64::new(1)),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/api/nodes", get(api_nodes))
        .route("/api/validate", post(api_validate))
        .route("/api/explain", post(api_explain))
        .route("/api/run", post(api_run))
        .route("/api/runs", get(api_runs))
        .route("/api/hooks", post(api_hook_register).get(api_hook_list))
        .route("/api/hooks/:path", delete(api_hook_delete))
        .route("/hook/:path", post(api_hook_fire))
        .with_state(state);
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind 0.0.0.0:3000");
    println!("n8n-rust server: http://{addr}");
    axum::serve(listener, app).await.expect("serve");
}

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn record(state: &AppState, workflow: &str, ok: bool, order: Vec<String>, total_ms: u128) {
    let id = state.next_id.fetch_add(1, Ordering::SeqCst);
    let mut runs = state.runs.lock().unwrap_or_else(|e| e.into_inner());
    runs.push_back(RunSummary {
        id,
        at_epoch: now_epoch(),
        workflow: workflow.to_string(),
        ok,
        order,
        total_ms,
    });
    while runs.len() > 50 {
        runs.pop_front();
    }
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../ui/app.html"))
}

async fn api_nodes(State(s): State<AppState>) -> Json<Vec<String>> {
    Json(s.registry.types())
}

async fn api_validate(
    State(s): State<AppState>,
    Json(wf): Json<Workflow>,
) -> Json<Vec<Diagnostic>> {
    Json(Engine::lint(&wf, &s.registry))
}

async fn api_explain(
    Json(wf): Json<Workflow>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    Engine::explain(&wf)
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn api_run(
    State(s): State<AppState>,
    Json(wf): Json<Workflow>,
) -> Result<Json<RunReport>, (StatusCode, String)> {
    // Engine sinkron (boleh blocking I/O seperti HTTP) -> thread pool
    // blocking supaya executor async tak terhambat.
    let reg = s.registry.clone();
    let name = wf.name.clone();
    let res = tokio::task::spawn_blocking(move || Engine::run(&wf, &reg)).await;
    match res {
        Ok(Ok(rep)) => {
            let total = rep.durations_ms.values().sum();
            record(&s, &name, true, rep.order.clone(), total);
            Ok(Json(rep))
        }
        Ok(Err(e)) => {
            record(&s, &name, false, Vec::new(), 0);
            Err((StatusCode::BAD_REQUEST, e.to_string()))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("join: {e}"),
        )),
    }
}

async fn api_runs(State(s): State<AppState>) -> Json<Vec<RunSummary>> {
    let runs = s.runs.lock().unwrap_or_else(|e| e.into_inner());
    Json(runs.iter().cloned().collect())
}

async fn api_hook_register(
    State(s): State<AppState>,
    Json(reg): Json<HookReg>,
) -> Result<(StatusCode, Json<String>), (StatusCode, String)> {
    let path = reg.path.trim().trim_matches('/').to_string();
    if path.is_empty() || path.contains('/') {
        return Err((
            StatusCode::BAD_REQUEST,
            "hook path harus satu segmen (mis. 'demo')".to_string(),
        ));
    }
    s.hooks.write().await.insert(path.clone(), reg.workflow);
    Ok((StatusCode::CREATED, Json(path)))
}

async fn api_hook_list(State(s): State<AppState>) -> Json<Vec<String>> {
    let hooks = s.hooks.read().await;
    let mut v: Vec<String> = hooks.keys().cloned().collect();
    v.sort();
    Json(v)
}

async fn api_hook_delete(
    State(s): State<AppState>,
    Path(path): Path<String>,
) -> Json<bool> {
    Json(s.hooks.write().await.remove(&path).is_some())
}

async fn api_hook_fire(
    State(s): State<AppState>,
    Path(path): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<RunReport>, (StatusCode, String)> {
    let wf = s.hooks.read().await.get(&path).cloned();
    let wf =
        wf.ok_or_else(|| (StatusCode::NOT_FOUND, format!("hook tak dikenal: {path}")))?;
    let text = String::from_utf8_lossy(&body);
    let data: serde_json::Value =
        serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text.to_string()));
    let mut hm = serde_json::Map::new();
    for (k, v) in headers.iter() {
        hm.insert(
            k.to_string(),
            serde_json::Value::String(v.to_str().unwrap_or("").to_string()),
        );
    }
    let payload = serde_json::json!({
        "method": "POST",
        "path": path,
        "query": query,
        "headers": hm,
        "body": data,
    });
    let reg = s.registry.clone();
    let name = wf.name.clone();
    let res =
        tokio::task::spawn_blocking(move || Engine::run_with(&wf, &reg, Some(payload))).await;
    match res {
        Ok(Ok(rep)) => {
            let total = rep.durations_ms.values().sum();
            record(&s, &name, true, rep.order.clone(), total);
            Ok(Json(rep))
        }
        Ok(Err(e)) => {
            record(&s, &name, false, Vec::new(), 0);
            Err((StatusCode::BAD_REQUEST, e.to_string()))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("join: {e}"),
        )),
    }
}
