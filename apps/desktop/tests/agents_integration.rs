//! Agent library integration tests (AGENT-10/11/15).

use serial_test::serial;
use velumia_lib::agents::{AgentService, ListAgentFilters};
use velumia_lib::db::AppDatabase;
use velumia_lib::prompts::PromptService;

fn temp_db() -> AppDatabase {
    let dir = std::env::temp_dir().join(format!("velumia-agent-{}", uuid::Uuid::new_v4()));
    let db = AppDatabase::open(&dir).expect("open");
    db.bootstrap_owner(None).expect("bootstrap");
    db
}

// AGENT-10
#[test]
#[serial]
fn agent_10_create_in_library() {
    let db = temp_db();
    let id = AgentService::create(&db, "Research assistant").expect("create");
    let list = AgentService::list(&db, ListAgentFilters::default()).expect("list");
    assert!(list.iter().any(|a| a.id == id && a.title == "Research assistant"));
    assert!(!list[0].model.is_empty());

    let detail = AgentService::get(&db, &id).expect("get");
    assert_eq!(detail.title, "Research assistant");
    assert!(detail.instructions.is_empty());

    let slug = &detail.slug;
    let instructions_path = db.data_dir.join(format!("agents/{slug}/instructions.md"));
    assert!(instructions_path.is_file());
}

// AGENT-11
#[test]
#[serial]
fn agent_11_edit_and_attach_prompts() {
    let db = temp_db();
    let prompt_id = PromptService::create(&db, "Attached prompt", None).expect("prompt");
    let agent_id = AgentService::create(&db, "Editor agent").expect("create");

    let updated = AgentService::update(
        &db,
        &agent_id,
        Some("Updated title"),
        Some("You are a helpful research assistant."),
        Some("gpt-4o"),
        Some(true),
    )
    .expect("update");

    assert_eq!(updated.title, "Updated title");
    assert_eq!(updated.instructions, "You are a helpful research assistant.");
    assert_eq!(updated.model, "gpt-4o");
    assert!(updated.web_search);

    let with_prompts = AgentService::set_prompts(&db, &agent_id, &[prompt_id.clone()])
        .expect("set prompts");
    assert_eq!(with_prompts.prompts.len(), 1);
    assert_eq!(with_prompts.prompts[0].prompt_id, prompt_id);
    assert_eq!(with_prompts.prompts[0].sort_order, 0);
}

// AGENT-15
#[test]
#[serial]
fn agent_15_subagents_one_level() {
    let db = temp_db();
    let parent_id = AgentService::create(&db, "Parent agent").expect("parent");
    let child_id = AgentService::create(&db, "Child agent").expect("child");

    let linked = AgentService::set_subagents(&db, &parent_id, &[child_id.clone()]).expect("link");
    assert_eq!(linked.subagents.len(), 1);
    assert_eq!(linked.subagents[0].agent_id, child_id);

    let err = AgentService::set_subagents(&db, &parent_id, &[parent_id.clone()]).unwrap_err();
    assert!(err.contains("own sub-agent"));

    let grandchild_id = AgentService::create(&db, "Grandchild").expect("grandchild");
    let err = AgentService::set_subagents(&db, &child_id, &[grandchild_id]).unwrap_err();
    assert!(err.contains("sub-agents"));
}
