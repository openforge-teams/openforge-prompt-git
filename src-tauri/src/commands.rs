use crate::db::DbState;
use crate::error::AppResult;
use crate::llm::OllamaAdapter;
use crate::models::*;
use crate::services::{batch, compare, export, model, project, settings, template, version};
use tauri::State;

#[tauri::command]
pub fn create_project(
    state: State<DbState>,
    name: String,
    description: Option<String>,
) -> AppResult<Project> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::create_project(&conn, &name, description.as_deref())
}

#[tauri::command]
pub fn list_projects(state: State<DbState>) -> AppResult<Vec<Project>> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::list_projects(&conn)
}

#[tauri::command]
pub fn update_project(
    state: State<DbState>,
    id: i64,
    name: String,
    description: Option<String>,
) -> AppResult<Project> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::update_project(&conn, id, &name, description.as_deref())
}

#[tauri::command]
pub fn delete_project(state: State<DbState>, id: i64) -> AppResult<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::delete_project(&conn, id)
}

#[tauri::command]
pub fn create_folder(
    state: State<DbState>,
    project_id: i64,
    parent_id: Option<i64>,
    name: String,
) -> AppResult<Folder> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::create_folder(&conn, project_id, parent_id, &name)
}

#[tauri::command]
pub fn list_folders(state: State<DbState>, project_id: i64) -> AppResult<Vec<Folder>> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::list_folders(&conn, project_id)
}

#[tauri::command]
pub fn delete_folder(state: State<DbState>, id: i64) -> AppResult<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::delete_folder(&conn, id)
}

#[tauri::command]
pub fn create_prompt_file(
    state: State<DbState>,
    project_id: i64,
    folder_id: Option<i64>,
    name: String,
    system_prompt: Option<String>,
    user_prompt: Option<String>,
) -> AppResult<PromptFile> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::create_prompt_file(
        &conn,
        project_id,
        folder_id,
        &name,
        system_prompt.as_deref(),
        user_prompt.as_deref(),
    )
}

#[tauri::command]
pub fn list_prompt_files(state: State<DbState>, project_id: i64) -> AppResult<Vec<PromptFile>> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::list_prompt_files(&conn, project_id)
}

#[tauri::command]
pub fn get_prompt_file(state: State<DbState>, id: i64) -> AppResult<PromptFile> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::get_prompt_file(&conn, id)
}

#[tauri::command]
pub fn rename_prompt_file(state: State<DbState>, id: i64, name: String) -> AppResult<PromptFile> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::rename_prompt_file(&conn, id, &name)
}

#[tauri::command]
pub fn delete_prompt_file(state: State<DbState>, id: i64) -> AppResult<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    project::delete_prompt_file(&conn, id)
}

#[tauri::command]
pub fn commit_prompt_version(
    state: State<DbState>,
    file_id: i64,
    system_prompt: String,
    user_prompt: String,
    commit_message: String,
    remark: Option<String>,
    tag_ids: Option<Vec<i64>>,
) -> AppResult<PromptVersion> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::commit_prompt_version(
        &conn,
        file_id,
        &system_prompt,
        &user_prompt,
        &commit_message,
        remark.as_deref(),
        tag_ids.unwrap_or_default(),
    )
}

#[tauri::command]
pub fn get_version_history(state: State<DbState>, file_id: i64) -> AppResult<Vec<PromptVersion>> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::get_version_history(&conn, file_id)
}

#[tauri::command]
pub fn get_version(state: State<DbState>, version_hash: String) -> AppResult<PromptVersion> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::get_version_by_hash(&conn, &version_hash)
}

#[tauri::command]
pub fn diff_versions(
    state: State<DbState>,
    version_hash1: String,
    version_hash2: String,
) -> AppResult<VersionDiff> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::diff_versions(&conn, &version_hash1, &version_hash2)
}

#[tauri::command]
pub fn rollback_version(
    state: State<DbState>,
    file_id: i64,
    target_version_hash: String,
) -> AppResult<PromptVersion> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::rollback_version(&conn, file_id, &target_version_hash)
}

