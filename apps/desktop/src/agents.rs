use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

use crate::authz::{authorize, AuthzResult, Permission};
use crate::db::AppDatabase;
use crate::prompt_runs::{DEFAULT_MODEL_FALLBACK, DEFAULT_MODEL_SETTING};
use crate::state::principal;

const MAX_TITLE_LEN: usize = 200;
const MAX_INSTRUCTIONS_LEN: usize = 512 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    pub model: String,
    #[serde(default)]
    pub web_search: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub lifecycle_status: String,
    pub model: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPromptAttachment {
    pub prompt_id: String,
    pub title: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkillAttachment {
    pub skill_id: String,
    pub title: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSubagentRef {
    pub agent_id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetail {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub lifecycle_status: String,
    pub model: String,
    pub web_search: bool,
    pub updated_at: String,
    pub instructions: String,
    pub prompts: Vec<AgentPromptAttachment>,
    pub skills: Vec<AgentSkillAttachment>,
    pub subagents: Vec<AgentSubagentRef>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListAgentFilters {
    pub lifecycle_filter: Option<String>,
}

pub struct AgentService;

impl AgentService {
    fn require_read(db: &AppDatabase) -> Result<(), String> {
        let p = principal(db)?;
        match authorize(&p, Permission::AgentRead) {
            AuthzResult::Allowed => Ok(()),
            AuthzResult::Denied { .. } => Err("permission denied".into()),
        }
    }

    fn require_write(db: &AppDatabase) -> Result<(), String> {
        let p = principal(db)?;
        match authorize(&p, Permission::AgentWrite) {
            AuthzResult::Allowed => Ok(()),
            AuthzResult::Denied { .. } => Err("permission denied".into()),
        }
    }

    fn validate_title(title: &str) -> Result<&str, String> {
        let t = title.trim();
        if t.is_empty() {
            return Err("title is required".into());
        }
        if t.len() > MAX_TITLE_LEN {
            return Err(format!("title must be at most {MAX_TITLE_LEN} characters"));
        }
        Ok(t)
    }

    fn validate_instructions(instructions: &str) -> Result<(), String> {
        if instructions.len() > MAX_INSTRUCTIONS_LEN {
            return Err(format!(
                "instructions must be at most {MAX_INSTRUCTIONS_LEN} characters"
            ));
        }
        Ok(())
    }

    fn owner_id(db: &AppDatabase) -> Result<String, String> {
        db.solo_user_id().map_err(|e| e.to_string())
    }

    fn lifecycle_filter_value(filters: &ListAgentFilters) -> Result<&'static str, String> {
        match filters.lifecycle_filter.as_deref().unwrap_or("active") {
            "active" => Ok("active"),
            "archived" => Ok("archived"),
            "trashed" => Ok("trashed"),
            _ => Err("invalid lifecycle filter".into()),
        }
    }

    fn default_model(db: &AppDatabase) -> String {
        db.get_setting(DEFAULT_MODEL_SETTING)
            .ok()
            .flatten()
            .and_then(|json| {
                serde_json::from_str::<String>(&json)
                    .ok()
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_else(|| DEFAULT_MODEL_FALLBACK.to_string())
    }

    fn parse_config(json: &str) -> AgentConfig {
        serde_json::from_str(json).unwrap_or_default()
    }

    fn config_to_json(config: &AgentConfig) -> Result<String, String> {
        serde_json::to_string(config).map_err(|e| e.to_string())
    }

    fn instructions_relative_path(slug: &str) -> String {
        format!("agents/{slug}/instructions.md")
    }

    fn resolve_instructions_path(db: &AppDatabase, slug: &str) -> PathBuf {
        db.data_dir.join(Self::instructions_relative_path(slug))
    }

    fn assert_safe_relative_path(relative: &str) -> Result<(), String> {
        let path = Path::new(relative);
        if path.is_absolute() {
            return Err("invalid instructions path".into());
        }
        for component in path.components() {
            if matches!(component, Component::ParentDir) {
                return Err("invalid instructions path".into());
            }
        }
        Ok(())
    }

    fn write_instructions(db: &AppDatabase, slug: &str, body: &str) -> Result<(), String> {
        Self::validate_instructions(body)?;
        let rel = Self::instructions_relative_path(slug);
        Self::assert_safe_relative_path(&rel)?;
        let path = Self::resolve_instructions_path(db, slug);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| "failed to write instructions".to_string())?;
        }
        fs::write(&path, body).map_err(|_| "failed to write instructions".to_string())
    }

    fn read_instructions(db: &AppDatabase, slug: &str) -> Result<String, String> {
        let rel = Self::instructions_relative_path(slug);
        Self::assert_safe_relative_path(&rel)?;
        let path = Self::resolve_instructions_path(db, slug);
        if !path.is_file() {
            return Ok(String::new());
        }
        fs::read_to_string(&path).map_err(|_| "failed to read instructions".to_string())
    }

    fn assert_agent_owned(db: &AppDatabase, owner_id: &str, agent_id: &str) -> Result<String, String> {
        db.conn
            .query_row(
                "SELECT slug FROM agents WHERE id = ?1 AND owner_id = ?2",
                params![agent_id, owner_id],
                |r| r.get(0),
            )
            .map_err(|_| "agent not found".to_string())
    }

    fn map_summary(
        id: String,
        title: String,
        slug: String,
        lifecycle_status: String,
        config_json: String,
        updated_at: String,
    ) -> AgentSummary {
        let config = Self::parse_config(&config_json);
        AgentSummary {
            id,
            title,
            slug,
            lifecycle_status,
            model: config.model,
            updated_at,
        }
    }

    pub fn list(db: &AppDatabase, filters: ListAgentFilters) -> Result<Vec<AgentSummary>, String> {
        Self::require_read(db)?;
        let owner_id = Self::owner_id(db)?;
        let lifecycle = Self::lifecycle_filter_value(&filters)?;

        let mut stmt = db
            .conn
            .prepare(&format!(
                "SELECT id, title, slug, lifecycle_status, config_json, updated_at
                 FROM agents
                 WHERE owner_id = ?1 AND lifecycle_status = '{lifecycle}'
                 ORDER BY updated_at DESC"
            ))
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![owner_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut agents = Vec::new();
        for row in rows {
            let (id, title, slug, lifecycle_status, config_json, updated_at) =
                row.map_err(|e| e.to_string())?;
            agents.push(Self::map_summary(
                id,
                title,
                slug,
                lifecycle_status,
                config_json,
                updated_at,
            ));
        }
        Ok(agents)
    }

    pub fn get(db: &AppDatabase, agent_id: &str) -> Result<AgentDetail, String> {
        Self::require_read(db)?;
        let owner_id = Self::owner_id(db)?;

        let row = db
            .conn
            .query_row(
                "SELECT id, title, slug, lifecycle_status, config_json, updated_at
                 FROM agents WHERE id = ?1 AND owner_id = ?2",
                params![agent_id, owner_id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                    ))
                },
            )
            .map_err(|_| "agent not found".to_string())?;

        let config = Self::parse_config(&row.4);
        let instructions = Self::read_instructions(db, &row.2)?;

        Ok(AgentDetail {
            id: row.0.clone(),
            title: row.1,
            slug: row.2,
            lifecycle_status: row.3,
            model: config.model,
            web_search: config.web_search,
            updated_at: row.5,
            instructions,
            prompts: Self::prompt_attachments(db, &row.0)?,
            skills: Self::skill_attachments(db, &row.0)?,
            subagents: Self::subagent_refs(db, &row.0)?,
        })
    }

    pub fn create(db: &AppDatabase, title: &str) -> Result<String, String> {
        Self::require_write(db)?;
        let title = Self::validate_title(title)?;
        let owner_id = Self::owner_id(db)?;

        let id = Uuid::new_v4().to_string();
        let slug = format!("agent-{}", &id[..8]);
        let now = Utc::now().to_rfc3339();
        let config = AgentConfig {
            model: Self::default_model(db),
            web_search: false,
        };
        let config_json = Self::config_to_json(&config)?;

        db.conn
            .execute(
                "INSERT INTO agents (id, owner_id, slug, title, lifecycle_status, config_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6, ?6)",
                params![id, owner_id, slug, title, config_json, now],
            )
            .map_err(|e| e.to_string())?;

        Self::write_instructions(db, &slug, "")?;

        Ok(id)
    }

    pub fn update(
        db: &AppDatabase,
        agent_id: &str,
        title: Option<&str>,
        instructions: Option<&str>,
        model: Option<&str>,
        web_search: Option<bool>,
    ) -> Result<AgentDetail, String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        let slug = Self::assert_agent_owned(db, &owner_id, agent_id)?;
        let now = Utc::now().to_rfc3339();

        if let Some(t) = title {
            let t = Self::validate_title(t)?;
            db.conn
                .execute(
                    "UPDATE agents SET title = ?1, updated_at = ?2 WHERE id = ?3",
                    params![t, now, agent_id],
                )
                .map_err(|e| e.to_string())?;
        }

        if let Some(body) = instructions {
            Self::write_instructions(db, &slug, body)?;
            db.conn
                .execute(
                    "UPDATE agents SET updated_at = ?1 WHERE id = ?2",
                    params![now, agent_id],
                )
                .map_err(|e| e.to_string())?;
        }

        if model.is_some() || web_search.is_some() {
            let current_json: String = db
                .conn
                .query_row(
                    "SELECT config_json FROM agents WHERE id = ?1",
                    params![agent_id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            let mut config = Self::parse_config(&current_json);
            if let Some(m) = model {
                let m = m.trim();
                if m.is_empty() {
                    return Err("model is required".into());
                }
                config.model = m.to_string();
            }
            if let Some(ws) = web_search {
                config.web_search = ws;
            }
            let config_json = Self::config_to_json(&config)?;
            db.conn
                .execute(
                    "UPDATE agents SET config_json = ?1, updated_at = ?2 WHERE id = ?3",
                    params![config_json, now, agent_id],
                )
                .map_err(|e| e.to_string())?;
        }

        Self::get(db, agent_id)
    }

    fn prompt_attachments(
        db: &AppDatabase,
        agent_id: &str,
    ) -> Result<Vec<AgentPromptAttachment>, String> {
        let mut stmt = db
            .conn
            .prepare(
                "SELECT ap.prompt_id, p.title, ap.sort_order
                 FROM agent_prompts ap
                 JOIN prompts p ON p.id = ap.prompt_id
                 WHERE ap.agent_id = ?1
                 ORDER BY ap.sort_order ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![agent_id], |r| {
                Ok(AgentPromptAttachment {
                    prompt_id: r.get(0)?,
                    title: r.get(1)?,
                    sort_order: r.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    fn skill_attachments(
        db: &AppDatabase,
        agent_id: &str,
    ) -> Result<Vec<AgentSkillAttachment>, String> {
        let mut stmt = db
            .conn
            .prepare(
                "SELECT ask.skill_id, COALESCE(s.title, ask.skill_id), ask.sort_order
                 FROM agent_skills ask
                 LEFT JOIN skills s ON s.id = ask.skill_id
                 WHERE ask.agent_id = ?1
                 ORDER BY ask.sort_order ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![agent_id], |r| {
                Ok(AgentSkillAttachment {
                    skill_id: r.get(0)?,
                    title: r.get(1)?,
                    sort_order: r.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    fn subagent_refs(db: &AppDatabase, agent_id: &str) -> Result<Vec<AgentSubagentRef>, String> {
        let mut stmt = db
            .conn
            .prepare(
                "SELECT a.id, a.title
                 FROM agent_subagents sub
                 JOIN agents a ON a.id = sub.child_agent_id
                 WHERE sub.parent_agent_id = ?1
                 ORDER BY a.title ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![agent_id], |r| {
                Ok(AgentSubagentRef {
                    agent_id: r.get(0)?,
                    title: r.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    fn assert_active_prompt_owned(
        db: &AppDatabase,
        owner_id: &str,
        prompt_id: &str,
    ) -> Result<(), String> {
        let status: String = db
            .conn
            .query_row(
                "SELECT lifecycle_status FROM prompts WHERE id = ?1 AND owner_id = ?2",
                params![prompt_id, owner_id],
                |r| r.get(0),
            )
            .map_err(|_| "prompt not found".to_string())?;
        if status != "active" {
            return Err("prompt is not active".into());
        }
        Ok(())
    }

    fn assert_skill_owned(db: &AppDatabase, owner_id: &str, skill_id: &str) -> Result<(), String> {
        let exists: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM skills WHERE id = ?1 AND owner_id = ?2 AND lifecycle_status = 'active'",
                params![skill_id, owner_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists == 0 {
            return Err("skill not found".into());
        }
        Ok(())
    }

    fn assert_subagent_linkable(
        db: &AppDatabase,
        owner_id: &str,
        parent_id: &str,
        child_id: &str,
    ) -> Result<(), String> {
        if parent_id == child_id {
            return Err("agent cannot be its own sub-agent".into());
        }

        Self::assert_agent_owned(db, owner_id, child_id)?;

        let child_status: String = db
            .conn
            .query_row(
                "SELECT lifecycle_status FROM agents WHERE id = ?1",
                params![child_id],
                |r| r.get(0),
            )
            .map_err(|_| "agent not found".to_string())?;
        if child_status != "active" {
            return Err("sub-agent must be active".into());
        }

        let child_is_parent: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM agent_subagents WHERE parent_agent_id = ?1",
                params![child_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if child_is_parent > 0 {
            return Err("sub-agent cannot have its own sub-agents".into());
        }

        let parent_is_child: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM agent_subagents WHERE child_agent_id = ?1",
                params![parent_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if parent_is_child > 0 {
            return Err("agent that is a sub-agent cannot have sub-agents".into());
        }

        let child_has_other_parent: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM agent_subagents WHERE child_agent_id = ?1 AND parent_agent_id != ?2",
                params![child_id, parent_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if child_has_other_parent > 0 {
            return Err("sub-agent is already linked to another agent".into());
        }

        Ok(())
    }

    pub fn set_prompts(
        db: &AppDatabase,
        agent_id: &str,
        prompt_ids: &[String],
    ) -> Result<AgentDetail, String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_agent_owned(db, &owner_id, agent_id)?;

        for pid in prompt_ids {
            Self::assert_active_prompt_owned(db, &owner_id, pid)?;
        }

        let tx = db.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "DELETE FROM agent_prompts WHERE agent_id = ?1",
            params![agent_id],
        )
        .map_err(|e| e.to_string())?;
        for (i, pid) in prompt_ids.iter().enumerate() {
            tx.execute(
                "INSERT INTO agent_prompts (agent_id, prompt_id, sort_order) VALUES (?1, ?2, ?3)",
                params![agent_id, pid, i as i64],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;

        let now = Utc::now().to_rfc3339();
        db.conn
            .execute(
                "UPDATE agents SET updated_at = ?1 WHERE id = ?2",
                params![now, agent_id],
            )
            .map_err(|e| e.to_string())?;

        Self::get(db, agent_id)
    }

    pub fn set_skills(
        db: &AppDatabase,
        agent_id: &str,
        skill_ids: &[String],
    ) -> Result<AgentDetail, String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_agent_owned(db, &owner_id, agent_id)?;

        for sid in skill_ids {
            Self::assert_skill_owned(db, &owner_id, sid)?;
        }

        let tx = db.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "DELETE FROM agent_skills WHERE agent_id = ?1",
            params![agent_id],
        )
        .map_err(|e| e.to_string())?;
        for (i, sid) in skill_ids.iter().enumerate() {
            tx.execute(
                "INSERT INTO agent_skills (agent_id, skill_id, sort_order) VALUES (?1, ?2, ?3)",
                params![agent_id, sid, i as i64],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;

        let now = Utc::now().to_rfc3339();
        db.conn
            .execute(
                "UPDATE agents SET updated_at = ?1 WHERE id = ?2",
                params![now, agent_id],
            )
            .map_err(|e| e.to_string())?;

        Self::get(db, agent_id)
    }

    pub fn set_subagents(
        db: &AppDatabase,
        agent_id: &str,
        child_agent_ids: &[String],
    ) -> Result<AgentDetail, String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_agent_owned(db, &owner_id, agent_id)?;

        for child_id in child_agent_ids {
            Self::assert_subagent_linkable(db, &owner_id, agent_id, child_id)?;
        }

        let tx = db.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "DELETE FROM agent_subagents WHERE parent_agent_id = ?1",
            params![agent_id],
        )
        .map_err(|e| e.to_string())?;
        for child_id in child_agent_ids {
            tx.execute(
                "INSERT INTO agent_subagents (parent_agent_id, child_agent_id) VALUES (?1, ?2)",
                params![agent_id, child_id],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;

        let now = Utc::now().to_rfc3339();
        db.conn
            .execute(
                "UPDATE agents SET updated_at = ?1 WHERE id = ?2",
                params![now, agent_id],
            )
            .map_err(|e| e.to_string())?;

        Self::get(db, agent_id)
    }

    pub fn count_active(db: &AppDatabase) -> Result<i64, String> {
        Self::require_read(db)?;
        let owner_id = Self::owner_id(db)?;
        db.conn
            .query_row(
                "SELECT COUNT(*) FROM agents WHERE owner_id = ?1 AND lifecycle_status = 'active'",
                params![owner_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }
}
