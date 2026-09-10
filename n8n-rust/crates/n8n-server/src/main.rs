//! n8n-server: REST API + UI single-file embedded + webhook hooks.
//!
//! v0.5.0 (fidelity n8n): route hook terima GET/POST/PUT/PATCH/DELETE;
//! node webhook mengatur `httpMethod` (default GET, di-enforce → 404 bila
//! salah ala n8n), `responseMode` (onReceived = laporan penuh; lastNode =
//! tunggu selesai lalu bentuk body), `responseData`
//! (allEntries/firstEntryJson/noResponseBody), `responseCode`.
//! Beda disengaja: onReceived mengembalikan RunReport (n8n: ack
//! "Workflow was started") — run lokal sinkron, laporan lebih berguna.
//! State in-memory: restart = hooks + riwayat hilang (terdokumentasi).
//!
//! v0.6.0: `responseMode: "responseNode"` — webhook node menunjuk node
//! respondToWebhook (`respondWith` json/text/redirect; stream ditolak).
//! Template `={{ }}` di responseBody dirender via n8n-core expr.

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use n8n_core::Workflow;
use n8n_engine::{BranchOutputs, Diagnostic, Engine, Registry, RunReport};
use n8n_nodes::{find_respond, find_webhook};
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
        .route(
            "/hook/:path",
            get(api_hook_fire)
                .post(api_hook_fire)
                .put(api_hook_fire)
                .patch(api_hook_fire)
                .delete(api_hook_fire),
        )
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

fn hook_param<'a>(
    wf: &'a Workflow,
    key: &str,
) -> Option<&'a serde_json::Value> {
    find_webhook(wf).and_then(|n| n.parameters.get(key))
}

/// Render `responseBody` node respond: string `={{ }}` dirender dengan
/// konteks item pertama output node itu; non-string dipakai apa adanya.
/// Absen → semua items sebagai array.
fn respond_json(
    body: Option<&serde_json::Value>,
    items: &[serde_json::Value],
    outputs: &std::collections::HashMap<String, BranchOutputs>,
) -> serde_json::Value {
    match body {
        None => serde_json::Value::Array(items.to_vec()),
        Some(b) => {
            if let Some(t) = b.as_str().filter(|s| s.starts_with('=')) {
                let null = serde_json::Value::Null;
                let item = items.first().unwrap_or(&null);
                let ectx = n8n_core::expr::ExprContext { item, outputs };
                n8n_core::expr::render(t, &ectx)
            } else {
                b.clone()
            }
        }
    }
}

fn respond_text(
    body: Option<&serde_json::Value>,
    items: &[serde_json::Value],
    outputs: &std::collections::HashMap<String, BranchOutputs>,
) -> String {
    let v = match body.and_then(serde_json::Value::as_str) {
        Some(t) if t.starts_with('=') => {
            let null = serde_json::Value::Null;
            let item = items.first().unwrap_or(&null);
            let ectx = n8n_core::expr::ExprContext { item, outputs };
            n8n_core::expr::render(t, &ectx)
        }
        Some(t) => serde_json::Value::String(t.to_string()),
        None => serde_json::Value::Array(items.to_vec()),
    };
    match v {
        serde_json::Value::String(s) => s,
        other => serde_json::to_string(&other).unwrap_or_default(),
    }
}

