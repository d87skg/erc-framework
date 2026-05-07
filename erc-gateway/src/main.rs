use axum::{
    Router,
    routing::post,
    extract::State,
    Json,
    http::StatusCode,
};
use serde_json::Value;
use std::sync::Arc;

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
}

#[tokio::main]
async fn main() {
    let config = GatewayConfig::from_env();

    let recorder = Arc::new(Recorder::new(
        "erc_buffer.db",
        &config.agent_id,
    ));

    let state = AppState {
        config: config.clone(),
        recorder,
        client: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/v1/messages", post(proxy_handler))
        .route("/health", axum::routing::get(health_handler))
        .with_state(state);

    println!("🚀 ERC Gateway listening on {}", config.listen_addr);
    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> &'static str {
    "ERC Gateway: OK"
}

async fn proxy_handler(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // 从请求头中获取 API key
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .unwrap_or_default();

    // 转发到上游
    let response = forward_to_upstream(
        &state.client,
        &state.config.upstream_url,
        &body,
        &api_key,
    )
    .await
    .map_err(|e| {
        eprintln!("代理错误: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    // 记录执行
    let receipt = state.recorder.record_execution(&body, &response);
    println!("✅ 回执已生成: {}", receipt.receipt_id);

    Ok(Json(response))
}