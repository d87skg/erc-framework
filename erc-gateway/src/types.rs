#[derive(Clone)]
#[allow(dead_code)]
pub struct GatewayConfig {
    pub listen_addr: String,
    pub upstream_url: String,
    pub agent_id: String,
    pub public_key: String,
}

impl GatewayConfig {
    pub fn from_env() -> Self {
        Self {
            listen_addr: std::env::var("GATEWAY_LISTEN")
                .unwrap_or("0.0.0.0:8080".into()),
            upstream_url: std::env::var("UPSTREAM_URL")
                .unwrap_or("https://api.deepseek.com/anthropic/v1/messages".into()),
            agent_id: std::env::var("ERC_AGENT_ID")
                .unwrap_or("gateway-001".into()),
            public_key: std::env::var("ERC_PUBLIC_KEY")
                .unwrap_or("".into()),
        }
    }
}