async fn api_hook_fire(
    State(s): State<AppState>,
    Path(path): Path<String>,
    method: Method,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, (StatusCode, String)> {
    let wf = s.hooks.read().await.get(&path).cloned();
    let wf =
        wf.ok_or_else(|| (StatusCode::NOT_FOUND, format!("hook tak dikenal: {path}")))?;
    // n8n mendaftarkan webhook per (method, path): method salah → 404.
    // Tanpa node webhook di workflow → semua method diterima (warisan 0.4).
    if find_webhook(&wf).is_some() {
        let want = hook_param(&wf, "httpMethod")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("GET")
            .to_uppercase();
        if method.as_str() != want {
            return Err((
                StatusCode::NOT_FOUND,
                format!("hook '{path}' tidak terdaftar untuk {method} (mau {want})"),
            ));
        }
    }
    let data: serde_json::Value = if body.is_empty() {
        serde_json::Value::Null
    } else {
        let text = String::from_utf8_lossy(&body);
        serde_json::from_str(&text)
            .unwrap_or(serde_json::Value::String(text.to_string()))
    };
    let mut hm = serde_json::Map::new();
    for (k, v) in headers.iter() {
        hm.insert(
            k.to_string(),
            serde_json::Value::String(v.to_str().unwrap_or("").to_string()),
        );
    }
    let payload = serde_json::json!({
        "method": method.as_str(),
        "path": path,
        "query": query,
        "headers": hm,
        "body": data,
    });
    let reg = s.registry.clone();
    let name = wf.name.clone();
    // `wf` dipakai lagi setelah run (responseMode) → clone untuk thread.
    let wf_run = wf.clone();
    let res = tokio::task::spawn_blocking(move || {
        Engine::run_with(&wf_run, &reg, Some(payload))
    })
    .await;
    let rep = match res {
        Ok(Ok(rep)) => rep,
        Ok(Err(e)) => {
            record(&s, &name, false, Vec::new(), 0);
            return Err((StatusCode::BAD_REQUEST, e.to_string()));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("join: {e}"),
            ))
        }
    };
    let total = rep.durations_ms.values().sum();
    record(&s, &name, true, rep.order.clone(), total);

    let mode = hook_param(&wf, "responseMode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("onReceived");
    match mode {
        "onReceived" => {
            let v = serde_json::to_value(&rep)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            Ok((StatusCode::OK, Json(v)).into_response())
        }
        "lastNode" => {
            let code = hook_param(&wf, "responseCode")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(200);
            let status =
                StatusCode::from_u16(code as u16).unwrap_or(StatusCode::OK);
            let items: Vec<serde_json::Value> = rep
                .order
                .last()
                .and_then(|last| rep.outputs.get(last))
                .map(|branches| branches.iter().flatten().cloned().collect())
                .unwrap_or_default();
            let data_mode = hook_param(&wf, "responseData")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("firstEntryJson");
            match data_mode {
                "firstEntryJson" => {
                    let first = items.into_iter().next().unwrap_or(serde_json::Value::Null);
                    Ok((status, Json(first)).into_response())
                }
                "allEntries" => Ok((status, Json(items)).into_response()),
                "noResponseBody" => Ok((status, "").into_response()),
                "firstEntryBinary" => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "responseData 'firstEntryBinary' belum didukung (tanpa binary)".to_string(),
                )),
                other => Err((
                    StatusCode::BAD_REQUEST,
                    format!("responseData tak dikenal '{other}'"),
                )),
            }
        }
        "responseNode" => {
            let rnode = match find_respond(&wf) {
                Some(n) => n,
                None => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "responseMode 'responseNode' tapi workflow tanpa node respondToWebhook"
                            .to_string(),
                    ))
                }
            };
            let p = &rnode.parameters;
            let items: Vec<serde_json::Value> = rep
                .outputs
                .get(&rnode.name)
                .map(|branches| branches.iter().flatten().cloned().collect())
                .unwrap_or_default();
            let code = p
                .get("responseCode")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(200);
            let status = StatusCode::from_u16(code as u16).unwrap_or(StatusCode::OK);
            let mut headers = HeaderMap::new();
            if let Some(vals) = p
                .get("responseHeaders")
                .and_then(|h| h.get("values"))
                .and_then(serde_json::Value::as_array)
            {
                for hv in vals {
                    let name = hv.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let value = hv.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    if name.is_empty() {
                        continue;
                    }
                    let n: HeaderName = name.parse().map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            format!("response header tak valid: '{name}'"),
                        )
                    })?;
                    let v: HeaderValue = value.parse().map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            format!("response header tak valid: '{name}'"),
                        )
                    })?;
                    headers.insert(n, v);
                }
            }
            let respond_with = p
                .get("respondWith")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("json");
            match respond_with {
                "json" => {
                    let v = respond_json(p.get("responseBody"), &items, &rep.outputs);
                    let mut resp = (status, Json(v)).into_response();
                    resp.headers_mut().extend(headers);
                    Ok(resp)
                }
                "text" => {
                    let t = respond_text(p.get("responseBody"), &items, &rep.outputs);
                    let mut resp = (status, t).into_response();
                    resp.headers_mut().extend(headers);
                    Ok(resp)
                }
                "redirect" => {
                    let url = p
                        .get("redirectUrl")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("");
                    if url.is_empty() {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            "respondWith 'redirect' tapi redirectUrl kosong".to_string(),
                        ));
                    }
                    let loc: HeaderValue = url.parse().map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            format!("redirectUrl tak valid: '{url}'"),
                        )
                    })?;
                    headers.insert(header::LOCATION, loc);
                    let mut resp = (status, "").into_response();
                    resp.headers_mut().extend(headers);
                    Ok(resp)
                }
                "stream" => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "respondWith 'stream' belum didukung (tanpa binary)".to_string(),
                )),
                other => Err((
                    StatusCode::BAD_REQUEST,
                    format!("respondWith tak dikenal '{other}'"),
                )),
            }
        }
        other => Err((
            StatusCode::BAD_REQUEST,
            format!("responseMode tak dikenal '{other}'"),
        )),
    }
}
