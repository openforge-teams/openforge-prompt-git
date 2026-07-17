use crate::crypto::hash_password;
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;

pub fn load_settings(data_dir: &PathBuf, conn: &Connection) -> AppResult<AppSettings> {
    let config_path = data_dir.join("config.json");
    let mut settings = if config_path.exists() {
        let raw = fs::read_to_string(&config_path)?;
        serde_json::from_str::<AppSettings>(&raw).unwrap_or_default()
    } else {
        AppSettings::default()
    };
    let has_password: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'app_password_hash'",
            [],
            |row| row.get(0),
        )
        .ok();
    settings.has_app_password = has_password.is_some();
    Ok(settings)
}

pub fn save_settings(data_dir: &PathBuf, settings: &AppSettings) -> AppResult<()> {
    let config_path = data_dir.join("config.json");
    let mut to_save = settings.clone();
    // Don't persist password flag as source of truth
    to_save.has_app_password = false;
    fs::write(config_path, serde_json::to_string_pretty(&to_save)?)?;
    Ok(())
}

pub fn set_app_password(conn: &Connection, password: &str) -> AppResult<()> {
    if password.len() < 4 {
        return Err(AppError::msg("password must be at least 4 characters"));
    }
    let hash = hash_password(password);
    conn.execute(
        "INSERT OR REPLACE INTO meta(key, value) VALUES('app_password_hash', ?1)",
        params![hash],
    )?;
    Ok(())
}

pub fn clear_app_password(conn: &Connection) -> AppResult<()> {
    conn.execute("DELETE FROM meta WHERE key = 'app_password_hash'", [])?;
    Ok(())
}

pub fn verify_app_password(conn: &Connection, password: &str) -> AppResult<bool> {
    let stored: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'app_password_hash'",
            [],
            |row| row.get(0),
        )
        .ok();
    match stored {
        None => Ok(true),
        Some(hash) => Ok(hash == hash_password(password)),
    }
}

pub fn backup_database(data_dir: &PathBuf) -> AppResult<String> {
    let src = data_dir.join("data.db");
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let dest = data_dir.join("backups").join(format!("backup_{stamp}.db"));
    fs::copy(&src, &dest)?;
    Ok(dest.to_string_lossy().to_string())
}

pub fn list_backups(data_dir: &PathBuf) -> AppResult<Vec<String>> {
    let dir = data_dir.join("backups");
    let mut files = Vec::new();
    if dir.exists() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("db") {
                files.push(entry.path().to_string_lossy().to_string());
            }
        }
    }
    files.sort();
    files.reverse();
    Ok(files)
}
