use crate::error::{AppError, AppResult};
use crate::models::{DiffLine, PromptVersion, VersionDiff, VersionTag};
use crate::services::project::{get_file_project_id, touch_project};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use sha1::{Digest, Sha1};
use similar::{ChangeTag, TextDiff};

fn now() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

pub fn make_version_hash(file_id: i64, system_prompt: &str, user_prompt: &str, ts: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(file_id.to_string().as_bytes());
    hasher.update(system_prompt.as_bytes());
    hasher.update(user_prompt.as_bytes());
    hasher.update(ts.as_bytes());
    // Extra entropy avoids UNIQUE collisions when the same content is
    // committed twice within the same timestamp resolution (e.g. rollback snapshot).
    hasher.update(uuid::Uuid::new_v4().as_bytes());
    hex::encode(hasher.finalize())
}

pub fn commit_version_internal(
    conn: &Connection,
    file_id: i64,
    system_prompt: &str,
    user_prompt: &str,
    commit_message: &str,
    remark: Option<&str>,
    tag_ids: &[i64],
) -> AppResult<String> {
    let ts = now();
    let hash = make_version_hash(file_id, system_prompt, user_prompt, &ts);
    conn.execute(
        "INSERT INTO prompt_versions(version_hash, prompt_file_id, system_prompt, user_prompt, commit_message, remark, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![hash, file_id, system_prompt, user_prompt, commit_message, remark, ts],
    )?;
    for tag_id in tag_ids {
        conn.execute(
            "INSERT OR IGNORE INTO version_tag_relations(version_hash, tag_id) VALUES(?1, ?2)",
            params![hash, tag_id],
        )?;
    }
    conn.execute(
        "UPDATE prompt_files SET current_version_hash = ?1, updated_at = ?2 WHERE id = ?3",
        params![hash, ts, file_id],
    )?;
    if let Ok(project_id) = get_file_project_id(conn, file_id) {
        let _ = touch_project(conn, project_id);
    }
    Ok(hash)
}

pub fn commit_prompt_version(
    conn: &Connection,
    file_id: i64,
    system_prompt: &str,
    user_prompt: &str,
    commit_message: &str,
    remark: Option<&str>,
    tag_ids: Vec<i64>,
) -> AppResult<PromptVersion> {
    if commit_message.trim().is_empty() {
        return Err(AppError::msg("commit message is required"));
    }
    let hash = commit_version_internal(
        conn,
        file_id,
        system_prompt,
        user_prompt,
        commit_message,
        remark,
        &tag_ids,
    )?;
    get_version_by_hash(conn, &hash)
}

pub fn get_version_by_hash(conn: &Connection, hash: &str) -> AppResult<PromptVersion> {
    let mut version = conn.query_row(
        "SELECT id, version_hash, prompt_file_id, system_prompt, user_prompt, commit_message, remark, created_at
         FROM prompt_versions WHERE version_hash = ?1",
        params![hash],
        |row| {
            Ok(PromptVersion {
                id: row.get(0)?,
                version_hash: row.get(1)?,
                prompt_file_id: row.get(2)?,
                system_prompt: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                user_prompt: row.get(4)?,
                commit_message: row.get(5)?,
                remark: row.get(6)?,
                created_at: row.get(7)?,
                tags: vec![],
            })
        },
    )?;
    version.tags = get_tags_for_version(conn, hash)?;
    Ok(version)
}

