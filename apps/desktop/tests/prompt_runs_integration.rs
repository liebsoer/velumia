use std::collections::HashMap;

use velumia_lib::db::AppDatabase;
use velumia_lib::langdock::{normalize_base_url, ProfileInput, ProfileService};
use velumia_lib::prompt_runs::PromptRunService;
use velumia_lib::prompts::PromptService;
use velumia_lib::sessions::{SessionService, TranscriptLine};
use velumia_lib::state::principal;
use velumia_lib::variables::parse_variable_placeholders;

fn temp_db() -> AppDatabase {
    let dir = std::env::temp_dir().join(format!("velumia-prun-{}", uuid::Uuid::new_v4()));
    let db = AppDatabase::open(&dir).expect("open");
    db.bootstrap_owner(None).expect("bootstrap");
    db
}

fn seed_connected_profile(db: &AppDatabase, base_url: &str) {
    let p = principal(db).expect("principal");
    let profile = ProfileService::save(
        db,
        &p,
        ProfileInput {
            name: "Test".into(),
            base_url: Some(base_url.into()),
            api_key: Some("test-key".into()),
            is_default: Some(true),
        },
        None,
        false,
    )
    .expect("save profile");

    db.conn
        .execute(
            "UPDATE langdock_profiles SET connection_status = 'connected', base_url = ?1 WHERE id = ?2",
            rusqlite::params![normalize_base_url(base_url), profile.id],
        )
        .expect("connect");
}

// PROMPT-04 — Not configured, Offline, Configuration error block runs; Connected allows
#[test]
fn prompt_04_run_blocked_when_not_connected() {
    let db = temp_db();
    assert!(!PromptRunService::connection_allows_run(&db).expect("not configured"));

    seed_connected_profile(&db, "http://127.0.0.1:18080");

    db.conn
        .execute(
            "UPDATE langdock_profiles SET connection_status = 'offline'",
            [],
        )
        .expect("offline");
    assert!(!PromptRunService::connection_allows_run(&db).expect("offline"));

    db.conn
        .execute(
            "UPDATE langdock_profiles SET connection_status = 'configuration_error'",
            [],
        )
        .expect("configuration error");
    assert!(!PromptRunService::connection_allows_run(&db).expect("configuration error"));

    db.conn
        .execute(
            "UPDATE langdock_profiles SET connection_status = 'connected'",
            [],
        )
        .expect("connected");
    assert!(PromptRunService::connection_allows_run(&db).expect("connected"));
}

// PROMPT-14
#[test]
fn prompt_14_variables_block_empty_and_substitute() {
    let db = temp_db();
    let prompt_id = PromptService::create(&db, "Vars", None).expect("create");
    PromptService::save_prompt_content(&db, &prompt_id, "Write about {{topic}}").expect("save");

    let head = PromptService::head_content_for_run(&db, &prompt_id).expect("head");
    let placeholders = parse_variable_placeholders(&head);
    assert_eq!(placeholders, vec!["topic".to_string()]);

    let err = PromptRunService::prepare_instructions(&db, &prompt_id, None, false).unwrap_err();
    assert!(err.contains("do not match"));

    let mut vars = HashMap::new();
    vars.insert("topic".into(), "".into());
    let err = PromptRunService::prepare_instructions(&db, &prompt_id, Some(&vars), false)
        .unwrap_err();
    assert!(err.contains("empty"));

    vars.insert("topic".into(), "AI".into());
    let out = PromptRunService::prepare_instructions(&db, &prompt_id, Some(&vars), false)
        .expect("substitute");
    assert_eq!(out, "Write about AI");
}

#[test]
fn prompt_14_allow_empty_variables() {
    let db = temp_db();
    let prompt_id = PromptService::create(&db, "Greet", None).expect("create");
    PromptService::save_prompt_content(&db, &prompt_id, "Greet {{name}}").expect("save");

    let mut vars = HashMap::new();
    vars.insert("name".into(), "".into());
    let out = PromptRunService::prepare_instructions(&db, &prompt_id, Some(&vars), true)
        .expect("allow empty");
    assert_eq!(out, "Greet ");
}

