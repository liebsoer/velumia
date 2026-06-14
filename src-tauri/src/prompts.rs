use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

use crate::authz::{authorize, Permission};
use crate::db::AppDatabase;
use crate::state::principal;

pub struct PromptService;

impl PromptService {
    pub fn create(db: &AppDatabase, title: &str) -> Result<String, String> {
        let p = principal(db)?;
        if !matches!(authorize(&p, Permission::PromptWrite), crate::authz::AuthzResult::Allowed) {
            return Err("permission denied".into());
        }

        let owner_id = db.solo_user_id().map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let slug = format!("prompt-{}", &id[..8]);
        let now = Utc::now().to_rfc3339();

        db.conn
            .execute(
                "INSERT INTO prompts (id, owner_id, slug, title, lifecycle_status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?5)",
                params![id, owner_id, slug, title, now],
            )
            .map_err(|e| e.to_string())?;

        Ok(id)
    }

    pub fn count_active(db: &AppDatabase) -> Result<i64, String> {
        db.conn
            .query_row(
                "SELECT COUNT(*) FROM prompts WHERE lifecycle_status = 'active'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }
}
