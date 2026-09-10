//! n8n-server: REST API + UI single-file embedded.
//! Tanpa build step, tanpa npm: satu binary menyajikan segalanya.

use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use n8n_core::Workflow;
use n8n_engine::{Diagnostic, Engine, Registry, RunReport};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    registry: Arc<Registry>,
}

#[tokio::main]
async fn main() {
    let mut registry = Registry::default();
    n8n_nodes::register_all(&mut registry);
    let state = AppState {
        registry: Arc::new(registry),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/api/nodes", get(api_nodes))
        .route("/api/validate", post(api_validate))
        .route("/api/explain", post(api_explain))
        .route("/api/run", post(api_run))
        .with_state(state);
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind 0.0.0.0:3000");
    println!("n8n-rust server: http://{addr}");
    axum::serve(listener, app).await.expect("serve");
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
    // Engine sinkron (boleh blocking I/O seperti HTTP) -> jalan di thread pool
    // blocking supaya executor async tak terhambat.
    let reg = s.registry.clone();
    match tokio::task::spawn_blocking(move || Engine::run(&wf, &reg)).await {
        Ok(Ok(rep)) => Ok(Json(rep)),
        Ok(Err(e)) => Err((StatusCode::BAD_REQUEST, e.to_string())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("join: {e}"),
        )),
    }
}
