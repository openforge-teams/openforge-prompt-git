use crate::error::AppResult;
use crate::llm::chat_with_config;
use crate::models::{BatchResult, BatchTask, InferenceParams, VariableCase};
use crate::services::model::get_model_raw;
use crate::services::template::{cartesian_product, extract_variables, render_template};
use chrono::Utc;
use futures::stream::{self, StreamExt};
use rusqlite::{params, Connection};
use std::sync::Mutex;

fn now() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

pub fn parse_variables(template: &str) -> Vec<String> {
    extract_variables(template)
}

pub fn generate_cases(
    variable_values: &serde_json::Map<String, serde_json::Value>,
) -> Vec<VariableCase> {
    cartesian_product(variable_values)
        .into_iter()
        .map(|variables| VariableCase { variables })
        .collect()
}

pub async fn run_batch_test(
    db: &Mutex<Connection>,
    project_id: i64,
    template: &str,
    system_prompt: Option<&str>,
    model_config_id: i64,
    cases: Vec<VariableCase>,
    params: InferenceParams,
    concurrency: usize,
) -> AppResult<BatchTask> {
    let task_id = {
        let conn = db.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        let ts = now();
        let params_json = serde_json::to_string(&params)?;
        conn.execute(
            "INSERT INTO batch_tasks(project_id, template, system_prompt, model_config_id, params, status, concurrency, created_at)
             VALUES(?1, ?2, ?3, ?4, ?5, 'running', ?6, ?7)",
            params![
                project_id,
                template,
                system_prompt,
                model_config_id,
                params_json,
                concurrency as i64,
                ts
            ],
        )?;
        conn.last_insert_rowid()
    };

    let (model_type, model_name, api_base, api_key, _, _) = {
        let conn = db.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        get_model_raw(&conn, model_config_id)?
    };

    let jobs: Vec<_> = cases
        .into_iter()
        .enumerate()
        .map(|(idx, case)| {
            let rendered = render_template(template, &case.variables);
            let model_type = model_type.clone();
            let model_name = model_name.clone();
            let api_base = api_base.clone();
            let api_key = api_key.clone();
            let sys = system_prompt.map(|s| s.to_string());
            let params = params.clone();
            async move {
                let result = chat_with_config(
                    &model_type,
                    &model_name,
                    api_base.as_deref(),
                    api_key.as_deref(),
                    sys.as_deref(),
                    &rendered,
                    &params,
                )
                .await;
                (idx as i64, case, rendered, result)
            }
        })
        .collect();

    let outcomes: Vec<_> = stream::iter(jobs)
        .buffer_unordered(concurrency.max(1))
        .collect()
        .await;

    {
        let conn = db.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        for (idx, case, rendered, result) in outcomes {
            let vars_json = serde_json::to_string(&case.variables)?;
            match result {
                Ok(resp) => {
                    conn.execute(
                        "INSERT INTO batch_results(task_id, case_index, variables, rendered_prompt, output_content, latency, status)
                         VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'success')",
                        params![task_id, idx, vars_json, rendered, resp.content, resp.latency_ms],
                    )?;
                }
                Err(err) => {
                    conn.execute(
                        "INSERT INTO batch_results(task_id, case_index, variables, rendered_prompt, status, error_msg)
                         VALUES(?1, ?2, ?3, ?4, 'failed', ?5)",
                        params![task_id, idx, vars_json, rendered, err.to_string()],
                    )?;
                }
            }
        }
        conn.execute(
            "UPDATE batch_tasks SET status = 'completed' WHERE id = ?1",
            params![task_id],
        )?;
    }

    get_batch_task(db, task_id)
}

pub fn get_batch_task(db: &Mutex<Connection>, task_id: i64) -> AppResult<BatchTask> {
    let conn = db.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    let mut task = conn.query_row(
        "SELECT id, project_id, template, system_prompt, model_config_id, params, status, concurrency, created_at
         FROM batch_tasks WHERE id = ?1",
        params![task_id],
        |row| {
            let params: InferenceParams =
                serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default();
            Ok(BatchTask {
                id: row.get(0)?,
                project_id: row.get(1)?,
                template: row.get(2)?,
                system_prompt: row.get(3)?,
                model_config_id: row.get(4)?,
                params,
                status: row.get(6)?,
                concurrency: row.get(7)?,
                created_at: row.get(8)?,
                results: vec![],
            })
        },
    )?;
    task.results = list_batch_results(&conn, task_id)?;
    Ok(task)
}

fn list_batch_results(conn: &Connection, task_id: i64) -> AppResult<Vec<BatchResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, case_index, variables, rendered_prompt, output_content, score, latency, status, error_msg
         FROM batch_results WHERE task_id = ?1 ORDER BY case_index",
    )?;
    let rows = stmt.query_map(params![task_id], |row| {
        let vars: serde_json::Value =
            serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(serde_json::json!({}));
        Ok(BatchResult {
            id: row.get(0)?,
            task_id: row.get(1)?,
            case_index: row.get(2)?,
            variables: vars,
            rendered_prompt: row.get(4)?,
            output_content: row.get(5)?,
            score: row.get(6)?,
            latency: row.get(7)?,
            status: row.get(8)?,
            error_msg: row.get(9)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn score_batch_result(conn: &Connection, result_id: i64, score: f64) -> AppResult<()> {
    conn.execute(
        "UPDATE batch_results SET score = ?1 WHERE id = ?2",
        params![score, result_id],
    )?;
    Ok(())
}

pub fn save_test_suite(
    conn: &Connection,
    project_id: i64,
    name: &str,
    variables_schema: &serde_json::Value,
    cases: &serde_json::Value,
) -> AppResult<i64> {
    let ts = now();
    conn.execute(
        "INSERT INTO test_suites(project_id, name, variables_schema, cases, created_at) VALUES(?1, ?2, ?3, ?4, ?5)",
        params![
            project_id,
            name,
            serde_json::to_string(variables_schema)?,
            serde_json::to_string(cases)?,
            ts
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_test_suites(conn: &Connection, project_id: i64) -> AppResult<Vec<serde_json::Value>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, variables_schema, cases, created_at FROM test_suites WHERE project_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
            "variables_schema": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(2)?).unwrap_or_default(),
            "cases": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(3)?).unwrap_or_default(),
            "created_at": row.get::<_, String>(4)?,
        }))
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}
