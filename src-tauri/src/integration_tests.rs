use crate::db::open_memory_db;
use crate::models::{ExportCodeRequest, InferenceParams, ModelConfigInput, ScoreInput};
use crate::services::{export, model, project, settings, template, version};

#[test]
fn project_file_version_diff_rollback_tags() {
    let conn = open_memory_db().expect("db");
    let p = project::create_project(&conn, "Demo", Some("desc")).unwrap();
    assert_eq!(p.name, "Demo");

    let tags = version::list_tags(&conn, p.id).unwrap();
    assert!(tags.iter().any(|t| t.name == "Production"));
    assert!(tags.iter().any(|t| t.name == "Testing"));

    let folder = project::create_folder(&conn, p.id, None, "core").unwrap();
    let file = project::create_prompt_file(
        &conn,
        p.id,
        Some(folder.id),
        "greeter.prompt",
        Some("You are helpful."),
        Some("Hello {{name}}"),
    )
    .unwrap();
    assert!(!file.current_version_hash.is_empty());

    let v1 = version::commit_prompt_version(
        &conn,
        file.id,
        "You are helpful.",
        "Hello {{name}}, welcome!",
        "improve greeting",
        Some("looks better"),
        vec![tags[0].id],
    )
    .unwrap();
    assert_eq!(v1.commit_message, "improve greeting");
    assert!(!v1.tags.is_empty());

    let history = version::get_version_history(&conn, file.id).unwrap();
    assert!(history.len() >= 2);

    let older = &history[1];
    let newer = &history[0];
    let diff = version::diff_versions(&conn, &older.version_hash, &newer.version_hash).unwrap();
    assert!(diff
        .user_diff
        .iter()
        .any(|l| l.kind == "insert" || l.kind == "delete" || l.kind == "equal"));

    let rolled = version::rollback_version(&conn, file.id, &older.version_hash).unwrap();
    let after = project::get_prompt_file(&conn, file.id).unwrap();
    assert_eq!(after.user_prompt, older.user_prompt);
    assert_eq!(after.current_version_hash, rolled.version_hash);

    // Cross-file rollback currently succeeds (BUG).
    let other = project::create_prompt_file(
        &conn,
        p.id,
        None,
        "other.prompt",
        None,
        Some("other content"),
    )
    .unwrap();
    let other_hist = version::get_version_history(&conn, other.id).unwrap();
    let cross = version::rollback_version(&conn, file.id, &other_hist[0].version_hash);
    assert!(cross.is_ok(), "documenting current bug: cross-file rollback is allowed");
    let polluted = project::get_prompt_file(&conn, file.id).unwrap();
    assert_eq!(
        polluted.user_prompt, "other content",
        "BUG confirmed: rollback copied another file's content"
    );
}

#[test]
fn template_cartesian_csv_and_export() {
    let vars = template::extract_variables("Hi {{name:Guest}} from {{city}}");
    assert_eq!(vars, vec!["city".to_string(), "name".to_string()]);

    let mut map = serde_json::Map::new();
    map.insert("name".into(), serde_json::json!(["Ada", "Bob"]));
    map.insert("city".into(), serde_json::json!(["Paris"]));
    let cases = template::cartesian_product(&map);
    assert_eq!(cases.len(), 2);

    let rendered = template::render_template(
        "Hi {{name:Guest}} from {{city}}",
        &{
            let mut m = serde_json::Map::new();
            m.insert("city".into(), serde_json::json!("Tokyo"));
            m
        },
    );
    assert_eq!(rendered, "Hi Guest from Tokyo");

    let (headers, rows) = template::parse_csv_cases("name,city\nAda,Paris\nBob,Berlin").unwrap();
    assert_eq!(headers, vec!["name", "city"]);
    assert_eq!(rows.len(), 2);

    let conn = open_memory_db().unwrap();
    let p = project::create_project(&conn, "Export", None).unwrap();
    let file =
        project::create_prompt_file(&conn, p.id, None, "x.prompt", Some("sys"), Some("user {{x}}"))
            .unwrap();
    let hash = file.current_version_hash.clone();
    let json = export::export_json(&conn, file.id, &hash, Some("gpt-4o-mini"), None).unwrap();
    assert!(json.contains("user_prompt"));
    let yaml = export::export_yaml(&conn, file.id, &hash).unwrap();
    assert!(yaml.contains("prompt_info"));

    let code = export::export_code_snippet(ExportCodeRequest {
        system_prompt: "sys".into(),
        user_prompt: "hi".into(),
        model_name: "gpt-4o-mini".into(),
        language: "python".into(),
        params: InferenceParams::default(),
    })
    .unwrap();
    assert!(code.contains("OpenAI"));

    let md = export::export_markdown_report(
        "demo",
        "sys",
        "user",
        &InferenceParams::default(),
        &[(
            "model-a".into(),
            "out".into(),
            Some(8.0),
            Some(120),
            Some("ok".into()),
        )],
    );
    assert!(md.contains("Prompt Compare Report"));
}

