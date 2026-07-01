use std::collections::HashMap;
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::authz::{authorize, AuthzResult, Permission};
use crate::db::AppDatabase;
use crate::keychain;
use crate::langdock::{
    build_completion_request, CompletionMessage, CompletionStreamError, LangDockClient,
    ProfileService,
};
use crate::prompts::PromptService;
use crate::sessions::{SessionService, TranscriptLine};
use crate::state::{principal, AppState};
use crate::variables::{parse_variable_placeholders, validate_and_substitute};

pub const DEFAULT_MODEL_SETTING: &str = "langdock_default_model";
pub const DEFAULT_MODEL_FALLBACK: &str = "gpt-4o-mini";

pub const EVENT_CHUNK: &str = "prompt-run-chunk";
pub const EVENT_DONE: &str = "prompt-run-done";
pub const EVENT_ERROR: &str = "prompt-run-error";
pub const EVENT_STOPPED: &str = "prompt-run-stopped";

#[derive(Debug, Clone, Serialize)]
pub struct StartPromptRunResult {
    pub session_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptRunChunkPayload {
    pub session_id: String,
    pub run_id: String,
    pub chunk: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptRunSessionPayload {
    pub session_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptRunErrorPayload {
    pub session_id: String,
    pub run_id: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct StartPromptRunInput {
    pub prompt_id: String,
    pub user_message: Option<String>,
    pub variables: Option<HashMap<String, String>>,
    pub allow_empty_variables: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SendPromptMessageInput {
    pub prompt_id: String,
    pub session_id: String,
    pub user_message: String,
}

#[derive(Debug, Deserialize)]
pub struct StopPromptRunInput {
    pub prompt_id: String,
    pub session_id: String,
    pub run_id: String,
}

#[derive(Clone)]
pub struct ActiveRunHandle {
    pub session_id: String,
    pub run_id: String,
    pub prompt_id: String,
    cancel: Arc<AtomicBool>,
}

impl ActiveRunHandle {
    pub fn new(session_id: String, run_id: String, prompt_id: String) -> Self {
        Self {
            session_id,
            run_id,
            prompt_id,
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }

    pub fn stop(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}

pub struct RunRegistry {
    inner: Mutex<HashMap<String, ActiveRunHandle>>,
}

impl RunRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, handle: ActiveRunHandle) -> Result<(), String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        if guard.contains_key(&handle.prompt_id) {
            return Err("a run is already active for this prompt".into());
        }
        guard.insert(handle.prompt_id.clone(), handle);
        Ok(())
    }

    pub fn take(&self, prompt_id: &str) -> Option<ActiveRunHandle> {
        self.inner
            .lock()
            .ok()
            .and_then(|mut guard| guard.remove(prompt_id))
    }

    pub fn get(&self, prompt_id: &str) -> Option<ActiveRunHandle> {
        self.inner.lock().ok().and_then(|guard| {
            guard.get(prompt_id).map(|h| ActiveRunHandle {
                session_id: h.session_id.clone(),
                run_id: h.run_id.clone(),
                prompt_id: h.prompt_id.clone(),
                cancel: h.cancel_flag(),
            })
        })
    }

    pub fn is_active(&self, prompt_id: &str) -> bool {
        self.inner
            .lock()
            .map(|guard| guard.contains_key(prompt_id))
            .unwrap_or(false)
    }
}

impl Default for RunRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PromptRunService;

impl PromptRunService {
    fn require_execute(db: &AppDatabase) -> Result<(), String> {
        let p = principal(db)?;
        match authorize(&p, Permission::PromptExecute) {
            AuthzResult::Allowed => Ok(()),
            AuthzResult::Denied { .. } => Err("permission denied".into()),
        }
    }

    pub fn connection_allows_run(db: &AppDatabase) -> Result<bool, String> {
        let widget = ProfileService::connection_widget(db).map_err(|e| e.to_string())?;
        Ok(matches!(
            widget.status,
            crate::langdock::ConnectionStatus::Connected
        ))
    }

    pub fn resolve_default_model(db: &AppDatabase) -> Result<String, String> {
        if let Ok(Some(raw)) = db.get_setting(DEFAULT_MODEL_SETTING) {
            if let Ok(model) = serde_json::from_str::<String>(&raw) {
                if !model.is_empty() {
                    return Ok(model);
                }
            }
            let trimmed = raw.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('"') {
                return Ok(trimmed.to_string());
            }
        }
        Ok(DEFAULT_MODEL_FALLBACK.to_string())
    }

    pub fn prepare_instructions(
        db: &AppDatabase,
        prompt_id: &str,
        variables: Option<&HashMap<String, String>>,
        allow_empty_variables: bool,
    ) -> Result<String, String> {
        let head = PromptService::head_content_for_run(db, prompt_id)?;
        let placeholders = parse_variable_placeholders(&head);
        if placeholders.is_empty() {
            return Ok(head);
        }
        let vars = variables.cloned().unwrap_or_default();
        validate_and_substitute(&head, &vars, allow_empty_variables)
    }

    pub fn validate_variables_for_prompt(
        db: &AppDatabase,
        prompt_id: &str,
        variables: Option<&HashMap<String, String>>,
        allow_empty_variables: bool,
    ) -> Result<Vec<String>, String> {
        let head = PromptService::head_content_for_run(db, prompt_id)?;
        let placeholders = parse_variable_placeholders(&head);
        if !placeholders.is_empty() {
            let vars = variables.cloned().unwrap_or_default();
            validate_and_substitute(&head, &vars, allow_empty_variables)?;
        }
        Ok(placeholders)
    }

    pub fn langdock_credentials(db: &AppDatabase) -> Result<(String, String), String> {
        let profile = ProfileService::default_profile(db)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "LangDock is not connected".to_string())?;
        if profile.connection_status != crate::langdock::ConnectionStatus::Connected {
            return Err("LangDock is not connected".to_string());
        }
        let keychain_ref: String = db
            .conn
            .query_row(
                "SELECT keychain_ref FROM langdock_profiles WHERE id = ?1",
                rusqlite::params![profile.id],
                |r| r.get(0),
            )
            .map_err(|_| "profile not found".to_string())?;
        let api_key = keychain::get_secret(&keychain_ref).map_err(|e| e.to_string())?;
        Ok((profile.base_url, api_key))
    }

    pub fn start_run(
        state: Arc<AppState>,
        app: AppHandle,
        input: StartPromptRunInput,
    ) -> Result<StartPromptRunResult, String> {
        let allow_empty = input.allow_empty_variables.unwrap_or(false);
        let prompt_id = input.prompt_id.clone();
        let user_message = input.user_message.clone();
        let variables = input.variables.clone();

        let (session_id, run_id, should_stream) = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            Self::require_execute(&db)?;
            if !Self::connection_allows_run(&db)? {
                return Err("LangDock is not connected".into());
            }
            if state.runs.is_active(&prompt_id) {
                return Err("a run is already active for this prompt".into());
            }

            let instructions =
                Self::prepare_instructions(&db, &prompt_id, variables.as_ref(), allow_empty)?;
            let model = Self::resolve_default_model(&db)?;
            let session = SessionService::create(&db, &prompt_id, &instructions, &model)?;

            let should_stream = user_message
                .as_ref()
                .is_some_and(|m| !m.trim().is_empty());

            if should_stream {
                let msg = user_message.as_ref().unwrap();
                SessionService::append_line(
                    &db,
                    &session.id,
                    &prompt_id,
                    &TranscriptLine::Message {
                        role: "user".into(),
                        content: msg.clone(),
                    },
                )?;
            }

            let run_id = Uuid::new_v4().to_string();

            if should_stream {
                let handle =
                    ActiveRunHandle::new(session.id.clone(), run_id.clone(), prompt_id.clone());
                state.runs.register(handle)?;
            }

            (session.id, run_id, should_stream)
        };

        if should_stream {
            spawn_stream(state, app, prompt_id, session_id.clone(), run_id.clone());
        }

        Ok(StartPromptRunResult {
            session_id,
            run_id,
        })
    }

    pub fn send_message(
        state: Arc<AppState>,
        app: AppHandle,
        input: SendPromptMessageInput,
    ) -> Result<StartPromptRunResult, String> {
        let prompt_id = input.prompt_id.clone();
        let session_id = input.session_id.clone();
        let user_message = input.user_message.trim().to_string();

        if user_message.is_empty() {
            return Err("message is required".into());
        }

        let run_id = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            Self::require_execute(&db)?;
            if !Self::connection_allows_run(&db)? {
                return Err("LangDock is not connected".into());
            }
            if state.runs.is_active(&prompt_id) {
                return Err("a run is already active for this prompt".into());
            }
            SessionService::assert_session_for_prompt(&db, &session_id, &prompt_id)?;
            SessionService::append_line(
                &db,
                &session_id,
                &prompt_id,
                &TranscriptLine::Message {
                    role: "user".into(),
                    content: user_message,
                },
            )?;

            let run_id = Uuid::new_v4().to_string();
            let handle = ActiveRunHandle::new(session_id.clone(), run_id.clone(), prompt_id.clone());
            state.runs.register(handle)?;
            run_id
        };

        spawn_stream(state, app, prompt_id, session_id.clone(), run_id.clone());

        Ok(StartPromptRunResult {
            session_id,
            run_id,
        })
    }

    pub fn stop_run(state: Arc<AppState>, input: StopPromptRunInput) -> Result<(), String> {
        {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            Self::require_execute(&db)?;
            SessionService::assert_session_for_prompt(&db, &input.session_id, &input.prompt_id)?;
        }

        if let Some(handle) = state.runs.get(&input.prompt_id) {
            if handle.session_id == input.session_id && handle.run_id == input.run_id {
                handle.stop();
                return Ok(());
            }
        }
        Err("no active run for session".into())
    }
}

fn spawn_stream(
    state: Arc<AppState>,
    app: AppHandle,
    prompt_id: String,
    session_id: String,
    run_id: String,
) {
    tokio::spawn(async move {
        let result = execute_stream(&app, &state, &prompt_id, &session_id, &run_id).await;
        state.runs.take(&prompt_id);
        if let Err(e) = result {
            let _ = emit_error(&app, &session_id, &run_id, &e);
        }
    });
}

async fn execute_stream(
    app: &AppHandle,
    state: &Arc<AppState>,
    prompt_id: &str,
    session_id: &str,
    run_id: &str,
) -> Result<(), String> {
    let cancel = state
        .runs
        .get(prompt_id)
        .filter(|h| h.session_id == session_id && h.run_id == run_id)
        .map(|h| h.cancel_flag())
        .ok_or_else(|| "run not found".to_string())?;

    let (instructions, model, messages, base_url, api_key) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let (instructions, model) =
            SessionService::read_run_config(&db, session_id, prompt_id)?;
        let lines = SessionService::read_transcript_for_run(&db, session_id, prompt_id)?;
        let chat = SessionService::chat_messages_from_transcript(&lines);
        let messages: Vec<CompletionMessage> = chat
            .into_iter()
            .map(|(role, content)| CompletionMessage { role, content })
            .collect();
        let (base_url, api_key) = PromptRunService::langdock_credentials(&db)?;
        (instructions, model, messages, base_url, api_key)
    };

