use reqwest::Client;
use serde_json::Value;

/// 透明转发请求到上游 API
pub async fn forward_to_upstream(
    client: &Client,
    upstream_url: &str,
    body: &Value,
    api_key: &str,
) -> Result<Value, String> {
    let response = client
        .post(upstream_url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(body)
        .send()
        .await
        .map_err(|e| format!("上游请求失败: {}", e))?;

    let status = response.status();
    let response_body: Value = response
        .json()
        .await
        .map_err(|e| format!("解析上游响应失败: {}", e))?;

    if !status.is_success() {
        return Err(format!("上游返回错误 {}: {}", status, response_body));
    }

    Ok(response_body)
}