#[test]
fn model_config_and_settings_password() {
    let conn = open_memory_db().unwrap();
    let saved = model::save_model_config(
        &conn,
        ModelConfigInput {
            id: None,
            model_type: "openai".into(),
            model_name: "gpt-4o-mini".into(),
            api_base: None,
            api_key: Some("sk-test-key-12345678".into()),
            default_params: Some(InferenceParams::default()),
            is_enabled: Some(true),
        },
    );
    match saved {
        Ok(cfg) => {
            assert!(cfg.has_api_key);
            assert_eq!(cfg.model_name, "gpt-4o-mini");
            let list = model::list_models(&conn).unwrap();
            assert_eq!(list.len(), 1);
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.to_lowercase().contains("keyring")
                    || msg.to_lowercase().contains("encryption")
                    || msg.to_lowercase().contains("encrypt"),
                "unexpected model save error: {msg}"
            );
        }
    }

    let dir = tempfile::tempdir().unwrap();
    let settings_val = crate::models::AppSettings::default();
    settings::save_settings(&dir.path().to_path_buf(), &settings_val).unwrap();
    let loaded = settings::load_settings(&dir.path().to_path_buf(), &conn).unwrap();
    assert_eq!(loaded.theme, "system");

    settings::set_app_password(&conn, "secret").unwrap();
    assert!(settings::verify_app_password(&conn, "secret").unwrap());
    assert!(!settings::verify_app_password(&conn, "wrong").unwrap());
    settings::clear_app_password(&conn).unwrap();
    assert!(settings::verify_app_password(&conn, "anything").unwrap());
}

#[test]
fn compare_score_overwrite_bug_confirmed() {
    let conn = open_memory_db().unwrap();
    let p = project::create_project(&conn, "Cmp", None).unwrap();
    conn.execute(
        "INSERT INTO compare_tasks(project_id, prompt_content, models, params, status, created_at)
         VALUES(?1, 'hi', '[1]', '{}', 'completed', '2026-01-01')",
        rusqlite::params![p.id],
    )
    .unwrap();
    let task_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO compare_results(task_id, model_config_id, model_name, output_content, status, is_best, created_at)
         VALUES(?1, 1, 'm', 'out', 'success', 0, '2026-01-01')",
        rusqlite::params![task_id],
    )
    .unwrap();
    let result_id = conn.last_insert_rowid();

    crate::services::compare::score_result(
        &conn,
        result_id,
        ScoreInput {
            accuracy: Some(9.0),
            instruction: None,
            format: None,
            speed: None,
            evaluation: None,
            is_best: None,
        },
    )
    .unwrap();

    let after_instruction = crate::services::compare::score_result(
        &conn,
        result_id,
        ScoreInput {
            accuracy: None,
            instruction: Some(8.0),
            format: None,
            speed: None,
            evaluation: None,
            is_best: None,
        },
    )
    .unwrap();

    let scores = after_instruction.scores.clone().unwrap();
    assert!(
        scores.get("accuracy").is_none(),
        "BUG confirmed: accuracy wiped when scoring instruction alone; got {scores:?}"
    );
    assert_eq!(scores.get("instruction").and_then(|v| v.as_f64()), Some(8.0));
}

#[test]
fn backup_database_works() {
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().to_path_buf();
    std::fs::create_dir_all(data_dir.join("backups")).unwrap();
    std::fs::write(data_dir.join("data.db"), b"sqlite-dummy").unwrap();
    let path = settings::backup_database(&data_dir).unwrap();
    assert!(std::path::Path::new(&path).exists());
    let list = settings::list_backups(&data_dir).unwrap();
    assert!(!list.is_empty());
}
