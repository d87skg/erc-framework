use axum::{
    Router,
    routing::post,
    extract::{State, Json, ConnectInfo, DefaultBodyLimit},
    http::StatusCode,
};
use serde_json::Value;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use axum::http::HeaderMap;

/// 设置数据库文件权限为600（仅所有者可读写）
#[cfg(unix)]
fn set_db_permissions(path: &str) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_db_permissions(_path: &str) -> std::io::Result<()> {
    // Windows系统暂不设置权限
    Ok(())
}

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
    
    // 设置数据库文件权限为600
    if let Err(e) = set_db_permissions("erc_rate_limits.db") {
        tracing::warn!("无法设置限流数据库文件权限: {}", e);
    }
    
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
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))  // 10MB限制
        .with_state(state);

    tracing::info!("🚀 ERC Gateway listening on {}", config.listen_addr);
    let listener = match tokio::net::TcpListener::bind(&config.listen_addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("无法绑定监听地址 {}: {}", config.listen_addr, e);
            std::process::exit(1);
        }
    };
    if let Err(e) = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await {
        tracing::error!("服务器启动失败: {}", e);
    }
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

    // 使用原子操作进行限流检查和扣减，避免竞态条件
    let db = state.rate_db.lock().await;
    let remaining: u32 = db.query_row(
        "INSERT INTO rate_limits (ip, remaining, window_end)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(ip) DO UPDATE SET
             remaining = CASE
                 WHEN window_end < ?4 THEN ?2 - 1
                 WHEN remaining > 0 THEN remaining - 1
                 ELSE 0
             END,
             window_end = CASE
                 WHEN window_end < ?4 THEN ?3
                 ELSE window_end
             END
         RETURNING remaining",
        rusqlite::params![ip, max_calls_per_hour, now + window_seconds, now],
        |row| row.get(0),
    ).map_err(|e| {
        tracing::error!("限流数据库操作失败: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if remaining == 0 {
        drop(db);
        tracing::warn!("Rate limit exceeded for IP: {}", ip);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    drop(db);

    let api_key = std::env::var("UPSTREAM_API_KEY")
        .unwrap_or_else(|_| std::env::var("ANTHROPIC_API_KEY").unwrap_or_default());

    if api_key.is_empty() {
        tracing::error!("No API Key configured. Set UPSTREAM_API_KEY or ANTHROPIC_API_KEY");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // 验证上游URL是否在白名单内
    state.config.validate_upstream_url(&state.config.upstream_url).map_err(|e| {
        tracing::error!("上游URL验证失败: {}", e);
        StatusCode::FORBIDDEN
    })?;

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