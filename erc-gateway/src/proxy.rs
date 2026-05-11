use reqwest::Client;
use serde_json::Value;
use jsonschema::JSONSchema;

/// 上游响应JSON Schema验证
const UPSTREAM_RESPONSE_SCHEMA: &str = r#"{
    "type": "object",
    "properties": {
        "id": {"type": "string"},
        "type": {"type": "string"},
        "role": {"type": "string"},
        "content": {"type": "array"},
        "model": {"type": "string"},
        "stop_reason": {"type": ["string", "null"]},
        "usage": {"type": "object"}
    },
    "additionalProperties": true
}"#;

/// 验证上游响应的JSON结构
fn validate_response_schema(body: &Value) -> Result<(), String> {
    let schema: Value = serde_json::from_str(UPSTREAM_RESPONSE_SCHEMA)
        .map_err(|e| format!("Schema解析失败: {}", e))?;
    
    let compiled = JSONSchema::compile(&schema)
        .map_err(|e| format!("Schema编译失败: {}", e))?;
    
    compiled.validate(body).map_err(|errors| {
        let error_msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
        format!("响应Schema验证失败: {}", error_msgs.join("; "))
    })
}

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

    // 验证响应Schema
    validate_response_schema(&response_body)?;

    Ok(response_body)
}