#[tauri::command]
pub fn update_version_remark(
    state: State<DbState>,
    version_hash: String,
    remark: String,
) -> AppResult<PromptVersion> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::update_version_remark(&conn, &version_hash, &remark)
}

#[tauri::command]
pub fn list_tags(state: State<DbState>, project_id: i64) -> AppResult<Vec<VersionTag>> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::list_tags(&conn, project_id)
}

#[tauri::command]
pub fn create_tag(
    state: State<DbState>,
    project_id: i64,
    name: String,
    color: String,
) -> AppResult<VersionTag> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::create_tag(&conn, project_id, &name, &color)
}

#[tauri::command]
pub fn delete_tag(state: State<DbState>, tag_id: i64) -> AppResult<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::delete_tag(&conn, tag_id)
}

#[tauri::command]
pub fn attach_tag(state: State<DbState>, version_hash: String, tag_id: i64) -> AppResult<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::attach_tag(&conn, &version_hash, tag_id)
}

#[tauri::command]
pub fn detach_tag(state: State<DbState>, version_hash: String, tag_id: i64) -> AppResult<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::detach_tag(&conn, &version_hash, tag_id)
}

#[tauri::command]
pub fn filter_history_by_tag(
    state: State<DbState>,
    file_id: i64,
    tag_id: i64,
) -> AppResult<Vec<PromptVersion>> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    version::filter_history_by_tag(&conn, file_id, tag_id)
}

#[tauri::command]
pub fn list_models(state: State<DbState>) -> AppResult<Vec<ModelConfig>> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    model::list_models(&conn)
}

#[tauri::command]
pub fn save_model_config(state: State<DbState>, config: ModelConfigInput) -> AppResult<ModelConfig> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    model::save_model_config(&conn, config)
}

#[tauri::command]
pub fn delete_model_config(state: State<DbState>, id: i64) -> AppResult<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    model::delete_model(&conn, id)
}

#[tauri::command]
pub async fn list_ollama_models(base_url: Option<String>) -> AppResult<Vec<String>> {
    let base = base_url.unwrap_or_else(|| "http://127.0.0.1:11434".into());
    OllamaAdapter::list_models(&base).await
}

#[tauri::command]
pub async fn run_compare_task(
    state: State<'_, DbState>,
    project_id: i64,
    prompt_content: String,
    system_prompt: Option<String>,
    prompt_version_hash: Option<String>,
    model_ids: Vec<i64>,
    params: Option<InferenceParams>,
) -> AppResult<CompareTask> {
    let task_id = {
        let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        compare::create_compare_task(
            &conn,
            project_id,
            &prompt_content,
            system_prompt.as_deref(),
            prompt_version_hash.as_deref(),
            &model_ids,
            &params.unwrap_or_default(),
        )?
    };
    compare::run_compare_task(&state.conn, task_id).await
}

#[tauri::command]
pub fn get_compare_result(state: State<DbState>, task_id: i64) -> AppResult<CompareTask> {
    compare::get_compare_task(&state.conn, task_id)
}

#[tauri::command]
pub fn list_compare_tasks(state: State<DbState>, project_id: i64) -> AppResult<Vec<CompareTask>> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    compare::list_compare_tasks(&conn, project_id)
}

#[tauri::command]
pub fn score_compare_result(
    state: State<DbState>,
    result_id: i64,
    scores: ScoreInput,
) -> AppResult<CompareResult> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    compare::score_result(&conn, result_id, scores)
}

#[tauri::command]
pub fn extract_template_variables(template: String) -> Vec<String> {
    batch::parse_variables(&template)
}

#[tauri::command]
pub fn generate_variable_cases(
    variable_values: serde_json::Map<String, serde_json::Value>,
) -> Vec<VariableCase> {
    batch::generate_cases(&variable_values)
}

