mod commands;
mod crypto;
mod db;
mod error;
mod llm;
mod models;
mod services;

#[cfg(test)]
mod integration_tests;

use db::init_db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_state = init_db().expect("failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(db_state)
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::update_project,
            commands::delete_project,
            commands::create_folder,
            commands::list_folders,
            commands::delete_folder,
            commands::create_prompt_file,
            commands::list_prompt_files,
            commands::get_prompt_file,
            commands::rename_prompt_file,
            commands::delete_prompt_file,
            commands::commit_prompt_version,
            commands::get_version_history,
            commands::get_version,
            commands::diff_versions,
            commands::rollback_version,
            commands::update_version_remark,
            commands::list_tags,
            commands::create_tag,
            commands::delete_tag,
            commands::attach_tag,
            commands::detach_tag,
            commands::filter_history_by_tag,
            commands::list_models,
            commands::save_model_config,
            commands::delete_model_config,
            commands::list_ollama_models,
            commands::run_compare_task,
            commands::get_compare_result,
            commands::list_compare_tasks,
            commands::score_compare_result,
            commands::extract_template_variables,
            commands::generate_variable_cases,
            commands::parse_csv_cases,
            commands::run_batch_test,
            commands::get_batch_result,
            commands::score_batch_result,
            commands::save_test_suite,
            commands::list_test_suites,
            commands::export_code_snippet,
            commands::export_json,
            commands::export_yaml,
            commands::export_markdown_report,
            commands::export_plain_prompt,
            commands::get_settings,
            commands::save_settings,
            commands::set_app_password,
            commands::clear_app_password,
            commands::verify_app_password,
            commands::backup_database,
            commands::list_backups,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Prompt Git");
}
