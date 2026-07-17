use crate::error::{AppError, AppResult};
use dirs::home_dir;
use rusqlite::{Connection, OptionalExtension};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct DbState {
    pub conn: Mutex<Connection>,
    pub data_dir: PathBuf,
}

pub fn app_data_dir() -> AppResult<PathBuf> {
    let home = home_dir().ok_or_else(|| AppError::msg("cannot resolve home directory"))?;
    let dir = home.join(".prompt-git");
    fs::create_dir_all(&dir)?;
    fs::create_dir_all(dir.join("backups"))?;
    fs::create_dir_all(dir.join("exports"))?;
    fs::create_dir_all(dir.join("cache"))?;
    Ok(dir)
}

pub fn init_db() -> AppResult<DbState> {
    let data_dir = app_data_dir()?;
    let db_path = data_dir.join("data.db");
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
    migrate(&conn)?;
    seed_default_tags(&conn)?;
    ensure_config_file(&data_dir)?;
    Ok(DbState {
        conn: Mutex::new(conn),
        data_dir,
    })
}

fn ensure_config_file(data_dir: &PathBuf) -> AppResult<()> {
    let config_path = data_dir.join("config.json");
    if !config_path.exists() {
        let default = serde_json::json!({
            "theme": "system",
            "auto_backup": true,
            "backup_interval_hours": 24,
            "default_concurrency": 3,
            "ollama_base": "http://127.0.0.1:11434"
        });
        fs::write(config_path, serde_json::to_string_pretty(&default)?)?;
    }
    Ok(())
}

fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            parent_id INTEGER,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY(parent_id) REFERENCES folders(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS prompt_files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            folder_id INTEGER,
            name TEXT NOT NULL,
            current_version_hash TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY(folder_id) REFERENCES folders(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS prompt_versions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version_hash TEXT NOT NULL UNIQUE,
            prompt_file_id INTEGER NOT NULL,
            system_prompt TEXT,
            user_prompt TEXT NOT NULL,
            commit_message TEXT NOT NULL,
            remark TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY(prompt_file_id) REFERENCES prompt_files(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS version_tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            color TEXT NOT NULL,
            project_id INTEGER NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
            UNIQUE(project_id, name)
        );

        CREATE TABLE IF NOT EXISTS version_tag_relations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version_hash TEXT NOT NULL,
            tag_id INTEGER NOT NULL,
            FOREIGN KEY(version_hash) REFERENCES prompt_versions(version_hash) ON DELETE CASCADE,
            FOREIGN KEY(tag_id) REFERENCES version_tags(id) ON DELETE CASCADE,
            UNIQUE(version_hash, tag_id)
        );

        CREATE TABLE IF NOT EXISTS model_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            model_type TEXT NOT NULL,
            model_name TEXT NOT NULL,
            api_base TEXT,
            api_key TEXT,
            default_params TEXT,
            is_enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS compare_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            prompt_version_hash TEXT,
            prompt_content TEXT NOT NULL,
            system_prompt TEXT,
            models TEXT NOT NULL,
            params TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS compare_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            model_config_id INTEGER NOT NULL,
            model_name TEXT NOT NULL,
            output_content TEXT,
            scores TEXT,
            total_score REAL,
            evaluation TEXT,
            latency INTEGER,
            status TEXT NOT NULL,
            error_msg TEXT,
            is_best INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY(task_id) REFERENCES compare_tasks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS batch_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            template TEXT NOT NULL,
            system_prompt TEXT,
            model_config_id INTEGER NOT NULL,
            params TEXT NOT NULL,
            status TEXT NOT NULL,
            concurrency INTEGER NOT NULL DEFAULT 3,
            created_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS batch_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            case_index INTEGER NOT NULL,
            variables TEXT NOT NULL,
            rendered_prompt TEXT NOT NULL,
            output_content TEXT,
            score REAL,
            latency INTEGER,
            status TEXT NOT NULL,
            error_msg TEXT,
            FOREIGN KEY(task_id) REFERENCES batch_tasks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS test_suites (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            variables_schema TEXT NOT NULL,
            cases TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )?;
    Ok(())
}

fn seed_default_tags(conn: &Connection) -> AppResult<()> {
    // Default tags are created per project on project creation.
    let _ = conn
        .query_row("SELECT value FROM meta WHERE key = 'schema_version'", [], |r| {
            r.get::<_, String>(0)
        })
        .optional()?;
    conn.execute(
        "INSERT OR REPLACE INTO meta(key, value) VALUES('schema_version', '1')",
        [],
    )?;
    Ok(())
}
