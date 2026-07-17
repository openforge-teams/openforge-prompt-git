use crate::crypto::encrypt_secret;
use crate::error::AppResult;
use crate::models::{InferenceParams, ModelConfig, ModelConfigInput};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

fn now() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

fn row_to_config(
    id: i64,
    model_type: String,
    model_name: String,
    api_base: Option<String>,
    api_key: Option<String>,
    default_params: Option<String>,
    is_enabled: i64,
    created_at: String,
) -> ModelConfig {
    let params: InferenceParams = default_params
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    let key = api_key.unwrap_or_default();
    ModelConfig {
        id,
        model_type,
        model_name,
        api_base,
        api_key_masked: if key.is_empty() {
            String::new()
        } else {
            "••••••••".into()
        },
        has_api_key: !key.is_empty(),
        default_params: params,
        is_enabled: is_enabled == 1,
        created_at,
    }
}

pub fn list_models(conn: &Connection) -> AppResult<Vec<ModelConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, model_type, model_name, api_base, api_key, default_params, is_enabled, created_at
         FROM model_configs ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, String>(7)?,
        ))
    })?;
    Ok(rows
        .filter_map(|r| r.ok())
        .map(|(id, mt, mn, ab, ak, dp, en, ca)| {
            row_to_config(id, mt, mn, ab, ak, dp, en, ca)
        })
        .collect())
}

pub fn get_model_raw(
    conn: &Connection,
    id: i64,
) -> AppResult<(String, String, Option<String>, Option<String>, InferenceParams, bool)> {
    conn.query_row(
        "SELECT model_type, model_name, api_base, api_key, default_params, is_enabled FROM model_configs WHERE id = ?1",
        params![id],
        |row| {
            let params_str: Option<String> = row.get(4)?;
            let params: InferenceParams = params_str
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                params,
                row.get::<_, i64>(5)? == 1,
            ))
        },
    )
    .map_err(Into::into)
}

pub fn save_model_config(conn: &Connection, input: ModelConfigInput) -> AppResult<ModelConfig> {
    let params_json = serde_json::to_string(&input.default_params.unwrap_or_default())?;
    let enabled = input.is_enabled.unwrap_or(true) as i64;

    if let Some(id) = input.id {
        if let Some(api_key) = input.api_key {
            if !api_key.is_empty() {
                let enc = encrypt_secret(&api_key)?;
                conn.execute(
                    "UPDATE model_configs SET model_type=?1, model_name=?2, api_base=?3, api_key=?4, default_params=?5, is_enabled=?6 WHERE id=?7",
                    params![input.model_type, input.model_name, input.api_base, enc, params_json, enabled, id],
                )?;
            } else {
                conn.execute(
                    "UPDATE model_configs SET model_type=?1, model_name=?2, api_base=?3, default_params=?4, is_enabled=?5 WHERE id=?6",
                    params![input.model_type, input.model_name, input.api_base, params_json, enabled, id],
                )?;
            }
        } else {
            conn.execute(
                "UPDATE model_configs SET model_type=?1, model_name=?2, api_base=?3, default_params=?4, is_enabled=?5 WHERE id=?6",
                params![input.model_type, input.model_name, input.api_base, params_json, enabled, id],
            )?;
        }
        return get_model(conn, id);
    }

    let enc = match &input.api_key {
        Some(k) if !k.is_empty() => Some(encrypt_secret(k)?),
        _ => None,
    };
    let ts = now();
    conn.execute(
        "INSERT INTO model_configs(model_type, model_name, api_base, api_key, default_params, is_enabled, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            input.model_type,
            input.model_name,
            input.api_base,
            enc,
            params_json,
            enabled,
            ts
        ],
    )?;
    get_model(conn, conn.last_insert_rowid())
}

pub fn get_model(conn: &Connection, id: i64) -> AppResult<ModelConfig> {
    let row = conn.query_row(
        "SELECT id, model_type, model_name, api_base, api_key, default_params, is_enabled, created_at
         FROM model_configs WHERE id = ?1",
        params![id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, String>(7)?,
            ))
        },
    )?;
    Ok(row_to_config(
        row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7,
    ))
}

pub fn delete_model(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM model_configs WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn model_exists(conn: &Connection, id: i64) -> AppResult<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT id FROM model_configs WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}