pub fn get_version_history(conn: &Connection, file_id: i64) -> AppResult<Vec<PromptVersion>> {
    let mut stmt = conn.prepare(
        "SELECT version_hash FROM prompt_versions WHERE prompt_file_id = ?1 ORDER BY created_at DESC",
    )?;
    let hashes: Vec<String> = stmt
        .query_map(params![file_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    let mut versions = Vec::new();
    for hash in hashes {
        versions.push(get_version_by_hash(conn, &hash)?);
    }
    Ok(versions)
}

pub fn update_version_remark(conn: &Connection, version_hash: &str, remark: &str) -> AppResult<PromptVersion> {
    conn.execute(
        "UPDATE prompt_versions SET remark = ?1 WHERE version_hash = ?2",
        params![remark, version_hash],
    )?;
    get_version_by_hash(conn, version_hash)
}

pub fn rollback_version(conn: &Connection, file_id: i64, target_hash: &str) -> AppResult<PromptVersion> {
    let current = conn.query_row(
        "SELECT current_version_hash FROM prompt_files WHERE id = ?1",
        params![file_id],
        |row| row.get::<_, String>(0),
    )?;
    if !current.is_empty() {
        let cur = get_version_by_hash(conn, &current)?;
        commit_version_internal(
            conn,
            file_id,
            &cur.system_prompt,
            &cur.user_prompt,
            "Snapshot before rollback",
            Some(&format!("Auto snapshot before rollback to {target_hash}")),
            &[],
        )?;
    }
    let target = get_version_by_hash(conn, target_hash)?;
    let hash = commit_version_internal(
        conn,
        file_id,
        &target.system_prompt,
        &target.user_prompt,
        &format!("Rollback to {}", &target_hash[..8.min(target_hash.len())]),
        Some(&format!("Restored from {target_hash}")),
        &[],
    )?;
    get_version_by_hash(conn, &hash)
}

fn diff_text(old: &str, new: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(old, new);
    let mut lines = Vec::new();
    let mut old_no = 1usize;
    let mut new_no = 1usize;
    for change in diff.iter_all_changes() {
        let text = change.value().trim_end_matches('\n').to_string();
        match change.tag() {
            ChangeTag::Equal => {
                lines.push(DiffLine {
                    kind: "equal".into(),
                    old_line: Some(text.clone()),
                    new_line: Some(text),
                    old_no: Some(old_no),
                    new_no: Some(new_no),
                });
                old_no += 1;
                new_no += 1;
            }
            ChangeTag::Delete => {
                lines.push(DiffLine {
                    kind: "delete".into(),
                    old_line: Some(text),
                    new_line: None,
                    old_no: Some(old_no),
                    new_no: None,
                });
                old_no += 1;
            }
            ChangeTag::Insert => {
                lines.push(DiffLine {
                    kind: "insert".into(),
                    old_line: None,
                    new_line: Some(text),
                    old_no: None,
                    new_no: Some(new_no),
                });
                new_no += 1;
            }
        }
    }
    lines
}

pub fn diff_versions(conn: &Connection, hash_a: &str, hash_b: &str) -> AppResult<VersionDiff> {
    let a = get_version_by_hash(conn, hash_a)?;
    let b = get_version_by_hash(conn, hash_b)?;
    Ok(VersionDiff {
        system_diff: diff_text(&a.system_prompt, &b.system_prompt),
        user_diff: diff_text(&a.user_prompt, &b.user_prompt),
        version_a: a,
        version_b: b,
    })
}

pub fn list_tags(conn: &Connection, project_id: i64) -> AppResult<Vec<VersionTag>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, project_id FROM version_tags WHERE project_id = ?1 ORDER BY name",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(VersionTag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            project_id: row.get(3)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn create_tag(
    conn: &Connection,
    project_id: i64,
    name: &str,
    color: &str,
) -> AppResult<VersionTag> {
    conn.execute(
        "INSERT INTO version_tags(name, color, project_id) VALUES(?1, ?2, ?3)",
        params![name, color, project_id],
    )?;
    let id = conn.last_insert_rowid();
    Ok(VersionTag {
        id,
        name: name.to_string(),
        color: color.to_string(),
        project_id,
    })
}

pub fn delete_tag(conn: &Connection, tag_id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM version_tags WHERE id = ?1", params![tag_id])?;
    Ok(())
}

pub fn attach_tag(conn: &Connection, version_hash: &str, tag_id: i64) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO version_tag_relations(version_hash, tag_id) VALUES(?1, ?2)",
        params![version_hash, tag_id],
    )?;
    Ok(())
}

pub fn detach_tag(conn: &Connection, version_hash: &str, tag_id: i64) -> AppResult<()> {
    conn.execute(
        "DELETE FROM version_tag_relations WHERE version_hash = ?1 AND tag_id = ?2",
        params![version_hash, tag_id],
    )?;
    Ok(())
}

fn get_tags_for_version(conn: &Connection, version_hash: &str) -> AppResult<Vec<VersionTag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.project_id
         FROM version_tags t
         JOIN version_tag_relations r ON r.tag_id = t.id
         WHERE r.version_hash = ?1",
    )?;
    let rows = stmt.query_map(params![version_hash], |row| {
        Ok(VersionTag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            project_id: row.get(3)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn filter_history_by_tag(
    conn: &Connection,
    file_id: i64,
    tag_id: i64,
) -> AppResult<Vec<PromptVersion>> {
    let mut stmt = conn.prepare(
        "SELECT v.version_hash FROM prompt_versions v
         JOIN version_tag_relations r ON r.version_hash = v.version_hash
         WHERE v.prompt_file_id = ?1 AND r.tag_id = ?2
         ORDER BY v.created_at DESC",
    )?;
    let hashes: Vec<String> = stmt
        .query_map(params![file_id, tag_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    let mut versions = Vec::new();
    for hash in hashes {
        versions.push(get_version_by_hash(conn, &hash)?);
    }
    Ok(versions)
}

#[allow(dead_code)]
pub fn version_exists(conn: &Connection, hash: &str) -> AppResult<bool> {
    let found: Option<String> = conn
        .query_row(
            "SELECT version_hash FROM prompt_versions WHERE version_hash = ?1",
            params![hash],
            |row| row.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}
