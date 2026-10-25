use std::time::{Duration, Instant};

use reqwest::Client;
use serde_json::{json, Value};

use crate::settings::LlmSettings;

pub fn endpoint(base_url: &str) -> Result<String, String> {
    let base = base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err("API Base URL 不能为空".into());
    }
    if base.ends_with("/chat/completions") {
        return Ok(base.to_string());
    }
    Ok(format!("{base}/chat/completions"))
}

fn client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| format!("初始化 HTTP 客户端失败：{e}"))
}

fn body(settings: &LlmSettings, system: &str, user: &str, json_mode: bool, max_tokens: u32) -> Value {
    let mut body = json!({
        "model": settings.model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ],
        "temperature": settings.temperature,
        "max_tokens": max_tokens,
        "stream": false
    });
    if json_mode {
        body["response_format"] = json!({ "type": "json_object" });
    }
    body
}

async fn post(settings: &LlmSettings, payload: &Value) -> Result<String, String> {
    let url = endpoint(&settings.base_url)?;
    let response = client()?
        .post(&url)
        .bearer_auth(settings.api_key.trim())
        .json(payload)
        .send()
        .await
        .map_err(|e| format!("请求模型失败：{e}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("读取模型响应失败：{e}"))?;

    if !status.is_success() {
        let snippet: String = text.chars().take(300).collect();
        return Err(format!("模型返回 HTTP {status}：{snippet}"));
    }

    let value: Value = serde_json::from_str(&text)
        .map_err(|e| format!("模型响应不是合法 JSON：{e}｜{}", text.chars().take(200).collect::<String>()))?;

    value["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            let snippet: String = text.chars().take(300).collect();
            format!("模型响应缺少 content：{snippet}")
        })
}

async fn post_retry(settings: &LlmSettings, payload: &Value) -> Result<String, String> {
    let mut last = String::new();
    for attempt in 0..3 {
        match post(settings, payload).await {
            Ok(text) => return Ok(text),
            Err(err) => {
                let retryable = err.contains("请求模型失败")
                    || err.contains("读取模型响应失败")
                    || err.contains("HTTP 429")
                    || err.contains("HTTP 500")
                    || err.contains("HTTP 502")
                    || err.contains("HTTP 503")
                    || err.contains("HTTP 504");
                last = err;
                if !retryable || attempt == 2 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(500 * (1 << attempt))).await;
            }
        }
    }
    Err(last)
}

pub async fn chat(
    settings: &LlmSettings,
    system: &str,
    user: &str,
    json_mode: bool,
    max_tokens: u32,
) -> Result<String, String> {
    if settings.api_key.trim().is_empty() {
        return Err("未配置 API Key".into());
    }
    if settings.model.trim().is_empty() {
        return Err("未配置模型名称".into());
    }

    let payload = body(settings, system, user, json_mode, max_tokens);
    match post_retry(settings, &payload).await {
        Ok(text) => Ok(text),
        Err(err) if json_mode && (err.contains("HTTP 400") || err.contains("HTTP 422")) => {
            // 部分 OpenAI 兼容网关不支持 response_format，退化为纯文本 JSON 输出。
            let mut fallback = payload.clone();
            if let Some(map) = fallback.as_object_mut() {
                map.remove("response_format");
            }
            let retry = post_retry(settings, &fallback).await?;
            Ok(retry)
        }
        Err(err) => Err(err),
    }
}

pub async fn test_connection(settings: &LlmSettings) -> Result<String, String> {
    if settings.is_demo() {
        return Ok("演示模式：不会发起网络请求".into());
    }
    let started = Instant::now();
    let reply = chat(
        settings,
        "你是连通性测试助手，只回复 OK。",
        "ping",
        false,
        8,
    )
    .await?;
    let elapsed = started.elapsed().as_millis();
    Ok(format!(
        "连接成功 · {} ms · {}",
        elapsed,
        reply.trim().chars().take(20).collect::<String>()
    ))
}
