use crate::error::{AppError, AppResult};
use crate::models::InferenceParams;
use serde_json::json;

pub struct OllamaAdapter {
    pub model_name: String,
    pub api_base: String,
}

impl OllamaAdapter {
    pub fn new(model_name: &str, api_base: Option<&str>) -> AppResult<Self> {
        let base = api_base
            .filter(|s| !s.is_empty())
            .unwrap_or("http://127.0.0.1:11434")
            .trim_end_matches('/')
            .to_string();
        Ok(Self {
            model_name: model_name.to_string(),
            api_base: base,
        })
    }

    pub async fn chat(
        &self,
        system_prompt: Option<&str>,
        user_prompt: &str,
        params: &InferenceParams,
    ) -> AppResult<String> {
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
            "stream": false,
            "options": {
                "temperature": params.temperature,
                "num_predict": params.max_tokens,
                "top_p": params.top_p,
            }
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/chat", self.api_base))
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(AppError::msg(format!(
                "Ollama error ({status}): {text}"
            )));
        }
        let value: serde_json::Value = serde_json::from_str(&text)?;
        value
            .pointer("/message/content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::msg("missing content in Ollama response"))
    }

    pub async fn list_models(api_base: &str) -> AppResult<Vec<String>> {
        let base = api_base.trim_end_matches('/');
        let client = reqwest::Client::new();
        let resp = client.get(format!("{base}/api/tags")).send().await?;
        if !resp.status().is_success() {
            return Err(AppError::msg("failed to list Ollama models"));
        }
        let value: serde_json::Value = resp.json().await?;
        let models = value
            .get("models")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(models)
    }
}
