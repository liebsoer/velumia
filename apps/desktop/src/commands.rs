use crate::authz::{authorize, AuthzResult, Permission};
use crate::langdock::{LangdockProfile, ProfileInput, ProfileService};
use crate::prompt_runs::{
    PromptRunService, SendPromptMessageInput, StartPromptRunInput, StartPromptRunResult,
    StopPromptRunInput,
};
use crate::prompts::{
    ListPromptFilters, PromptFolder, PromptService, PromptSummary, PromptVersionSummary,
    TagSummary,
};
use crate::sessions::{SessionService, SessionSummary, TranscriptLine};
use crate::state::{principal, AppState};
use serde::Deserialize;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn ping() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(Debug, Deserialize)]
pub struct BootstrapInput {
    pub display_name: Option<String>,
}

#[tauri::command]
pub fn bootstrap_owner(state: State<'_, Arc<AppState>>, input: BootstrapInput) -> Result<String, String> {
    state.with_db(|db| {
        db.bootstrap_owner(input.display_name.as_deref())
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn is_first_launch(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    state.with_db(|db| db.is_first_launch().map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn complete_wizard(state: State<'_, Arc<AppState>>, skipped: bool) -> Result<(), String> {
    state.with_db(|db| {
        if !skipped {
            db.bootstrap_owner(None).map_err(|e| e.to_string())?;
        } else {
            let _ = db.bootstrap_owner(None);
        }
        db.set_setting("wizard_completed", "true")
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn get_connection_widget(state: State<'_, Arc<AppState>>) -> Result<crate::langdock::ConnectionWidgetState, String> {
    state.with_db(|db| {
        ProfileService::connection_widget(db).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn list_langdock_profiles(state: State<'_, Arc<AppState>>) -> Result<Vec<LangdockProfile>, String> {
    state.with_db(|db| ProfileService::list(db).map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn save_langdock_profile(
    state: State<'_, Arc<AppState>>,
    input: ProfileInput,
    profile_id: Option<String>,
    test_connectivity: Option<bool>,
) -> Result<LangdockProfile, String> {
    state.with_db(|db| {
        ProfileService::save(
            db,
            &principal(db)?,
            input,
            profile_id.as_deref(),
            test_connectivity.unwrap_or(true),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn test_langdock_connection(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
) -> Result<LangdockProfile, String> {
    let (base_url, keychain_ref) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.conn
            .query_row(
                "SELECT base_url, keychain_ref FROM langdock_profiles WHERE id = ?1",
                rusqlite::params![profile_id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .map_err(|_| "profile not found".to_string())?
    };

    let api_key = crate::keychain::get_secret(&keychain_ref).map_err(|e| e.to_string())?;
    let client = crate::langdock::LangDockClient::new();
    let outcome = client.probe_models(&base_url, &api_key).await;
    let status = crate::langdock::ConnectionStatus::from_outcome(outcome);
    let now = chrono::Utc::now().to_rfc3339();

    state.with_db(|db| {
        db.conn
            .execute(
                "UPDATE langdock_profiles SET connection_status = ?1, last_tested_at = ?2, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![status.as_str(), now, profile_id],
            )
            .map_err(|e| e.to_string())?;
        ProfileService::get(db, &profile_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "profile not found".to_string())
    })
}

#[tauri::command]
pub fn set_default_langdock_profile(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
) -> Result<LangdockProfile, String> {
    state.with_db(|db| {
        ProfileService::set_default(db, &profile_id).map_err(|e| e.to_string())?;
        ProfileService::get(db, &profile_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "profile not found".to_string())
    })
}

#[tauri::command]
pub fn delete_langdock_profile(state: State<'_, Arc<AppState>>, profile_id: String) -> Result<(), String> {
    state.with_db(|db| {
        ProfileService::delete(db, &principal(db)?, &profile_id).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn check_authorize(state: State<'_, Arc<AppState>>, action: String) -> Result<AuthzResult, String> {
    state.with_db(|db| {
        let p = principal(db)?;
        let permission = Permission::from_action(&action).ok_or_else(|| "unknown permission".to_string())?;
        Ok(authorize(&p, permission))
    })
}

#[tauri::command]
pub fn list_prompts(
    state: State<'_, Arc<AppState>>,
    folder_id: Option<String>,
    tag_id: Option<String>,
    favorites_only: Option<bool>,
    lifecycle_filter: Option<String>,
) -> Result<Vec<PromptSummary>, String> {
    state.with_db(|db| {
        PromptService::list(
            db,
            ListPromptFilters {
                folder_id,
                tag_id,
                favorites_only,
                lifecycle_filter,
            },
        )
    })
}

#[tauri::command]
pub fn get_prompt(state: State<'_, Arc<AppState>>, prompt_id: String) -> Result<PromptSummary, String> {
    state.with_db(|db| PromptService::get(db, &prompt_id))
}

#[tauri::command]
pub fn create_prompt(
    state: State<'_, Arc<AppState>>,
    title: String,
    folder_id: Option<String>,
) -> Result<String, String> {
    state.with_db(|db| PromptService::create(db, &title, folder_id.as_deref()))
}

#[tauri::command]
pub fn update_prompt(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
    title: Option<String>,
    folder_id: Option<Option<String>>,
    content_syntax: Option<String>,
) -> Result<PromptSummary, String> {
    state.with_db(|db| {
        PromptService::update(
            db,
            &prompt_id,
            title.as_deref(),
            folder_id.as_ref().map(|f| f.as_deref()),
            content_syntax.as_deref(),
        )
    })
}

#[tauri::command]
pub fn trash_prompt(state: State<'_, Arc<AppState>>, prompt_id: String) -> Result<(), String> {
    state.with_db(|db| PromptService::trash(db, &prompt_id))
}

#[tauri::command]
pub fn archive_prompt(state: State<'_, Arc<AppState>>, prompt_id: String) -> Result<(), String> {
    state.with_db(|db| PromptService::archive(db, &prompt_id))
}

#[tauri::command]
pub fn unarchive_prompt(state: State<'_, Arc<AppState>>, prompt_id: String) -> Result<(), String> {
    state.with_db(|db| PromptService::unarchive(db, &prompt_id))
}

#[tauri::command]
pub fn restore_prompt(state: State<'_, Arc<AppState>>, prompt_id: String) -> Result<(), String> {
    state.with_db(|db| PromptService::restore_from_trash(db, &prompt_id))
}

#[tauri::command]
pub fn list_prompt_folders(state: State<'_, Arc<AppState>>) -> Result<Vec<PromptFolder>, String> {
    state.with_db(|db| PromptService::list_folders(db))
}

#[tauri::command]
pub fn create_prompt_folder(
    state: State<'_, Arc<AppState>>,
    title: String,
    parent_id: Option<String>,
) -> Result<PromptFolder, String> {
    state.with_db(|db| PromptService::create_folder(db, &title, parent_id.as_deref()))
}

#[tauri::command]
pub fn move_prompt_to_folder(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
    folder_id: Option<String>,
) -> Result<PromptSummary, String> {
    state.with_db(|db| PromptService::move_to_folder(db, &prompt_id, folder_id.as_deref()))
}

#[tauri::command]
pub fn list_tags(state: State<'_, Arc<AppState>>) -> Result<Vec<TagSummary>, String> {
    state.with_db(|db| PromptService::list_tags(db))
}

#[tauri::command]
pub fn set_prompt_tags(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
    tag_names: Vec<String>,
) -> Result<PromptSummary, String> {
    state.with_db(|db| PromptService::set_tags(db, &prompt_id, &tag_names))
}

#[tauri::command]
pub fn add_prompt_tag(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
    tag_name: String,
) -> Result<PromptSummary, String> {
    state.with_db(|db| PromptService::add_tag(db, &prompt_id, &tag_name))
}

#[tauri::command]
pub fn remove_prompt_tag(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
    tag_id: String,
) -> Result<PromptSummary, String> {
    state.with_db(|db| PromptService::remove_tag(db, &prompt_id, &tag_id))
}

#[tauri::command]
pub fn set_prompt_favorite(state: State<'_, Arc<AppState>>, prompt_id: String) -> Result<(), String> {
    state.with_db(|db| PromptService::set_favorite(db, &prompt_id))
}

#[tauri::command]
pub fn unset_prompt_favorite(state: State<'_, Arc<AppState>>, prompt_id: String) -> Result<(), String> {
    state.with_db(|db| PromptService::unset_favorite(db, &prompt_id))
}

#[tauri::command]
pub fn save_prompt_content(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
    content: String,
) -> Result<PromptVersionSummary, String> {
    state.with_db(|db| PromptService::save_prompt_content(db, &prompt_id, &content))
}

#[tauri::command]
pub fn list_prompt_versions(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
) -> Result<Vec<PromptVersionSummary>, String> {
    state.with_db(|db| PromptService::list_prompt_versions(db, &prompt_id))
}

#[tauri::command]
pub fn get_prompt_version_content(
    state: State<'_, Arc<AppState>>,
    version_id: String,
) -> Result<String, String> {
    state.with_db(|db| PromptService::get_prompt_version_content(db, &version_id))
}

#[tauri::command]
pub fn restore_prompt_version(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
    version_id: String,
) -> Result<PromptVersionSummary, String> {
    state.with_db(|db| PromptService::restore_prompt_version(db, &prompt_id, &version_id))
}

#[tauri::command]
pub fn can_run_prompt(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    state.with_db(|db| PromptRunService::connection_allows_run(db))
}

#[tauri::command]
pub fn start_prompt_run(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    input: StartPromptRunInput,
) -> Result<StartPromptRunResult, String> {
    PromptRunService::start_run(Arc::clone(&*state), app, input)
}

#[tauri::command]
pub fn send_prompt_message(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    input: SendPromptMessageInput,
) -> Result<StartPromptRunResult, String> {
    PromptRunService::send_message(Arc::clone(&*state), app, input)
}

#[tauri::command]
pub fn stop_prompt_run(
    state: State<'_, Arc<AppState>>,
    input: StopPromptRunInput,
) -> Result<(), String> {
    PromptRunService::stop_run(Arc::clone(&*state), input)
}

#[tauri::command]
pub fn list_prompt_sessions(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
) -> Result<Vec<SessionSummary>, String> {
    state.with_db(|db| SessionService::list_for_prompt(db, &prompt_id))
}

#[tauri::command]
pub fn get_session_transcript(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
    session_id: String,
) -> Result<Vec<TranscriptLine>, String> {
    state.with_db(|db| SessionService::get_transcript(db, &session_id, &prompt_id))
}

#[tauri::command]
pub fn delete_prompt_session(
    state: State<'_, Arc<AppState>>,
    prompt_id: String,
    session_id: String,
) -> Result<(), String> {
    state.with_db(|db| SessionService::delete(db, &session_id, &prompt_id))
}

#[tauri::command]
pub fn seed_starter_samples(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.with_db(|db| {
        PromptService::create(db, "Sample prompt", None)?;
        db.set_setting("samples_seeded", "true")
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn library_counts(state: State<'_, Arc<AppState>>) -> Result<LibraryCounts, String> {
    state.with_db(|db| {
        let prompts = PromptService::count_active(db)?;
        let agents: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM agents WHERE lifecycle_status = 'active'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let skills: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM skills WHERE lifecycle_status = 'active'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(LibraryCounts {
            prompts,
            agents,
            skills,
        })
    })
}

#[derive(serde::Serialize)]
pub struct LibraryCounts {
    pub prompts: i64,
    pub agents: i64,
    pub skills: i64,
}
