use axum::{
    Router,
    routing::get,
    extract::{Query, State},
    Json,
};
use erc_store::Store;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Clone)]
struct AppState {
    store: Arc<Store>,
}

#[tokio::main]
async fn main() {
    let db_path = "D:/erc-project/erc_buffer.db";
    eprintln!("Opening database: {}", db_path);
    let store = match Store::open(db_path) {
        Ok(s) => { eprintln!("Database opened"); Arc::new(s) }
        Err(e) => { eprintln!("Failed: {}", e); return; }
    };
    let state = AppState { store };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/receipts", get(get_receipt))
        .route("/api/v1/executions/events", get(get_events_by_execution))
        .route("/api/v1/traces/causality", get(get_events_by_trace))
        .with_state(state);

    println!("Listening on 127.0.0.1:8082");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8082").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str { "OK" }

async fn get_receipt(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, String> {
    let id = params.get("id").ok_or("missing id")?;
    let payload = state.store.get_receipt(id).ok_or("not found")?;
    let value: serde_json::Value = serde_json::from_str(&payload).map_err(|e| e.to_string())?;
    Ok(Json(value))
}

async fn get_events_by_execution(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<serde_json::Value>>, String> {
    let id = params.get("id").ok_or("missing id")?;
    let payloads = state.store.get_events_by_execution_id(id);
    let events: Vec<serde_json::Value> = payloads.iter().filter_map(|p| serde_json::from_str(p).ok()).collect();
    Ok(Json(events))
}

async fn get_events_by_trace(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<serde_json::Value>>, String> {
    let id = params.get("id").ok_or("missing id")?;
    let payloads = state.store.get_events_by_trace_id(id);
    let events: Vec<serde_json::Value> = payloads.iter().filter_map(|p| serde_json::from_str(p).ok()).collect();
    Ok(Json(events))
}