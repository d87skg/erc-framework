use axum::{
    Router,
    routing::post,
    extract::{State, Json, ConnectInfo},
    http::StatusCode,
};
use serde_json::Value;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use axum::http::HeaderMap;

mod proxy;
mod recorder;
mod types;

use proxy::forward_to_upstream;
use recorder::Recorder;
use types::GatewayConfig;

#[derive(Clone)]
struct AppState {
    config: GatewayConfig,
    recorder: Arc<Recorder>,
    client: reqwest::Client,
    rate_db: Arc<Mutex<rusqlite::Connection>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = GatewayConfig::from_env();

    // 初始化限流数据库
    let rate_db = rusqlite::Connection::open("erc_rate_limits.db")
        .expect("无法打开限流数据库");
    rate_db.execute_batch(
        "CREATE TABLE IF NOT EXISTS rate_limits (
            ip TEXT PRIMARY KEY,
            remaining INTEGER NOT NULL,
            window_end INTEGER NOT NULL
        );"
    ).expect("无法创建限流表");
    let rate_db = Arc::new(Mutex::new(rate_db));

    let recorder = Arc::new(Recorder::new(
        "erc_buffer.db",
        &config.agent_id,
    ));

    let state = AppState {
        config: config.clone(),
        recorder,
        client: reqwest::Client::new(),
        rate_db,
    };

    let app = Router::new()
        .route("/v1/messages", post(proxy_handler))
        .route("/health", axum::routing::get(health_handler))
        .with_state(state);

    tracing::info!("🚀 ERC Gateway listening on {}", config.listen_addr);
    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

async fn health_handler() -> &'static str {
    "ERC Gateway: OK"
}

async fn proxy_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let ip = addr.ip().to_string();
    let max_calls_per_hour: u32 = 100;
    let window_seconds: i64 = 3600;
    let now = chrono::Utc::now().timestamp();

    // 从 SQLite 读取或初始化该 IP 的限流状态
    let db = state.rate_db.lock().await;
    let row = db.query_row(
        "SELECT remaining, window_end FROM rate_limits WHERE ip = ?1",
        [&ip],
        |row| Ok((row.get::<_, u32>(0)?, row.get::<_, i64>(1)?)),
    );

    let remaining: u32;

    match row {
        Ok((rem, end)) => {
            if now > end {
                db.execute(
                    "UPDATE rate_limits SET remaining = ?1, window_end = ?2 WHERE ip = ?3",
                    rusqlite::params![max_calls_per_hour - 1, now + window_seconds, ip],
                ).ok();
                remaining = max_calls_per_hour;
            } else {
                remaining = rem;
            }
        }
        Err(_) => {
            db.execute(
                "INSERT INTO rate_limits (ip, remaining, window_end) VALUES (?1, ?2, ?3)",
                rusqlite::params![ip, max_calls_per_hour - 1, now + window_seconds],
            ).ok();
            remaining = max_calls_per_hour;
        }
    }

    if remaining == 0 {
        drop(db);
        tracing::warn!("Rate limit exceeded for IP: {}", ip);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    db.execute(
        "UPDATE rate_limits SET remaining = remaining - 1 WHERE ip = ?1",
        [&ip],
    ).ok();
    drop(db);

    let api_key = std::env::var("UPSTREAM_API_KEY")
        .unwrap_or_else(|_| std::env::var("ANTHROPIC_API_KEY").unwrap_or_default());

    if api_key.is_empty() {
        tracing::error!("No API Key configured. Set UPSTREAM_API_KEY or ANTHROPIC_API_KEY");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let response = forward_to_upstream(
        &state.client,
        &state.config.upstream_url,
        &body,
        &api_key,
    )
    .await
    .map_err(|e| {
        tracing::error!("Proxy error: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    let receipt = state.recorder.record_execution(&body, &response);
    tracing::info!("✅ Receipt generated: {}", receipt.receipt_id);

    Ok(Json(response))
}