    let body = build_completion_request(model, instructions, messages);
    let client = LangDockClient::new();
    let app_handle = app.clone();
    let session_id_owned = session_id.to_string();
    let run_id_owned = run_id.to_string();
    let mut assistant_text = String::new();

    let stream_result = client
        .stream_completion(&base_url, &api_key, &body, |delta| {
            assistant_text.push_str(delta);
            let payload = PromptRunChunkPayload {
                session_id: session_id_owned.clone(),
                run_id: run_id_owned.clone(),
                chunk: delta.to_string(),
                done: false,
            };
            let _ = app_handle.emit(EVENT_CHUNK, &payload);
            if cancel.load(Ordering::SeqCst) {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        })
        .await;

    let stopped = cancel.load(Ordering::SeqCst);

    if !assistant_text.is_empty() {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        SessionService::append_line(
            &db,
            session_id,
            prompt_id,
            &TranscriptLine::Message {
                role: "assistant".into(),
                content: assistant_text,
            },
        )?;
    }

    if stopped || matches!(stream_result, Err(CompletionStreamError::Cancelled)) {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        SessionService::mark_stopped(&db, session_id, prompt_id)?;
        let _ = app.emit(
            EVENT_STOPPED,
            PromptRunSessionPayload {
                session_id: session_id.to_string(),
                run_id: run_id.to_string(),
            },
        );
        return Ok(());
    }

    match stream_result {
        Ok(()) => {
            let _ = app.emit(
                EVENT_DONE,
                PromptRunSessionPayload {
                    session_id: session_id.to_string(),
                    run_id: run_id.to_string(),
                },
            );
            Ok(())
        }
        Err(e) => Err(user_safe_stream_error(&e)),
    }
}

fn user_safe_stream_error(err: &CompletionStreamError) -> String {
    match err {
        CompletionStreamError::Api { status, message: _ } => {
            if *status == 401 {
                "LangDock rejected these credentials".into()
            } else if *status == 429 {
                "LangDock rate limit exceeded".into()
            } else if *status >= 500 {
                "LangDock is temporarily unavailable".into()
            } else {
                "LangDock request failed".into()
            }
        }
        CompletionStreamError::Http(_) => "Cannot reach LangDock".into(),
        CompletionStreamError::Cancelled => "stream cancelled".into(),
    }
}

fn emit_error(app: &AppHandle, session_id: &str, run_id: &str, message: &str) -> Result<(), String> {
    let _ = app.emit(
        EVENT_ERROR,
        PromptRunErrorPayload {
            session_id: session_id.to_string(),
            run_id: run_id.to_string(),
            message: message.to_string(),
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::PromptService;

    fn temp_db() -> AppDatabase {
        let dir = std::env::temp_dir().join(format!("velumia-run-{}", Uuid::new_v4()));
        let db = AppDatabase::open(&dir).expect("open");
        db.bootstrap_owner(None).expect("bootstrap");
        db
    }

    #[test]
    fn gating_denies_without_profile() {
        let db = temp_db();
        assert!(!PromptRunService::connection_allows_run(&db).unwrap());
    }

    #[test]
    fn variable_validation_blocks_empty() {
        let db = temp_db();
        let prompt_id = PromptService::create(&db, "Vars", None).expect("create");
        PromptService::save_prompt_content(&db, &prompt_id, "Write about {{topic}}")
            .expect("save");

        let err = PromptRunService::prepare_instructions(&db, &prompt_id, None, false).unwrap_err();
        assert!(err.contains("do not match") || err.contains("empty"));

        let mut vars = HashMap::new();
        vars.insert("topic".into(), "AI".into());
        let out = PromptRunService::prepare_instructions(&db, &prompt_id, Some(&vars), false)
            .expect("substitute");
        assert_eq!(out, "Write about AI");
    }
}
