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
use tower_http::cors::CorsLayer;
use axum::http::header;

#[derive(Clone)]
struct AppState {
    store: Arc<Store>,
    api_token: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_path = "D:/erc-project/erc_buffer.db";

    // 强制要求设置 API Token，不允许默认值
    let api_token = env::var("ERC_API_TOKEN").unwrap_or_else(|_| {
        eprintln!("FATAL: ERC_API_TOKEN environment variable is not set.");
        eprintln!("For security, the Query API will not start with a default token.");
        eprintln!("Set it with: $env:ERC_API_TOKEN = \"your-strong-password\"");
        std::process::exit(1);
    });

    tracing::info!("Opening database: {}", db_path);

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

    // 严格化 CORS：仅允许指定来源
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::exact(
            env::var("ERC_CORS_ORIGIN")
                .unwrap_or("http://localhost:3000".to_string())
                .parse()
                .unwrap(),
        ))
        .allow_methods([axum::http::Method::GET])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/receipts", get(get_receipt))
        .route("/api/v1/executions/events", get(get_events_by_execution))
        .route("/api/v1/traces/causality", get(get_events_by_trace))
        .layer(cors)
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
    let payloads = state.store.get_events_by_trace_id(id);
    let events: Vec<serde_json::Value> = payloads
        .iter()
        .filter_map(|p| serde_json::from_str(p).ok())
        .collect();
    Ok(Json(events))
}