// PROMPT-03 (persistence without live HTTP)
#[test]
fn prompt_03_session_and_transcript_persisted() {
    let db = temp_db();
    let prompt_id = PromptService::create(&db, "Stream", None).expect("create");
    PromptService::save_prompt_content(&db, &prompt_id, "Summarize the day").expect("save");

    let session =
        SessionService::create(&db, &prompt_id, "Summarize the day", "mock-model").expect("session");

    SessionService::append_line(
        &db,
        &session.id,
        &prompt_id,
        &TranscriptLine::Message {
            role: "user".into(),
            content: "Hello".into(),
        },
    )
    .expect("user");
    SessionService::append_line(
        &db,
        &session.id,
        &prompt_id,
        &TranscriptLine::Message {
            role: "assistant".into(),
            content: "mock-reply:Hello".into(),
        },
    )
    .expect("assistant");

    let rows: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE entity_id = ?1",
            rusqlite::params![prompt_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(rows, 1);

    let path = db
        .data_dir
        .join(SessionService::transcript_relative_path(&session.id));
    assert!(path.exists());

    let history = SessionService::list_for_prompt(&db, &prompt_id).expect("list");
    assert_eq!(history.len(), 1);

    let transcript =
        SessionService::get_transcript(&db, &session.id, &prompt_id).expect("transcript");
    let roles: Vec<_> = transcript
        .iter()
        .filter_map(|line| {
            if let TranscriptLine::Message { role, content } = line {
                Some((role.as_str(), content.as_str()))
            } else {
                None
            }
        })
        .collect();
    assert!(roles.contains(&("user", "Hello")));
    assert!(roles.contains(&("assistant", "mock-reply:Hello")));
}

#[test]
fn prompt_03_idor_foreign_session_denied() {
    let db = temp_db();
    let prompt_a = PromptService::create(&db, "A", None).expect("a");
    let prompt_b = PromptService::create(&db, "B", None).expect("b");
    let session = SessionService::create(&db, &prompt_a, "instr", "m").expect("session");

    assert!(SessionService::get_transcript(&db, &session.id, &prompt_b).is_err());
    assert!(SessionService::delete(&db, &session.id, &prompt_b).is_err());
}

// PROMPT-12
#[test]
fn prompt_12_stop_marks_stopped_and_retains_partial() {
    let db = temp_db();
    let prompt_id = PromptService::create(&db, "Stop", None).expect("create");
    let session = SessionService::create(&db, &prompt_id, "instr", "m").expect("session");

    SessionService::append_line(
        &db,
        &session.id,
        &prompt_id,
        &TranscriptLine::Message {
            role: "assistant".into(),
            content: "partial".into(),
        },
    )
    .expect("partial");
    SessionService::mark_stopped(&db, &session.id, &prompt_id).expect("stopped");

    let list = SessionService::list_for_prompt(&db, &prompt_id).expect("list");
    assert!(list[0].stopped);

    let transcript =
        SessionService::get_transcript(&db, &session.id, &prompt_id).expect("transcript");
    assert!(transcript.iter().any(|line| matches!(
        line,
        TranscriptLine::Meta { event } if event == "stopped"
    )));
    assert!(transcript.iter().any(|line| matches!(
        line,
        TranscriptLine::Message { content, .. } if content == "partial"
    )));
}

// PROMPT-16
#[test]
fn prompt_16_delete_session_removes_row_and_file() {
    let db = temp_db();
    let prompt_id = PromptService::create(&db, "Delete session", None).expect("create");
    let session = SessionService::create(&db, &prompt_id, "instr", "m").expect("session");
    let path = db
        .data_dir
        .join(SessionService::transcript_relative_path(&session.id));
    assert!(path.exists());

    SessionService::delete(&db, &session.id, &prompt_id).expect("delete");

    let rows: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE id = ?1",
            rusqlite::params![session.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(rows, 0);
    assert!(!path.exists());
}

#[tokio::test]
async fn prompt_03_stream_completion_against_mock() {
    use velumia_lib::langdock::{build_completion_request, LangDockClient};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/public/agent/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "0:\"mock\"\n0:\"-reply:Hi\"\nd:{\"finishReason\":\"stop\"}\n",
        ))
        .mount(&server)
        .await;

    let body = build_completion_request(
        "mock-model",
        "Summarize the day",
        vec![velumia_lib::langdock::CompletionMessage {
            role: "user".into(),
            content: "Hi".into(),
        }],
    );

    let client = LangDockClient::new();
    let mut collected = String::new();
    client
        .stream_completion(
            &normalize_base_url(&server.uri()),
            "test-key",
            &body,
            |delta| {
                collected.push_str(delta);
                std::ops::ControlFlow::Continue(())
            },
        )
        .await
        .expect("stream");

    assert_eq!(collected, "mock-reply:Hi");
}
