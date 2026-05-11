#[derive(Clone)]
#[allow(dead_code)]
pub struct GatewayConfig {
    pub listen_addr: String,
    pub upstream_url: String,
    pub agent_id: String,
    pub public_key: String,
    pub allowed_upstream_hosts: Vec<String>,
}

impl GatewayConfig {
    pub fn from_env() -> Self {
        let allowed_hosts = std::env::var("ALLOWED_UPSTREAM_HOSTS")
            .unwrap_or_else(|_| "api.deepseek.com,api.anthropic.com".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            listen_addr: std::env::var("GATEWAY_LISTEN")
                .unwrap_or("0.0.0.0:8080".into()),
            upstream_url: std::env::var("UPSTREAM_URL")
                .unwrap_or("https://api.deepseek.com/anthropic/v1/messages".into()),
            agent_id: std::env::var("ERC_AGENT_ID")
                .unwrap_or("gateway-001".into()),
            public_key: std::env::var("ERC_PUBLIC_KEY")
                .unwrap_or("".into()),
            allowed_upstream_hosts: allowed_hosts,
        }
    }

    pub fn validate_upstream_url(&self, url: &str) -> Result<(), String> {
        let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
        let host = parsed.host_str().ok_or("No host in URL")?;
        
        // 拒绝IP地址
        if host.parse::<std::net::IpAddr>().is_ok() {
            return Err("IP addresses not allowed for upstream URL".into());
        }
        
        // 验证host在白名单内
        let is_allowed = self.allowed_upstream_hosts.iter()
            .any(|allowed| host == allowed || host.ends_with(&format!(".{}", allowed)));
        
        if !is_allowed {
            return Err(format!(
                "Host '{}' not in allowed list: {:?}", 
                host, 
                self.allowed_upstream_hosts
            ));
        }
        Ok(())
    }
}
