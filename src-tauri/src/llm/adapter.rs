use crate::crypto::decrypt_secret;
use crate::error::{AppError, AppResult};
use crate::llm::ollama::OllamaAdapter;
use crate::llm::openai_compat::OpenAICompatAdapter;
use crate::models::InferenceParams;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub latency_ms: i64,
    pub usage: Option<serde_json::Value>,
}

pub async fn chat_with_config(
    model_type: &str,
    model_name: &str,
    api_base: Option<&str>,
    api_key_enc: Option<&str>,
    system_prompt: Option<&str>,
    user_prompt: &str,
    params: &InferenceParams,
) -> AppResult<LLMResponse> {
    let api_key = match api_key_enc {
        Some(enc) if !enc.is_empty() => Some(decrypt_secret(enc)?),
        _ => None,
    };

    let start = Instant::now();
    let content = match model_type {
        "openai" | "deepseek" | "qwen" | "doubao" | "claude" | "wenxin" | "custom" => {
            let adapter =
                OpenAICompatAdapter::new(model_type, model_name, api_base, api_key.as_deref())?;
            adapter.chat(system_prompt, user_prompt, params).await?
        }
        "ollama" => {
            let adapter = OllamaAdapter::new(model_name, api_base)?;
            adapter.chat(system_prompt, user_prompt, params).await?
        }
        other => {
            return Err(AppError::msg(format!("unsupported model type: {other}")));
        }
    };

    Ok(LLMResponse {
        content,
        latency_ms: start.elapsed().as_millis() as i64,
        usage: None,
    })
}
