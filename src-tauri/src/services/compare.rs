use crate::error::AppResult;
use crate::llm::chat_with_config;
use crate::models::{CompareResult, CompareTask, InferenceParams, ScoreInput};
use crate::services::model::get_model_raw;
use chrono::Utc;
use futures::future::join_all;
use rusqlite::{params, Connection};
use std::sync::Mutex;

fn now() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

pub fn create_compare_task(
    conn: &Connection,
    project_id: i64,
    prompt_content: &str,
    system_prompt: Option<&str>,
    prompt_version_hash: Option<&str>,
    model_ids: &[i64],
    params: &InferenceParams,
) -> AppResult<i64> {
    let ts = now();
    let models_json = serde_json::to_string(model_ids)?;
    let params_json = serde_json::to_string(params)?;
    conn.execute(
        "INSERT INTO compare_tasks(project_id, prompt_version_hash, prompt_content, system_prompt, models, params, status, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7)",
        params![
            project_id,
            prompt_version_hash,
            prompt_content,
            system_prompt,
            models_json,
            params_json,
            ts
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub async fn run_compare_task(db: &Mutex<Connection>, task_id: i64) -> AppResult<CompareTask> {
    let (prompt_content, system_prompt, model_ids, params) = {
        let conn = db.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        conn.execute(
            "UPDATE compare_tasks SET status = 'running' WHERE id = ?1",
            params![task_id],
        )?;
        let row: (String, Option<String>, String, String) = conn.query_row(
            "SELECT prompt_content, system_prompt, models, params FROM compare_tasks WHERE id = ?1",
            params![task_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        let model_ids: Vec<i64> = serde_json::from_str(&row.2)?;
        let params: InferenceParams = serde_json::from_str(&row.3)?;
        (row.0, row.1, model_ids, params)
    };

    let mut jobs = Vec::new();
    for model_id in model_ids {
        let (model_type, model_name, api_base, api_key, model_params, _) = {
            let conn = db.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
            get_model_raw(&conn, model_id)?
        };
        let merged = InferenceParams {
            temperature: params.temperature,
            max_tokens: params.max_tokens,
            top_p: params.top_p,
            frequency_penalty: params.frequency_penalty,
        };
        let _ = model_params;
        let prompt = prompt_content.clone();
        let sys = system_prompt.clone();
        jobs.push(async move {
            let result = chat_with_config(
                &model_type,
                &model_name,
                api_base.as_deref(),
                api_key.as_deref(),
                sys.as_deref(),
                &prompt,
                &merged,
            )
            .await;
            (model_id, model_name, result)
        });
    }

    let outcomes = join_all(jobs).await;
    {
        let conn = db.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        let ts = now();
        for (model_id, model_name, result) in outcomes {
            match result {
                Ok(resp) => {
                    conn.execute(
                        "INSERT INTO compare_results(task_id, model_config_id, model_name, output_content, latency, status, created_at)
                         VALUES(?1, ?2, ?3, ?4, ?5, 'success', ?6)",
                        params![task_id, model_id, model_name, resp.content, resp.latency_ms, ts],
                    )?;
                }
                Err(err) => {
                    conn.execute(
                        "INSERT INTO compare_results(task_id, model_config_id, model_name, status, error_msg, created_at)
                         VALUES(?1, ?2, ?3, 'failed', ?4, ?5)",
                        params![task_id, model_id, model_name, err.to_string(), ts],
                    )?;
                }
            }
        }
        conn.execute(
            "UPDATE compare_tasks SET status = 'completed' WHERE id = ?1",
            params![task_id],
        )?;
    }

    get_compare_task(db, task_id)
}

pub fn get_compare_task(db: &Mutex<Connection>, task_id: i64) -> AppResult<CompareTask> {
    let conn = db.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    let mut task = conn.query_row(
        "SELECT id, project_id, prompt_version_hash, prompt_content, system_prompt, models, params, status, created_at
         FROM compare_tasks WHERE id = ?1",
        params![task_id],
        |row| {
            let models: Vec<i64> = serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default();
            let params: InferenceParams =
                serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default();
            Ok(CompareTask {
                id: row.get(0)?,
                project_id: row.get(1)?,
                prompt_version_hash: row.get(2)?,
                prompt_content: row.get(3)?,
                system_prompt: row.get(4)?,
                models,
                params,
                status: row.get(7)?,
                created_at: row.get(8)?,
                results: vec![],
            })
        },
    )?;
    task.results = list_results(&conn, task_id)?;
    Ok(task)
}

fn list_results(conn: &Connection, task_id: i64) -> AppResult<Vec<CompareResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, model_config_id, model_name, output_content, scores, total_score, evaluation, latency, status, error_msg, is_best, created_at
         FROM compare_results WHERE task_id = ?1 ORDER BY id",
    )?;
    let rows = stmt.query_map(params![task_id], |row| {
        let scores_str: Option<String> = row.get(5)?;
        Ok(CompareResult {
            id: row.get(0)?,
            task_id: row.get(1)?,
            model_config_id: row.get(2)?,
            model_name: row.get(3)?,
            output_content: row.get(4)?,
            scores: scores_str.and_then(|s| serde_json::from_str(&s).ok()),
            total_score: row.get(6)?,
            evaluation: row.get(7)?,
            latency: row.get(8)?,
            status: row.get(9)?,
            error_msg: row.get(10)?,
            is_best: row.get::<_, i64>(11)? == 1,
            created_at: row.get(12)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_compare_tasks(conn: &Connection, project_id: i64) -> AppResult<Vec<CompareTask>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM compare_tasks WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 50",
    )?;
    let ids: Vec<i64> = stmt
        .query_map(params![project_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    // Use a dummy mutex wrapper - caller should use get for each
    let mut tasks = Vec::new();
    for id in ids {
        let mut task = conn.query_row(
            "SELECT id, project_id, prompt_version_hash, prompt_content, system_prompt, models, params, status, created_at
             FROM compare_tasks WHERE id = ?1",
            params![id],
            |row| {
                let models: Vec<i64> =
                    serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default();
                let params: InferenceParams =
                    serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default();
                Ok(CompareTask {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    prompt_version_hash: row.get(2)?,
                    prompt_content: row.get(3)?,
                    system_prompt: row.get(4)?,
                    models,
                    params,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    results: vec![],
                })
            },
        )?;
        task.results = list_results(conn, id)?;
        tasks.push(task);
    }
    Ok(tasks)
}

pub fn score_result(conn: &Connection, result_id: i64, input: ScoreInput) -> AppResult<CompareResult> {
    let mut scores = serde_json::Map::new();
    let mut total = 0.0;
    let mut count = 0.0;
    if let Some(v) = input.accuracy {
        scores.insert("accuracy".into(), serde_json::json!(v));
        total += v;
        count += 1.0;
    }
    if let Some(v) = input.instruction {
        scores.insert("instruction".into(), serde_json::json!(v));
        total += v;
        count += 1.0;
    }
    if let Some(v) = input.format {
        scores.insert("format".into(), serde_json::json!(v));
        total += v;
        count += 1.0;
    }
    if let Some(v) = input.speed {
        scores.insert("speed".into(), serde_json::json!(v));
        total += v;
        count += 1.0;
    }
    let avg = if count > 0.0 { Some(total / count) } else { None };
    let scores_json = serde_json::to_string(&scores)?;
    let is_best = input.is_best.unwrap_or(false) as i64;

    if is_best == 1 {
        let task_id: i64 = conn.query_row(
            "SELECT task_id FROM compare_results WHERE id = ?1",
            params![result_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "UPDATE compare_results SET is_best = 0 WHERE task_id = ?1",
            params![task_id],
        )?;
    }

    conn.execute(
        "UPDATE compare_results SET scores = ?1, total_score = ?2, evaluation = COALESCE(?3, evaluation), is_best = COALESCE(?4, is_best) WHERE id = ?5",
        params![scores_json, avg, input.evaluation, is_best, result_id],
    )?;

    conn.query_row(
        "SELECT id, task_id, model_config_id, model_name, output_content, scores, total_score, evaluation, latency, status, error_msg, is_best, created_at
         FROM compare_results WHERE id = ?1",
        params![result_id],
        |row| {
            let scores_str: Option<String> = row.get(5)?;
            Ok(CompareResult {
                id: row.get(0)?,
                task_id: row.get(1)?,
                model_config_id: row.get(2)?,
                model_name: row.get(3)?,
                output_content: row.get(4)?,
                scores: scores_str.and_then(|s| serde_json::from_str(&s).ok()),
                total_score: row.get(6)?,
                evaluation: row.get(7)?,
                latency: row.get(8)?,
                status: row.get(9)?,
                error_msg: row.get(10)?,
                is_best: row.get::<_, i64>(11)? == 1,
                created_at: row.get(12)?,
            })
        },
    )
    .map_err(Into::into)
}
