use crate::authz::{authorize, Permission, Principal};
use crate::db::{AppDatabase, DbError};
use crate::keychain::{self, KeychainError};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::client::{ConnectivityOutcome, LangDockClient};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    NotConfigured,
    Connected,
    ConfigurationError,
    Offline,
}

impl ConnectionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotConfigured => "not_configured",
            Self::Connected => "connected",
            Self::ConfigurationError => "configuration_error",
            Self::Offline => "offline",
        }
    }

    pub fn ui_label(&self) -> &'static str {
        match self {
            Self::NotConfigured => "Not configured",
            Self::Connected => "Connected",
            Self::ConfigurationError => "Configuration error",
            Self::Offline => "Offline",
        }
    }

    pub fn from_outcome(outcome: ConnectivityOutcome) -> Self {
        match outcome {
            ConnectivityOutcome::Connected => Self::Connected,
            ConnectivityOutcome::ConfigurationError => Self::ConfigurationError,
            ConnectivityOutcome::Offline => Self::Offline,
            ConnectivityOutcome::RateLimited => Self::Connected,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangdockProfile {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub is_default: bool,
    pub connection_status: ConnectionStatus,
    pub last_tested_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileInput {
    pub name: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub is_default: Option<bool>,
}

pub struct ProfileService;

impl ProfileService {
    pub fn list(db: &AppDatabase) -> Result<Vec<LangdockProfile>, DbError> {
        let mut stmt = db.conn.prepare(
            "SELECT id, name, base_url, is_default, connection_status, last_tested_at
             FROM langdock_profiles ORDER BY is_default DESC, name ASC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(LangdockProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    base_url: row.get(2)?,
                    is_default: row.get::<_, i64>(3)? != 0,
                    connection_status: parse_status(&row.get::<_, String>(4)?),
                    last_tested_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn save(
        db: &AppDatabase,
        principal: &Principal,
        input: ProfileInput,
        profile_id: Option<&str>,
        test_connectivity: bool,
    ) -> Result<LangdockProfile, ProfileError> {
        if !matches!(authorize(principal, Permission::CredentialWrite), crate::authz::AuthzResult::Allowed) {
            return Err(ProfileError::Authz);
        }

        let is_update = profile_id.is_some();
        let name_only = is_update
            && input.api_key.as_ref().map(|k| k.is_empty()).unwrap_or(true);

        if !is_update && input.api_key.as_ref().map(|k| k.is_empty()).unwrap_or(true) {
            return Err(ProfileError::Validation("API key is required".into()));
        }

        let id = profile_id
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let owner_id = db.solo_user_id()?;
        let base_url = normalize_base_url(input.base_url.as_deref().unwrap_or_default());
        let now = Utc::now().to_rfc3339();
        let keychain_ref = format!("langdock-profile-{id}");

        if is_update {
            if let Some(key) = input.api_key.as_ref().filter(|k| !k.is_empty()) {
                keychain::store_secret(&keychain_ref, key)?;
            } else if !keychain::secret_exists(&keychain_ref) && !name_only {
                return Err(ProfileError::Validation("API key is required".into()));
            }

            db.conn.execute(
                "UPDATE langdock_profiles SET name = ?1, base_url = ?2, updated_at = ?3 WHERE id = ?4",
                params![input.name, base_url, now, id],
            )?;
        } else {
            let api_key = input.api_key.as_ok()?.trim();
            if api_key.is_empty() {
                return Err(ProfileError::Validation("API key is required".into()));
            }
            keychain::store_secret(&keychain_ref, api_key)?;
            db.conn.execute(
                "INSERT INTO langdock_profiles (id, owner_id, name, base_url, is_default, keychain_ref, connection_status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 0, ?5, 'not_configured', ?6, ?6)",
                params![id, owner_id, input.name, base_url, keychain_ref, now],
            )?;
        }

        if input.is_default.unwrap_or(false) {
            db.conn.execute("UPDATE langdock_profiles SET is_default = 0", [])?;
            db.conn
                .execute("UPDATE langdock_profiles SET is_default = 1 WHERE id = ?1", params![id])?;
        }

        let profile = Self::get(db, &id)?.ok_or(ProfileError::NotFound)?;

        if test_connectivity && !name_only {
            let rt = tokio::runtime::Runtime::new().map_err(|e| ProfileError::Validation(e.to_string()))?;
            return rt.block_on(Self::test_profile(db, &id));
        }

        Ok(profile)
    }

    pub async fn test_profile(db: &AppDatabase, profile_id: &str) -> Result<LangdockProfile, ProfileError> {
        let row: (String, String) = db
            .conn
            .query_row(
                "SELECT base_url, keychain_ref FROM langdock_profiles WHERE id = ?1",
                params![profile_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|_| ProfileError::NotFound)?;

        let api_key = keychain::get_secret(&row.1)?;
        let client = LangDockClient::new();
        let outcome = client.probe_models(&row.0, &api_key).await;
        let status = ConnectionStatus::from_outcome(outcome);
        let now = Utc::now().to_rfc3339();

        db.conn.execute(
            "UPDATE langdock_profiles SET connection_status = ?1, last_tested_at = ?2, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now, profile_id],
        )?;

        Self::get(db, profile_id)?.ok_or(ProfileError::NotFound)
    }

    pub fn delete(db: &AppDatabase, principal: &Principal, profile_id: &str) -> Result<(), ProfileError> {
        if !matches!(authorize(principal, Permission::CredentialWrite), crate::authz::AuthzResult::Allowed) {
            return Err(ProfileError::Authz);
        }

        let keychain_ref: String = db
            .conn
            .query_row(
                "SELECT keychain_ref FROM langdock_profiles WHERE id = ?1",
                params![profile_id],
                |r| r.get(0),
            )
            .map_err(|_| ProfileError::NotFound)?;

        db.conn
            .execute("DELETE FROM langdock_profiles WHERE id = ?1", params![profile_id])?;
        let _ = keychain::delete_secret(&keychain_ref);
        Ok(())
    }

    pub fn get(db: &AppDatabase, id: &str) -> Result<Option<LangdockProfile>, DbError> {
        db.conn
            .query_row(
                "SELECT id, name, base_url, is_default, connection_status, last_tested_at
                 FROM langdock_profiles WHERE id = ?1",
                params![id],
                |row| {
                    Ok(LangdockProfile {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        base_url: row.get(2)?,
                        is_default: row.get::<_, i64>(3)? != 0,
                        connection_status: parse_status(&row.get::<_, String>(4)?),
                        last_tested_at: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(DbError::from)
    }

    pub fn default_profile(db: &AppDatabase) -> Result<Option<LangdockProfile>, DbError> {
        db.conn
            .query_row(
                "SELECT id, name, base_url, is_default, connection_status, last_tested_at
                 FROM langdock_profiles WHERE is_default = 1 LIMIT 1",
                [],
                |row| {
                    Ok(LangdockProfile {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        base_url: row.get(2)?,
                        is_default: true,
                        connection_status: parse_status(&row.get::<_, String>(4)?),
                        last_tested_at: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(DbError::from)
    }

    pub fn set_default(db: &AppDatabase, profile_id: &str) -> Result<(), DbError> {
        db.conn.execute("UPDATE langdock_profiles SET is_default = 0", [])?;
        db.conn.execute(
            "UPDATE langdock_profiles SET is_default = 1 WHERE id = ?1",
            params![profile_id],
        )?;
        Ok(())
    }

    pub fn connection_widget(db: &AppDatabase) -> Result<ConnectionWidgetState, DbError> {
        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM langdock_profiles", [], |r| r.get(0))?;

        if count == 0 {
            return Ok(ConnectionWidgetState {
                status: ConnectionStatus::NotConfigured,
                message: None,
            });
        }

        if let Some(p) = Self::default_profile(db)? {
            let message = match p.connection_status {
                ConnectionStatus::Connected => Some("Connected to LangDock.".into()),
                ConnectionStatus::ConfigurationError => {
                    Some("LangDock rejected these credentials. Check your API key and base URL.".into())
                }
                ConnectionStatus::Offline => {
                    Some("Cannot reach LangDock. Check your network or base URL.".into())
                }
                ConnectionStatus::NotConfigured => None,
            };
            return Ok(ConnectionWidgetState {
                status: p.connection_status,
                message,
            });
        }

        Ok(ConnectionWidgetState {
            status: ConnectionStatus::NotConfigured,
            message: None,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionWidgetState {
    pub status: ConnectionStatus,
    pub message: Option<String>,
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("database: {0}")]
    Db(#[from] DbError),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("keychain: {0}")]
    Keychain(#[from] KeychainError),
    #[error("validation: {0}")]
    Validation(String),
    #[error("not found")]
    NotFound,
    #[error("authorization denied")]
    Authz,
}

use thiserror::Error;

fn parse_status(s: &str) -> ConnectionStatus {
    match s {
        "connected" => ConnectionStatus::Connected,
        "configuration_error" => ConnectionStatus::ConfigurationError,
        "offline" => ConnectionStatus::Offline,
        _ => ConnectionStatus::NotConfigured,
    }
}

pub fn normalize_base_url(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "https://api.langdock.com" {
        return "https://api.langdock.com".into();
    }
    if trimmed.ends_with("/api/public") {
        return trimmed.to_string();
    }
    if trimmed.contains("api.langdock.com") {
        return trimmed.to_string();
    }
    format!("{trimmed}/api/public")
}

trait ApiKeyOk {
    fn as_ok(&self) -> Result<&str, ProfileError>;
}

impl ApiKeyOk for Option<String> {
    fn as_ok(&self) -> Result<&str, ProfileError> {
        self.as_deref()
            .ok_or_else(|| ProfileError::Validation("API key is required".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_dedicated_url() {
        assert_eq!(
            normalize_base_url("https://langdock.example.com"),
            "https://langdock.example.com/api/public"
        );
        assert_eq!(
            normalize_base_url("https://langdock.example.com/api/public"),
            "https://langdock.example.com/api/public"
        );
        assert_eq!(
            normalize_base_url("https://api.langdock.com/"),
            "https://api.langdock.com"
        );
    }
}