#[tauri::command]
pub fn parse_csv_cases(csv: String) -> AppResult<serde_json::Value> {
    let (headers, cases) = template::parse_csv_cases(&csv)
        .map_err(crate::error::AppError::msg)?;
    Ok(serde_json::json!({ "headers": headers, "cases": cases }))
}

#[tauri::command]
pub async fn run_batch_test(
    state: State<'_, DbState>,
    project_id: i64,
    template: String,
    system_prompt: Option<String>,
    model_id: i64,
    cases: Vec<VariableCase>,
    params: Option<InferenceParams>,
    concurrency: Option<u32>,
) -> AppResult<BatchTask> {
    batch::run_batch_test(
        &state.conn,
        project_id,
        &template,
        system_prompt.as_deref(),
        model_id,
        cases,
        params.unwrap_or_default(),
        concurrency.unwrap_or(3) as usize,
    )
    .await
}

#[tauri::command]
pub fn get_batch_result(state: State<DbState>, task_id: i64) -> AppResult<BatchTask> {
    batch::get_batch_task(&state.conn, task_id)
}

#[tauri::command]
pub fn score_batch_result(state: State<DbState>, result_id: i64, score: f64) -> AppResult<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    batch::score_batch_result(&conn, result_id, score)
}

#[tauri::command]
pub fn save_test_suite(
    state: State<DbState>,
    project_id: i64,
    name: String,
    variables_schema: serde_json::Value,
    cases: serde_json::Value,
) -> AppResult<i64> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    batch::save_test_suite(&conn, project_id, &name, &variables_schema, &cases)
}

#[tauri::command]
pub fn list_test_suites(state: State<DbState>, project_id: i64) -> AppResult<Vec<serde_json::Value>> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    batch::list_test_suites(&conn, project_id)
}

#[tauri::command]
pub fn export_code_snippet(request: ExportCodeRequest) -> AppResult<String> {
    export::export_code_snippet(request)
}

#[tauri::command]
pub fn export_json(
    state: State<DbState>,
    file_id: i64,
    version_hash: String,
    model_name: Option<String>,
    params: Option<InferenceParams>,
) -> AppResult<String> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    export::export_json(
        &conn,
        file_id,
        &version_hash,
        model_name.as_deref(),
        params,
    )
}

#[tauri::command]
pub fn export_yaml(state: State<DbState>, file_id: i64, version_hash: String) -> AppResult<String> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    export::export_yaml(&conn, file_id, &version_hash)
}

#[tauri::command]
pub fn export_markdown_report(
    prompt_name: String,
    system_prompt: String,
    user_prompt: String,
    params: InferenceParams,
    results: Vec<(String, String, Option<f64>, Option<i64>, Option<String>)>,
) -> String {
    export::export_markdown_report(&prompt_name, &system_prompt, &user_prompt, &params, &results)
}

#[tauri::command]
pub fn export_plain_prompt(system_prompt: String, user_prompt: String) -> String {
    export::export_plain_prompt(&system_prompt, &user_prompt)
}

#[tauri::command]
pub fn get_settings(state: State<DbState>) -> AppResult<AppSettings> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    settings::load_settings(&state.data_dir, &conn)
}

#[tauri::command]
pub fn save_settings(state: State<DbState>, settings_input: AppSettings) -> AppResult<()> {
    settings::save_settings(&state.data_dir, &settings_input)
}

#[tauri::command]
pub fn set_app_password(state: State<DbState>, password: String) -> AppResult<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    settings::set_app_password(&conn, &password)
}

#[tauri::command]
pub fn clear_app_password(state: State<DbState>) -> AppResult<()> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    settings::clear_app_password(&conn)
}

#[tauri::command]
pub fn verify_app_password(state: State<DbState>, password: String) -> AppResult<bool> {
    let conn = state.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    settings::verify_app_password(&conn, &password)
}

#[tauri::command]
pub fn backup_database(state: State<DbState>) -> AppResult<String> {
    settings::backup_database(&state.data_dir)
}

#[tauri::command]
pub fn list_backups(state: State<DbState>) -> AppResult<Vec<String>> {
    settings::list_backups(&state.data_dir)
}
