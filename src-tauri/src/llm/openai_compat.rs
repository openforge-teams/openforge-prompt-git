use crate::error::{AppError, AppResult};
use crate::models::InferenceParams;
use serde_json::json;

pub struct OpenAICompatAdapter {
    pub model_type: String,
    pub model_name: String,
    pub api_base: String,
    pub api_key: Option<String>,
}

impl OpenAICompatAdapter {
    pub fn new(
        model_type: &str,
        model_name: &str,
        api_base: Option<&str>,
        api_key: Option<&str>,
    ) -> AppResult<Self> {
        let default_base = match model_type {
            "openai" => "https://api.openai.com/v1",
            "deepseek" => "https://api.deepseek.com/v1",
            "qwen" => "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "doubao" => "https://ark.cn-beijing.volces.com/api/v3",
            "claude" => "https://api.anthropic.com/v1",
            "wenxin" => "https://qianfan.baidubce.com/v2",
            _ => "https://api.openai.com/v1",
        };
        let base = api_base
            .filter(|s| !s.is_empty())
            .unwrap_or(default_base)
            .trim_end_matches('/')
            .to_string();

        Ok(Self {
            model_type: model_type.to_string(),
            model_name: model_name.to_string(),
            api_base: base,
            api_key: api_key.map(|s| s.to_string()),
        })
    }

    pub async fn chat(
        &self,
        system_prompt: Option<&str>,
        user_prompt: &str,
        params: &InferenceParams,
    ) -> AppResult<String> {
        if self.model_type == "claude" {
            return self.chat_claude(system_prompt, user_prompt, params).await;
        }

        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            if !sys.is_empty() {
                messages.push(json!({"role": "system", "content": sys}));
            }
        }
        messages.push(json!({"role": "user", "content": user_prompt}));

        let body = json!({
            "model": self.model_name,
            "messages": messages,
            "temperature": params.temperature,
            "max_tokens": params.max_tokens,
            "top_p": params.top_p,
            "frequency_penalty": params.frequency_penalty,
        });

        let client = reqwest::Client::new();
        let mut req = client
            .post(format!("{}/chat/completions", self.api_base))
            .header("Content-Type", "application/json")
            .json(&body);

        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(AppError::msg(format!(
                "API error ({status}): {}",
                truncate(&text, 500)
            )));
        }

        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| AppError::msg(format!("invalid API response: {e}")))?;
        value
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::msg("missing content in API response"))
    }

    async fn chat_claude(
        &self,
        system_prompt: Option<&str>,
        user_prompt: &str,
        params: &InferenceParams,
    ) -> AppResult<String> {
        let mut body = json!({
            "model": self.model_name,
            "max_tokens": params.max_tokens,
            "temperature": params.temperature,
            "top_p": params.top_p,
            "messages": [{"role": "user", "content": user_prompt}],
        });
        if let Some(sys) = system_prompt {
            if !sys.is_empty() {
                body["system"] = json!(sys);
            }
        }

        let client = reqwest::Client::new();
        let mut req = client
            .post(format!("{}/messages", self.api_base))
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&body);

        if let Some(key) = &self.api_key {
            req = req.header("x-api-key", key);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(AppError::msg(format!(
                "Claude API error ({status}): {}",
                truncate(&text, 500)
            )));
        }
        let value: serde_json::Value = serde_json::from_str(&text)?;
        value
            .pointer("/content/0/text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::msg("missing content in Claude response"))
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}
