use reqwest::Client;
use serde_json::Value;

/// 透明转发请求到上游 API，增强了解析逻辑和错误处理
pub async fn forward_to_upstream(
    client: &Client,
    upstream_url: &str,
    body: &Value,
    api_key: &str,
) -> Result<Value, String> {
    let response = client
        .post(upstream_url)
        .header("x-api-key", api_key)
        .header("content-type", "application/json")
        .json(body)
        .send()
        .await
        .map_err(|e| format!("上游请求失败: {}", e))?;

    let status = response.status();
    
    // 先获取原始响应文本用于调试
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取上游响应文本失败: {}", e))?;

    // 打印出上游的状态码和原始文本，方便你定位问题
    tracing::debug!(
        "上游 status={}, text_len={}", 
        status, 
        response_text.len()
    );
    if !status.is_success() {
        // 如果不是 2xx，则将状态码和原始文本作为错误返回
        return Err(format!(
            "上游返回错误 {}: {}", 
            status, 
            response_text
        ));
    }

    // 检查空响应体
    if response_text.trim().is_empty() {
        return Err("上游返回空响应体".to_string());
    }

    // 尝试解析为JSON
    let response_body: Value = serde_json::from_str(&response_text)
        .map_err(|e| {
            let preview = if response_text.len() > 200 {
                format!("{}...", &response_text[..200])
            } else {
                response_text.clone()
            };
            format!(
                "解析上游响应失败: {}。原始响应预览: {}", 
                e, preview
            )
        })?;

    Ok(response_body)
}