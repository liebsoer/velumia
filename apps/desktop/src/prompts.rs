use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::authz::{authorize, AuthzResult, Permission};
use crate::db::AppDatabase;
use crate::state::principal;

const MAX_TITLE_LEN: usize = 200;
const MAX_TAG_LEN: usize = 64;
const MAX_CONTENT_LEN: usize = 512 * 1024;
const ENTITY_TYPE_PROMPT: &str = "prompt";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSummary {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub folder_id: Option<String>,
    pub tags: Vec<TagSummary>,
    pub is_favorite: bool,
    pub content_syntax: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptFolder {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListPromptFilters {
    pub folder_id: Option<String>,
    pub tag_id: Option<String>,
    pub favorites_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVersionSummary {
    pub id: String,
    pub prompt_id: String,
    pub version_number: i64,
    pub created_at: String,
    pub is_head: bool,
}

pub struct PromptService;

impl PromptService {
    fn require_read(db: &AppDatabase) -> Result<(), String> {
        let p = principal(db)?;
        match authorize(&p, Permission::PromptRead) {
            AuthzResult::Allowed => Ok(()),
            AuthzResult::Denied { .. } => Err("permission denied".into()),
        }
    }

    fn require_write(db: &AppDatabase) -> Result<(), String> {
        let p = principal(db)?;
        match authorize(&p, Permission::PromptWrite) {
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

    fn validate_content_syntax(syntax: &str) -> Result<&str, String> {
        match syntax {
            "auto" | "plaintext" | "markdown" | "xml" | "json" => Ok(syntax),
            _ => Err("invalid content syntax".into()),
        }
    }

    fn validate_tag_name(name: &str) -> Result<&str, String> {
        let n = name.trim();
        if n.is_empty() {
            return Err("tag name is required".into());
        }
        if n.len() > MAX_TAG_LEN {
            return Err(format!("tag name must be at most {MAX_TAG_LEN} characters"));
        }
        Ok(n)
    }

    fn owner_id(db: &AppDatabase) -> Result<String, String> {
        db.solo_user_id().map_err(|e| e.to_string())
    }

    pub fn list(db: &AppDatabase, filters: ListPromptFilters) -> Result<Vec<PromptSummary>, String> {
        Self::require_read(db)?;
        let owner_id = Self::owner_id(db)?;
        let user_id = owner_id.clone();
        let favorites_only = filters.favorites_only.unwrap_or(false);

        let mut sql = String::from(
            "SELECT p.id, p.title, p.slug, p.folder_id, p.content_syntax, p.updated_at,
                    CASE WHEN f.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_favorite
             FROM prompts p
             LEFT JOIN favorites f ON f.entity_type = ?1 AND f.entity_id = p.id AND f.user_id = ?2
             WHERE p.lifecycle_status = 'active' AND p.owner_id = ?3
               AND (?4 IS NULL OR p.folder_id = ?4)",
        );

        if favorites_only {
            sql.push_str(" AND f.user_id IS NOT NULL");
        }

        sql.push_str(
            " AND (?5 IS NULL OR EXISTS (SELECT 1 FROM prompt_tags pt WHERE pt.prompt_id = p.id AND pt.tag_id = ?5))
              ORDER BY p.updated_at DESC",
        );

        let mut stmt = db.conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(
                params![
                    ENTITY_TYPE_PROMPT,
                    user_id,
                    owner_id,
                    filters.folder_id,
                    filters.tag_id,
                ],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, Option<String>>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, i64>(6)? != 0,
                    ))
                },
            )
            .map_err(|e| e.to_string())?;

        let mut prompts = Vec::new();
        for row in rows {
            let (id, title, slug, folder_id, content_syntax, updated_at, is_favorite) =
                row.map_err(|e| e.to_string())?;
            prompts.push(PromptSummary {
                id: id.clone(),
                title,
                slug,
                folder_id,
                tags: Self::tags_for_prompt(db, &id)?,
                is_favorite,
                content_syntax,
                updated_at,
            });
        }
        Ok(prompts)
    }

    pub fn get(db: &AppDatabase, prompt_id: &str) -> Result<PromptSummary, String> {
        Self::require_read(db)?;
        let owner_id = Self::owner_id(db)?;

        let row = db
            .conn
            .query_row(
                "SELECT p.id, p.title, p.slug, p.folder_id, p.content_syntax, p.updated_at,
                        CASE WHEN f.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_favorite
                 FROM prompts p
                 LEFT JOIN favorites f ON f.entity_type = ?1 AND f.entity_id = p.id AND f.user_id = ?2
                 WHERE p.id = ?3 AND p.owner_id = ?4 AND p.lifecycle_status = 'active'",
                params![ENTITY_TYPE_PROMPT, owner_id, prompt_id, owner_id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, Option<String>>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, i64>(6)? != 0,
                    ))
                },
            )
            .map_err(|_| "prompt not found".to_string())?;

        Ok(PromptSummary {
            id: row.0.clone(),
            title: row.1,
            slug: row.2,
            folder_id: row.3,
            tags: Self::tags_for_prompt(db, &row.0)?,
            is_favorite: row.6,
            content_syntax: row.4,
            updated_at: row.5,
        })
    }

    pub fn create(
        db: &AppDatabase,
        title: &str,
        folder_id: Option<&str>,
    ) -> Result<String, String> {
        Self::require_write(db)?;
        let title = Self::validate_title(title)?;
        let owner_id = Self::owner_id(db)?;

        if let Some(fid) = folder_id {
            Self::assert_folder_owned(db, &owner_id, fid)?;
        }

        let id = Uuid::new_v4().to_string();
        let slug = format!("prompt-{}", &id[..8]);
        let now = Utc::now().to_rfc3339();

        db.conn
            .execute(
                "INSERT INTO prompts (id, owner_id, folder_id, slug, title, lifecycle_status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'active', ?6, ?6)",
                params![id, owner_id, folder_id, slug, title, now],
            )
            .map_err(|e| e.to_string())?;

        Self::append_version_internal(db, &id, &slug, "")?;

        Ok(id)
    }

    pub fn update(
        db: &AppDatabase,
        prompt_id: &str,
        title: Option<&str>,
        folder_id: Option<Option<&str>>,
        content_syntax: Option<&str>,
    ) -> Result<PromptSummary, String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_prompt_owned(db, &owner_id, prompt_id)?;

        let now = Utc::now().to_rfc3339();

        if let Some(t) = title {
            let t = Self::validate_title(t)?;
            db.conn
                .execute(
                    "UPDATE prompts SET title = ?1, updated_at = ?2 WHERE id = ?3",
                    params![t, now, prompt_id],
                )
                .map_err(|e| e.to_string())?;
        }

        if let Some(fid) = folder_id {
            if let Some(f) = fid {
                Self::assert_folder_owned(db, &owner_id, f)?;
            }
            db.conn
                .execute(
                    "UPDATE prompts SET folder_id = ?1, updated_at = ?2 WHERE id = ?3",
                    params![fid, now, prompt_id],
                )
                .map_err(|e| e.to_string())?;
        }

        if let Some(syntax) = content_syntax {
            let syntax = Self::validate_content_syntax(syntax)?;
            db.conn
                .execute(
                    "UPDATE prompts SET content_syntax = ?1, updated_at = ?2 WHERE id = ?3",
                    params![syntax, now, prompt_id],
                )
                .map_err(|e| e.to_string())?;
        }

        Self::get(db, prompt_id)
    }

    pub fn trash(db: &AppDatabase, prompt_id: &str) -> Result<(), String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_prompt_owned(db, &owner_id, prompt_id)?;
        let now = Utc::now().to_rfc3339();

        let updated = db
            .conn
            .execute(
                "UPDATE prompts SET lifecycle_status = 'trashed', trashed_at = ?1, updated_at = ?1
                 WHERE id = ?2 AND owner_id = ?3",
                params![now, prompt_id, owner_id],
            )
            .map_err(|e| e.to_string())?;

        if updated == 0 {
            return Err("prompt not found".into());
        }
        Ok(())
    }

    pub fn create_folder(
        db: &AppDatabase,
        title: &str,
        parent_id: Option<&str>,
    ) -> Result<PromptFolder, String> {
        Self::require_write(db)?;
        let title = Self::validate_title(title)?;
        let owner_id = Self::owner_id(db)?;

        if let Some(pid) = parent_id {
            let grandparent: Option<String> = db
                .conn
                .query_row(
                    "SELECT parent_id FROM prompt_folders WHERE id = ?1 AND owner_id = ?2",
                    params![pid, owner_id],
                    |r| r.get(0),
                )
                .map_err(|_| "parent folder not found".to_string())?;

            if grandparent.is_some() {
                return Err("folder nesting limited to two levels".into());
            }
        }

        let id = Uuid::new_v4().to_string();
        let slug = format!("folder-{}", &id[..8]);
        let now = Utc::now().to_rfc3339();

        db.conn
            .execute(
                "INSERT INTO prompt_folders (id, owner_id, parent_id, slug, title, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                params![id, owner_id, parent_id, slug, title, now],
            )
            .map_err(|e| e.to_string())?;

        Ok(PromptFolder {
            id,
            title: title.to_string(),
            slug,
            parent_id: parent_id.map(String::from),
        })
    }

    pub fn list_folders(db: &AppDatabase) -> Result<Vec<PromptFolder>, String> {
        Self::require_read(db)?;
        let owner_id = Self::owner_id(db)?;

        let mut stmt = db
            .conn
            .prepare(
                "SELECT id, title, slug, parent_id FROM prompt_folders
                 WHERE owner_id = ?1 ORDER BY title ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![owner_id], |r| {
                Ok(PromptFolder {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    slug: r.get(2)?,
                    parent_id: r.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn move_to_folder(
        db: &AppDatabase,
        prompt_id: &str,
        folder_id: Option<&str>,
    ) -> Result<PromptSummary, String> {
        Self::update(db, prompt_id, None, Some(folder_id), None)
    }

    pub fn list_tags(db: &AppDatabase) -> Result<Vec<TagSummary>, String> {
        Self::require_read(db)?;
        let owner_id = Self::owner_id(db)?;

        let mut stmt = db
            .conn
            .prepare("SELECT id, name FROM tags WHERE owner_id = ?1 ORDER BY name ASC")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![owner_id], |r| {
                Ok(TagSummary {
                    id: r.get(0)?,
                    name: r.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    fn ensure_tag(db: &AppDatabase, owner_id: &str, name: &str) -> Result<String, String> {
        let name = Self::validate_tag_name(name)?;

        if let Ok(id) = db.conn.query_row(
            "SELECT id FROM tags WHERE owner_id = ?1 AND name = ?2",
            params![owner_id, name],
            |r| r.get::<_, String>(0),
        ) {
            return Ok(id);
        }

        let id = Uuid::new_v4().to_string();
        db.conn
            .execute(
                "INSERT INTO tags (id, owner_id, name) VALUES (?1, ?2, ?3)",
                params![id, owner_id, name],
            )
            .map_err(|e| e.to_string())?;
        Ok(id)
    }

    pub fn set_tags(db: &AppDatabase, prompt_id: &str, tag_names: &[String]) -> Result<PromptSummary, String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_prompt_owned(db, &owner_id, prompt_id)?;

        let tx = db.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "DELETE FROM prompt_tags WHERE prompt_id = ?1",
            params![prompt_id],
        )
        .map_err(|e| e.to_string())?;

        for name in tag_names {
            let tag_id = Self::ensure_tag(db, &owner_id, name)?;
            tx.execute(
                "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?1, ?2)",
                params![prompt_id, tag_id],
            )
            .map_err(|e| e.to_string())?;
        }

        tx.commit().map_err(|e| e.to_string())?;
        Self::get(db, prompt_id)
    }

    pub fn add_tag(db: &AppDatabase, prompt_id: &str, tag_name: &str) -> Result<PromptSummary, String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_prompt_owned(db, &owner_id, prompt_id)?;
        let tag_id = Self::ensure_tag(db, &owner_id, tag_name)?;

        db.conn
            .execute(
                "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?1, ?2)",
                params![prompt_id, tag_id],
            )
            .map_err(|e| e.to_string())?;

        Self::get(db, prompt_id)
    }

    pub fn remove_tag(db: &AppDatabase, prompt_id: &str, tag_id: &str) -> Result<PromptSummary, String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_prompt_owned(db, &owner_id, prompt_id)?;

        db.conn
            .execute(
                "DELETE FROM prompt_tags WHERE prompt_id = ?1 AND tag_id = ?2",
                params![prompt_id, tag_id],
            )
            .map_err(|e| e.to_string())?;

        Self::get(db, prompt_id)
    }

    pub fn set_favorite(db: &AppDatabase, prompt_id: &str) -> Result<(), String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_prompt_owned(db, &owner_id, prompt_id)?;
        let now = Utc::now().to_rfc3339();

        db.conn
            .execute(
                "INSERT OR IGNORE INTO favorites (user_id, entity_type, entity_id, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![owner_id, ENTITY_TYPE_PROMPT, prompt_id, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn unset_favorite(db: &AppDatabase, prompt_id: &str) -> Result<(), String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;

        db.conn
            .execute(
                "DELETE FROM favorites WHERE user_id = ?1 AND entity_type = ?2 AND entity_id = ?3",
                params![owner_id, ENTITY_TYPE_PROMPT, prompt_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
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

    fn tags_for_prompt(db: &AppDatabase, prompt_id: &str) -> Result<Vec<TagSummary>, String> {
        let mut stmt = db
            .conn
            .prepare(
                "SELECT t.id, t.name FROM tags t
                 INNER JOIN prompt_tags pt ON pt.tag_id = t.id
                 WHERE pt.prompt_id = ?1 ORDER BY t.name ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![prompt_id], |r| {
                Ok(TagSummary {
                    id: r.get(0)?,
                    name: r.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    fn assert_prompt_owned(db: &AppDatabase, owner_id: &str, prompt_id: &str) -> Result<(), String> {
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

    fn assert_folder_owned(db: &AppDatabase, owner_id: &str, folder_id: &str) -> Result<(), String> {
        db.conn
            .query_row(
                "SELECT 1 FROM prompt_folders WHERE id = ?1 AND owner_id = ?2",
                params![folder_id, owner_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "folder not found".to_string())
    }

    fn validate_content(content: &str) -> Result<(), String> {
        if content.len() > MAX_CONTENT_LEN {
            return Err(format!(
                "content must be at most {MAX_CONTENT_LEN} characters"
            ));
        }
        Ok(())
    }

    fn relative_content_path(slug: &str, version_number: i64) -> String {
        format!("prompts/{slug}/versions/{version_number:04}.md")
    }

    fn write_content_file(db: &AppDatabase, relative_path: &str, content: &str) -> Result<(), String> {
        let path = db.data_dir.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn read_content_file(db: &AppDatabase, relative_path: &str) -> Result<String, String> {
        let path = db.data_dir.join(relative_path);
        fs::read_to_string(&path).map_err(|_| "version content not found".to_string())
    }

    fn prompt_slug_and_head(
        db: &AppDatabase,
        owner_id: &str,
        prompt_id: &str,
    ) -> Result<(String, Option<String>), String> {
        db.conn
            .query_row(
                "SELECT slug, head_version_id FROM prompts
                 WHERE id = ?1 AND owner_id = ?2 AND lifecycle_status = 'active'",
                params![prompt_id, owner_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|_| "prompt not found".to_string())
    }

    fn next_version_number(db: &AppDatabase, prompt_id: &str) -> Result<i64, String> {
        let max: Option<i64> = db
            .conn
            .query_row(
                "SELECT MAX(version_number) FROM prompt_versions WHERE prompt_id = ?1",
                params![prompt_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();
        Ok(max.unwrap_or(0) + 1)
    }

    fn version_summary_by_id(
        db: &AppDatabase,
        prompt_id: &str,
        version_id: &str,
    ) -> Result<PromptVersionSummary, String> {
        db.conn
            .query_row(
                "SELECT pv.id, pv.prompt_id, pv.version_number, pv.created_at,
                        CASE WHEN pv.id = p.head_version_id THEN 1 ELSE 0 END
                 FROM prompt_versions pv
                 INNER JOIN prompts p ON p.id = pv.prompt_id
                 WHERE pv.id = ?1 AND pv.prompt_id = ?2",
                params![version_id, prompt_id],
                |r| {
                    Ok(PromptVersionSummary {
                        id: r.get(0)?,
                        prompt_id: r.get(1)?,
                        version_number: r.get(2)?,
                        created_at: r.get(3)?,
                        is_head: r.get::<_, i64>(4)? != 0,
                    })
                },
            )
            .map_err(|_| "version not found".to_string())
    }

    fn append_version_internal(
        db: &AppDatabase,
        prompt_id: &str,
        slug: &str,
        content: &str,
    ) -> Result<PromptVersionSummary, String> {
        Self::validate_content(content)?;
        let version_number = Self::next_version_number(db, prompt_id)?;
        let relative_path = Self::relative_content_path(slug, version_number);
        Self::write_content_file(db, &relative_path, content)?;

        let version_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let tx = db.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO prompt_versions (id, prompt_id, version_number, content_path, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![version_id, prompt_id, version_number, relative_path, now],
        )
        .map_err(|e| e.to_string())?;

        tx.execute(
            "UPDATE prompts SET head_version_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![version_id, now, prompt_id],
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;

        Ok(PromptVersionSummary {
            id: version_id,
            prompt_id: prompt_id.to_string(),
            version_number,
            created_at: now,
            is_head: true,
        })
    }

    fn head_content(db: &AppDatabase, prompt_id: &str) -> Result<Option<String>, String> {
        let head_id: Option<String> = db
            .conn
            .query_row(
                "SELECT head_version_id FROM prompts WHERE id = ?1",
                params![prompt_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        let Some(hid) = head_id else {
            return Ok(None);
        };

        let path: String = db
            .conn
            .query_row(
                "SELECT content_path FROM prompt_versions WHERE id = ?1",
                params![hid],
                |r| r.get(0),
            )
            .map_err(|_| "version not found".to_string())?;

        Ok(Some(Self::read_content_file(db, &path)?))
    }

    pub fn save_prompt_content(
        db: &AppDatabase,
        prompt_id: &str,
        content: &str,
    ) -> Result<PromptVersionSummary, String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        let (slug, head_id) = Self::prompt_slug_and_head(db, &owner_id, prompt_id)?;

        if let Some(ref hid) = head_id {
            let head_body = Self::head_content(db, prompt_id)?.unwrap_or_default();
            if head_body == content {
                return Self::version_summary_by_id(db, prompt_id, hid);
            }
        }

        Self::append_version_internal(db, prompt_id, &slug, content)
    }

    pub fn list_prompt_versions(
        db: &AppDatabase,
        prompt_id: &str,
    ) -> Result<Vec<PromptVersionSummary>, String> {
        Self::require_read(db)?;
        let owner_id = Self::owner_id(db)?;
        Self::assert_prompt_owned(db, &owner_id, prompt_id)?;

        let mut stmt = db
            .conn
            .prepare(
                "SELECT pv.id, pv.prompt_id, pv.version_number, pv.created_at,
                        CASE WHEN pv.id = p.head_version_id THEN 1 ELSE 0 END
                 FROM prompt_versions pv
                 INNER JOIN prompts p ON p.id = pv.prompt_id
                 WHERE pv.prompt_id = ?1 AND p.owner_id = ?2
                 ORDER BY pv.version_number DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![prompt_id, owner_id], |r| {
                Ok(PromptVersionSummary {
                    id: r.get(0)?,
                    prompt_id: r.get(1)?,
                    version_number: r.get(2)?,
                    created_at: r.get(3)?,
                    is_head: r.get::<_, i64>(4)? != 0,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn get_prompt_version_content(
        db: &AppDatabase,
        version_id: &str,
    ) -> Result<String, String> {
        Self::require_read(db)?;
        let owner_id = Self::owner_id(db)?;

        let path: String = db
            .conn
            .query_row(
                "SELECT pv.content_path FROM prompt_versions pv
                 INNER JOIN prompts p ON p.id = pv.prompt_id
                 WHERE pv.id = ?1 AND p.owner_id = ?2 AND p.lifecycle_status = 'active'",
                params![version_id, owner_id],
                |r| r.get(0),
            )
            .map_err(|_| "version not found".to_string())?;

        Self::read_content_file(db, &path)
    }

    /// Frozen head instructions for prompt runs (active prompts only).
    pub fn head_content_for_run(db: &AppDatabase, prompt_id: &str) -> Result<String, String> {
        let owner_id = Self::owner_id(db)?;
        db.conn
            .query_row(
                "SELECT 1 FROM prompts WHERE id = ?1 AND owner_id = ?2 AND lifecycle_status = 'active'",
                params![prompt_id, owner_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "prompt not found".to_string())?;

        Self::head_content(db, prompt_id)?
            .ok_or_else(|| "prompt has no content".to_string())
    }

    pub fn restore_prompt_version(
        db: &AppDatabase,
        prompt_id: &str,
        version_id: &str,
    ) -> Result<PromptVersionSummary, String> {
        Self::require_write(db)?;
        let owner_id = Self::owner_id(db)?;
        let (slug, _) = Self::prompt_slug_and_head(db, &owner_id, prompt_id)?;

        let (scoped_prompt_id, content_path): (String, String) = db
            .conn
            .query_row(
                "SELECT pv.prompt_id, pv.content_path FROM prompt_versions pv
                 INNER JOIN prompts p ON p.id = pv.prompt_id
                 WHERE pv.id = ?1 AND p.owner_id = ?2",
                params![version_id, owner_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|_| "version not found".to_string())?;

        if scoped_prompt_id != prompt_id {
            return Err("version not found".into());
        }

        let body = Self::read_content_file(db, &content_path)?;
        if let Some(head_body) = Self::head_content(db, prompt_id)? {
            if head_body == body {
                return Err("already current version".into());
            }
        }

        Self::append_version_internal(db, prompt_id, &slug, &body)
    }
}
