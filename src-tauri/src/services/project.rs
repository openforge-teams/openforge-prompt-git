use crate::error::AppResult;
use crate::models::{Folder, Project, PromptFile};
use crate::services::version::{commit_version_internal, get_version_by_hash};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

fn now() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

pub fn create_project(conn: &Connection, name: &str, description: Option<&str>) -> AppResult<Project> {
    let ts = now();
    conn.execute(
        "INSERT INTO projects(name, description, created_at, updated_at) VALUES(?1, ?2, ?3, ?4)",
        params![name, description, ts, ts],
    )?;
    let id = conn.last_insert_rowid();

    // Seed default tags for this project
    for (tag_name, color) in [("Production", "#22c55e"), ("Testing", "#3b82f6")] {
        conn.execute(
            "INSERT OR IGNORE INTO version_tags(name, color, project_id) VALUES(?1, ?2, ?3)",
            params![tag_name, color, id],
        )?;
    }

    get_project(conn, id)
}

pub fn list_projects(conn: &Connection) -> AppResult<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, created_at, updated_at FROM projects ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_project(conn: &Connection, id: i64) -> AppResult<Project> {
    conn.query_row(
        "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?1",
        params![id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )
    .map_err(Into::into)
}

pub fn update_project(
    conn: &Connection,
    id: i64,
    name: &str,
    description: Option<&str>,
) -> AppResult<Project> {
    let ts = now();
    conn.execute(
        "UPDATE projects SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
        params![name, description, ts, id],
    )?;
    get_project(conn, id)
}

pub fn delete_project(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn create_folder(
    conn: &Connection,
    project_id: i64,
    parent_id: Option<i64>,
    name: &str,
) -> AppResult<Folder> {
    let ts = now();
    conn.execute(
        "INSERT INTO folders(project_id, parent_id, name, created_at) VALUES(?1, ?2, ?3, ?4)",
        params![project_id, parent_id, name, ts],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Folder {
        id,
        project_id,
        parent_id,
        name: name.to_string(),
        created_at: ts,
    })
}

pub fn list_folders(conn: &Connection, project_id: i64) -> AppResult<Vec<Folder>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, parent_id, name, created_at FROM folders WHERE project_id = ?1 ORDER BY name",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(Folder {
            id: row.get(0)?,
            project_id: row.get(1)?,
            parent_id: row.get(2)?,
            name: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn create_prompt_file(
    conn: &Connection,
    project_id: i64,
    folder_id: Option<i64>,
    name: &str,
    system_prompt: Option<&str>,
    user_prompt: Option<&str>,
) -> AppResult<PromptFile> {
    let ts = now();
    let sys = system_prompt.unwrap_or("");
    let user = user_prompt.unwrap_or("Write your prompt here...");
    conn.execute(
        "INSERT INTO prompt_files(project_id, folder_id, name, current_version_hash, created_at, updated_at)
         VALUES(?1, ?2, ?3, '', ?4, ?5)",
        params![project_id, folder_id, name, ts, ts],
    )?;
    let file_id = conn.last_insert_rowid();
    let hash = commit_version_internal(
        conn,
        file_id,
        sys,
        user,
        "Initial commit",
        None,
        &[],
    )?;
    get_prompt_file(conn, file_id).map(|mut f| {
        f.current_version_hash = hash;
        f
    })
}

pub fn list_prompt_files(conn: &Connection, project_id: i64) -> AppResult<Vec<PromptFile>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, folder_id, name, current_version_hash, created_at, updated_at
         FROM prompt_files WHERE project_id = ?1 ORDER BY name",
    )?;
    let mut files = Vec::new();
    let rows = stmt.query_map(params![project_id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, Option<i64>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
        ))
    })?;
    for row in rows.filter_map(|r| r.ok()) {
        let (id, project_id, folder_id, name, hash, created_at, updated_at) = row;
        let (system_prompt, user_prompt) = if hash.is_empty() {
            (String::new(), String::new())
        } else {
            match get_version_by_hash(conn, &hash) {
                Ok(v) => (v.system_prompt, v.user_prompt),
                Err(_) => (String::new(), String::new()),
            }
        };
        files.push(PromptFile {
            id,
            project_id,
            folder_id,
            name,
            current_version_hash: hash,
            system_prompt,
            user_prompt,
            created_at,
            updated_at,
        });
    }
    Ok(files)
}

pub fn get_prompt_file(conn: &Connection, id: i64) -> AppResult<PromptFile> {
    let (project_id, folder_id, name, hash, created_at, updated_at) = conn.query_row(
        "SELECT project_id, folder_id, name, current_version_hash, created_at, updated_at
         FROM prompt_files WHERE id = ?1",
        params![id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        },
    )?;
    let (system_prompt, user_prompt) = if hash.is_empty() {
        (String::new(), String::new())
    } else {
        let v = get_version_by_hash(conn, &hash)?;
        (v.system_prompt, v.user_prompt)
    };
    Ok(PromptFile {
        id,
        project_id,
        folder_id,
        name,
        current_version_hash: hash,
        system_prompt,
        user_prompt,
        created_at,
        updated_at,
    })
}

pub fn rename_prompt_file(conn: &Connection, id: i64, name: &str) -> AppResult<PromptFile> {
    let ts = now();
    conn.execute(
        "UPDATE prompt_files SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, ts, id],
    )?;
    get_prompt_file(conn, id)
}

pub fn delete_prompt_file(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM prompt_files WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn delete_folder(conn: &Connection, id: i64) -> AppResult<()> {
    // Move files in folder to root
    conn.execute(
        "UPDATE prompt_files SET folder_id = NULL WHERE folder_id = ?1",
        params![id],
    )?;
    conn.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn touch_project(conn: &Connection, project_id: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE projects SET updated_at = ?1 WHERE id = ?2",
        params![now(), project_id],
    )?;
    Ok(())
}

pub fn get_file_project_id(conn: &Connection, file_id: i64) -> AppResult<i64> {
    conn.query_row(
        "SELECT project_id FROM prompt_files WHERE id = ?1",
        params![file_id],
        |row| row.get(0),
    )
    .map_err(Into::into)
}

#[allow(dead_code)]
pub fn file_exists(conn: &Connection, id: i64) -> AppResult<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT id FROM prompt_files WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}
