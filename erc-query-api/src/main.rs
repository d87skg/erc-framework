use axum::{
    Router,
    routing::get,
    extract::{Query, State},
    Json,
    http::StatusCode,
};
use erc_store::Store;
use std::sync::Arc;
use std::collections::HashMap;
use std::env;

#[derive(Clone)]
struct AppState {
    store: Arc<Store>,
    api_token: String,
}

#[tokio::main]
async fn main() {
    // 初始化结构化日志（输出到 stderr，带时间戳和级别）
    tracing_subscriber::fmt::init();

    let db_path = "D:/erc-project/erc_buffer.db";
    let api_token = env::var("ERC_API_TOKEN").unwrap_or_else(|_| "erc-demo-token".to_string());

    tracing::info!("Opening database: {}", db_path);
    tracing::info!("API Token: {}...", &api_token[..8.min(api_token.len())]);

    let store = match Store::open(db_path) {
        Ok(s) => {
            tracing::info!("Database opened successfully");
            Arc::new(s)
        }
        Err(e) => {
            tracing::error!("Failed to open database: {}", e);
            return;
        }
    };
    let state = AppState { store, api_token };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/receipts", get(get_receipt))
        .route("/api/v1/executions/events", get(get_events_by_execution))
        .route("/api/v1/traces/causality", get(get_events_by_trace))
        .with_state(state);

    tracing::info!("Listening on 127.0.0.1:8082");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8082").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str { "OK" }

fn check_auth(state: &AppState, token: Option<&String>) -> Result<(), (StatusCode, String)> {
    match token {
        Some(t) if t == &state.api_token => Ok(()),
        _ => {
            tracing::warn!("Unauthorized access attempt");
            Err((StatusCode::UNAUTHORIZED, "invalid token".to_string()))
        }
    }
}

async fn get_receipt(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    check_auth(&state, params.get("token"))?;
    let id = params.get("id").ok_or((StatusCode::BAD_REQUEST, "missing id".into()))?;
    tracing::debug!("Query receipt: {}", id);
    let payload = state.store.get_receipt(id).ok_or((StatusCode::NOT_FOUND, "not found".into()))?;
    let value: serde_json::Value = serde_json::from_str(&payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(value))
}

async fn get_events_by_execution(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    check_auth(&state, params.get("token"))?;
    let id = params.get("id").ok_or((StatusCode::BAD_REQUEST, "missing id".into()))?;
    tracing::debug!("Query events for execution: {}", id);
    let payloads = state.store.get_events_by_execution_id(id);
    let events: Vec<serde_json::Value> = payloads
        .iter()
        .filter_map(|p| serde_json::from_str(p).ok())
        .collect();
    Ok(Json(events))
}

async fn get_events_by_trace(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    check_auth(&state, params.get("token"))?;
    let id = params.get("id").ok_or((StatusCode::BAD_REQUEST, "missing id".into()))?;
    tracing::debug!("Query events for trace: {}", id);
    let payloads = state.store.get_events_by_trace_id(id);
    let events: Vec<serde_json::Value> = payloads
        .iter()
        .filter_map(|p| serde_json::from_str(p).ok())
        .collect();
    Ok(Json(events))
}