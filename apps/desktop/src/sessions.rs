use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use uuid::Uuid;

use crate::authz::{authorize, AuthzResult, Permission};
use crate::db::AppDatabase;
use crate::state::principal;

pub const ENTITY_TYPE_PROMPT: &str = "prompt";
pub const ENTITY_TYPE_AGENT: &str = "agent";
pub const MAX_TRANSCRIPT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TranscriptLine {
    RunConfig {
        instructions: String,
        model: String,
    },
    Message {
        role: String,
        content: String,
    },
    Meta {
        event: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub prompt_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub stopped: bool,
}

pub struct SessionService;

impl SessionService {
    fn require_read(db: &AppDatabase) -> Result<(), String> {
        let p = principal(db)?;
        match authorize(&p, Permission::PromptRead) {
            AuthzResult::Allowed => Ok(()),
            AuthzResult::Denied { .. } => Err("permission denied".into()),
        }
    }

    fn require_execute(db: &AppDatabase) -> Result<(), String> {
        let p = principal(db)?;
        match authorize(&p, Permission::PromptExecute) {
            AuthzResult::Allowed => Ok(()),
            AuthzResult::Denied { .. } => Err("permission denied".into()),
        }
    }

    fn owner_id(db: &AppDatabase) -> Result<String, String> {
        db.solo_user_id().map_err(|e| e.to_string())
    }

    pub fn transcript_relative_path(session_id: &str) -> String {
        format!("sessions/{session_id}/transcript.jsonl")
    }

    fn resolve_transcript_path(db: &AppDatabase, session_id: &str) -> Result<PathBuf, String> {
        let rel = Self::transcript_relative_path(session_id);
        let path = db.data_dir.join(&rel);
        let data_root = db
            .data_dir
            .canonicalize()
            .map_err(|e| e.to_string())?;

        if path.exists() {
            let canonical = path.canonicalize().map_err(|e| e.to_string())?;
            if !canonical.starts_with(&data_root) {
                return Err("invalid transcript path".into());
            }
            return Ok(canonical);
        }

        if !path.starts_with(&db.data_dir) {
            return Err("invalid transcript path".into());
        }
        Ok(path)
    }

    pub fn create(
        db: &AppDatabase,
        prompt_id: &str,
        instructions: &str,
        model: &str,
    ) -> Result<SessionSummary, String> {
        Self::require_execute(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_prompt_active(db, &owner_id, prompt_id)?;

        let session_id = Uuid::new_v4().to_string();
        let rel_path = Self::transcript_relative_path(&session_id);
        let now = Utc::now().to_rfc3339();

        let tx = db.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO sessions (id, owner_id, entity_type, entity_id, transcript_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                session_id,
                owner_id,
                ENTITY_TYPE_PROMPT,
                prompt_id,
                rel_path,
                now,
            ],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;

        let config = TranscriptLine::RunConfig {
            instructions: instructions.to_string(),
            model: model.to_string(),
        };
        Self::append_line(db, &session_id, prompt_id, &config)?;

        Ok(SessionSummary {
            id: session_id,
            prompt_id: prompt_id.to_string(),
            created_at: now.clone(),
            updated_at: now,
            stopped: false,
        })
    }

    pub fn append_line(
        db: &AppDatabase,
        session_id: &str,
        prompt_id: &str,
        line: &TranscriptLine,
    ) -> Result<(), String> {
        Self::require_execute(db)?;
        Self::assert_session_for_prompt(db, session_id, prompt_id)?;

        let path = Self::resolve_transcript_path(db, session_id)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string(line).map_err(|e| e.to_string())?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        file.write_all(json.as_bytes())
            .and_then(|_| file.write_all(b"\n"))
            .map_err(|e| e.to_string())?;

        let now = Utc::now().to_rfc3339();
        db.conn
            .execute(
                "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
                params![now, session_id],
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn mark_stopped(db: &AppDatabase, session_id: &str, prompt_id: &str) -> Result<(), String> {
        Self::append_line(
            db,
            session_id,
            prompt_id,
            &TranscriptLine::Meta {
                event: "stopped".into(),
            },
        )
    }

    pub fn list_for_prompt(
        db: &AppDatabase,
        prompt_id: &str,
    ) -> Result<Vec<SessionSummary>, String> {
        Self::require_read(db)?;
        let owner_id = Self::owner_id(db)?;

        let mut stmt = db
            .conn
            .prepare(
                "SELECT id, entity_id, created_at, updated_at, transcript_path
                 FROM sessions
                 WHERE owner_id = ?1 AND entity_type = ?2 AND entity_id = ?3
                 ORDER BY updated_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(
                params![owner_id, ENTITY_TYPE_PROMPT, prompt_id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                    ))
                },
            )
            .map_err(|e| e.to_string())?;

        let mut out = Vec::new();
        for row in rows {
            let (id, entity_id, created_at, updated_at, rel_path) =
                row.map_err(|e| e.to_string())?;
            let stopped = Self::read_stopped_flag(db, &rel_path);
            out.push(SessionSummary {
                id,
                prompt_id: entity_id,
                created_at,
                updated_at,
                stopped,
            });
        }
        Ok(out)
    }

    pub fn read_run_config(
        db: &AppDatabase,
        session_id: &str,
        prompt_id: &str,
    ) -> Result<(String, String), String> {
        let lines = Self::read_transcript_for_run(db, session_id, prompt_id)?;
        for line in lines {
            if let TranscriptLine::RunConfig { instructions, model } = line {
                return Ok((instructions, model));
            }
        }
        Err("session run config not found".into())
    }

    /// Read transcript for an active run (caller holds DB lock; no nested auth).
    pub fn read_transcript_for_run(
        db: &AppDatabase,
        session_id: &str,
        prompt_id: &str,
    ) -> Result<Vec<TranscriptLine>, String> {
        Self::assert_session_for_prompt(db, session_id, prompt_id)?;
        let rel_path: String = db
            .conn
            .query_row(
                "SELECT transcript_path FROM sessions WHERE id = ?1 AND entity_id = ?2",
                params![session_id, prompt_id],
                |r| r.get(0),
            )
            .map_err(|_| "session not found".to_string())?;
        Self::read_transcript_lines(db, &rel_path)
    }

    pub fn get_transcript(
        db: &AppDatabase,
        session_id: &str,
        prompt_id: &str,
    ) -> Result<Vec<TranscriptLine>, String> {
        Self::require_read(db)?;
        Self::read_transcript_for_run(db, session_id, prompt_id)
    }

    pub fn chat_messages_from_transcript(lines: &[TranscriptLine]) -> Vec<(String, String)> {
        lines
            .iter()
            .filter_map(|line| {
                if let TranscriptLine::Message { role, content } = line {
                    Some((role.clone(), content.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn delete(db: &AppDatabase, session_id: &str, prompt_id: &str) -> Result<(), String> {
        Self::require_execute(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_session_for_prompt(db, session_id, prompt_id)?;

        let rel_path: String = db
            .conn
            .query_row(
                "SELECT transcript_path FROM sessions WHERE id = ?1 AND owner_id = ?2",
                params![session_id, owner_id],
                |r| r.get(0),
            )
            .map_err(|_| "session not found".to_string())?;

        let deleted = db
            .conn
            .execute(
                "DELETE FROM sessions WHERE id = ?1 AND owner_id = ?2",
                params![session_id, owner_id],
            )
            .map_err(|e| e.to_string())?;

        if deleted == 0 {
            return Err("session not found".into());
        }

        let path = Self::resolve_transcript_path(db, session_id)?;
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        if let Some(dir) = path.parent() {
            let _ = fs::remove_dir(dir);
        }

        let _ = rel_path;
        Ok(())
    }

    pub fn assert_session_for_prompt(
        db: &AppDatabase,
        session_id: &str,
        prompt_id: &str,
    ) -> Result<(), String> {
        let owner_id = Self::owner_id(db)?;
        db.conn
            .query_row(
                "SELECT 1 FROM sessions
                 WHERE id = ?1 AND owner_id = ?2 AND entity_type = ?3 AND entity_id = ?4",
                params![session_id, owner_id, ENTITY_TYPE_PROMPT, prompt_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "session not found".to_string())
    }

    fn assert_prompt_active(db: &AppDatabase, owner_id: &str, prompt_id: &str) -> Result<(), String> {
        db.conn
            .query_row(
                "SELECT 1 FROM prompts WHERE id = ?1 AND owner_id = ?2 AND lifecycle_status = 'active'",
                params![prompt_id, owner_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "prompt not found".to_string())
    }

    fn read_transcript_lines(db: &AppDatabase, rel_path: &str) -> Result<Vec<TranscriptLine>, String> {
        let path = db.data_dir.join(rel_path);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let data_root = db.data_dir.canonicalize().map_err(|e| e.to_string())?;
        let canonical = path.canonicalize().map_err(|e| e.to_string())?;
        if !canonical.starts_with(&data_root) {
            return Err("invalid transcript path".into());
        }

        let file = fs::File::open(&canonical).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut total_bytes = 0usize;

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            if line.trim().is_empty() {
                continue;
            }
            total_bytes += line.len();
            if total_bytes > MAX_TRANSCRIPT_BYTES {
                return Err("transcript too large".into());
            }
            let parsed: TranscriptLine =
                serde_json::from_str(&line).map_err(|e| format!("invalid transcript line: {e}"))?;
            lines.push(parsed);
        }
        Ok(lines)
    }

    fn read_stopped_flag(db: &AppDatabase, rel_path: &str) -> bool {
        Self::read_transcript_lines(db, rel_path)
            .ok()
            .map(|lines| {
                lines.iter().any(|line| {
                    matches!(line, TranscriptLine::Meta { event } if event == "stopped")
                })
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::PromptService;

    fn temp_db() -> AppDatabase {
        let dir = std::env::temp_dir().join(format!("velumia-session-{}", Uuid::new_v4()));
        let db = AppDatabase::open(&dir).expect("open");
        db.bootstrap_owner(None).expect("bootstrap");
        db
    }

    #[test]
    fn create_append_list_delete_session() {
        let db = temp_db();
        let prompt_id = PromptService::create(&db, "Run me", None).expect("create");

        let session = SessionService::create(&db, &prompt_id, "instr", "mock-model").expect("session");
        SessionService::append_line(
            &db,
            &session.id,
            &prompt_id,
            &TranscriptLine::Message {
                role: "user".into(),
                content: "Hi".into(),
            },
        )
        .expect("append");

        let list = SessionService::list_for_prompt(&db, &prompt_id).expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, session.id);

        let transcript = SessionService::get_transcript(&db, &session.id, &prompt_id).expect("get");
        assert!(transcript.len() >= 2);

        SessionService::delete(&db, &session.id, &prompt_id).expect("delete");
        assert!(SessionService::list_for_prompt(&db, &prompt_id).unwrap().is_empty());
        let path = db.data_dir.join(SessionService::transcript_relative_path(&session.id));
        assert!(!path.exists());
    }
}
