use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: i64,
    pub project_id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptFile {
    pub id: i64,
    pub project_id: i64,
    pub folder_id: Option<i64>,
    pub name: String,
    pub current_version_hash: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVersion {
    pub id: i64,
    pub version_hash: String,
    pub prompt_file_id: i64,
    pub system_prompt: String,
    pub user_prompt: String,
    pub commit_message: String,
    pub remark: Option<String>,
    pub created_at: String,
    pub tags: Vec<VersionTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionTag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub project_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: String,
    pub old_line: Option<String>,
    pub new_line: Option<String>,
    pub old_no: Option<usize>,
    pub new_no: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub system_diff: Vec<DiffLine>,
    pub user_diff: Vec<DiffLine>,
    pub version_a: PromptVersion,
    pub version_b: PromptVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParams {
    pub temperature: f64,
    pub max_tokens: u32,
    pub top_p: f64,
    pub frequency_penalty: f64,
}

impl Default for InferenceParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 1.0,
            frequency_penalty: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: i64,
    pub model_type: String,
    pub model_name: String,
    pub api_base: Option<String>,
    pub api_key_masked: String,
    pub has_api_key: bool,
    pub default_params: InferenceParams,
    pub is_enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigInput {
    pub id: Option<i64>,
    pub model_type: String,
    pub model_name: String,
    pub api_base: Option<String>,
    pub api_key: Option<String>,
    pub default_params: Option<InferenceParams>,
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareTask {
    pub id: i64,
    pub project_id: i64,
    pub prompt_version_hash: Option<String>,
    pub prompt_content: String,
    pub system_prompt: Option<String>,
    pub models: Vec<i64>,
    pub params: InferenceParams,
    pub status: String,
    pub created_at: String,
    pub results: Vec<CompareResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    pub id: i64,
    pub task_id: i64,
    pub model_config_id: i64,
    pub model_name: String,
    pub output_content: Option<String>,
    pub scores: Option<serde_json::Value>,
    pub total_score: Option<f64>,
    pub evaluation: Option<String>,
    pub latency: Option<i64>,
    pub status: String,
    pub error_msg: Option<String>,
    pub is_best: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreInput {
    pub accuracy: Option<f64>,
    pub instruction: Option<f64>,
    pub format: Option<f64>,
    pub speed: Option<f64>,
    pub evaluation: Option<String>,
    pub is_best: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTask {
    pub id: i64,
    pub project_id: i64,
    pub template: String,
    pub system_prompt: Option<String>,
    pub model_config_id: i64,
    pub params: InferenceParams,
    pub status: String,
    pub concurrency: i64,
    pub created_at: String,
    pub results: Vec<BatchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub id: i64,
    pub task_id: i64,
    pub case_index: i64,
    pub variables: serde_json::Value,
    pub rendered_prompt: String,
    pub output_content: Option<String>,
    pub score: Option<f64>,
    pub latency: Option<i64>,
    pub status: String,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableCase {
    pub variables: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub auto_backup: bool,
    #[serde(default = "default_backup_hours")]
    pub backup_interval_hours: u32,
    #[serde(default = "default_concurrency")]
    pub default_concurrency: u32,
    #[serde(default = "default_ollama")]
    pub ollama_base: String,
    #[serde(default)]
    pub has_app_password: bool,
}

fn default_theme() -> String {
    "system".into()
}
fn default_true() -> bool {
    true
}
fn default_backup_hours() -> u32 {
    24
}
fn default_concurrency() -> u32 {
    3
}
fn default_ollama() -> String {
    "http://127.0.0.1:11434".into()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            auto_backup: true,
            backup_interval_hours: 24,
            default_concurrency: 3,
            ollama_base: "http://127.0.0.1:11434".into(),
            has_app_password: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportCodeRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub model_name: String,
    pub language: String,
    pub params: InferenceParams,
}

#[allow(dead_code)]
pub fn now_str() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

#[allow(dead_code)]
pub fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.3f")
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
