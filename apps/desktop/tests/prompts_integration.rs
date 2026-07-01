//! Prompt library integration tests (PROMPT-*). Serial execution avoids env stub races.

use serial_test::serial;
use velumia_lib::db::AppDatabase;
use velumia_lib::prompts::{ListPromptFilters, PromptService};

fn temp_db() -> AppDatabase {
    let dir = std::env::temp_dir().join(format!("velumia-prompt-{}", uuid::Uuid::new_v4()));
    let db = AppDatabase::open(&dir).expect("open");
    db.bootstrap_owner(None).expect("bootstrap");
    db
}

struct AuthzStubDenyWrite;

impl Drop for AuthzStubDenyWrite {
    fn drop(&mut self) {
        unsafe { std::env::remove_var("VELUMIA_AUTHZ_STUB_DENY") };
    }
}

fn authz_stub_deny_write() -> AuthzStubDenyWrite {
    unsafe { std::env::set_var("VELUMIA_AUTHZ_STUB_DENY", "1") };
    AuthzStubDenyWrite
}

struct AuthzStubDenyExecute;

impl Drop for AuthzStubDenyExecute {
    fn drop(&mut self) {
        unsafe { std::env::remove_var("VELUMIA_AUTHZ_STUB_DENY_EXECUTE") };
    }
}

fn authz_stub_deny_execute() -> AuthzStubDenyExecute {
    unsafe { std::env::set_var("VELUMIA_AUTHZ_STUB_DENY_EXECUTE", "1") };
    AuthzStubDenyExecute
}

// PROMPT-06
#[test]
#[serial]
fn prompt_06_create_in_library() {
    let db = temp_db();
    let id = PromptService::create(&db, "Daily standup", None).expect("create");
    let list = PromptService::list(&db, ListPromptFilters::default()).expect("list");
    assert!(list.iter().any(|p| p.id == id && p.title == "Daily standup"));
}

// PROMPT-07
#[test]
#[serial]
fn prompt_07_folder_two_level_nesting() {
    let db = temp_db();
    let work = PromptService::create_folder(&db, "Work", None).expect("work");
    let standups = PromptService::create_folder(&db, "Standups", Some(&work.id)).expect("standups");
    let prompt_id = PromptService::create(&db, "Standup notes", None).expect("create");
    PromptService::move_to_folder(&db, &prompt_id, Some(&standups.id)).expect("move");

    let filtered = PromptService::list(
        &db,
        ListPromptFilters {
            folder_id: Some(standups.id.clone()),
            ..Default::default()
        },
    )
    .expect("list");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, prompt_id);

    let err = PromptService::create_folder(&db, "Too deep", Some(&standups.id)).unwrap_err();
    assert!(err.contains("two levels"));
}

// PROMPT-08
#[test]
#[serial]
fn prompt_08_tag_prompts() {
    let db = temp_db();
    let id = PromptService::create(&db, "Tagged prompt", None).expect("create");
    let updated = PromptService::add_tag(&db, &id, "planning").expect("add tag");
    assert!(updated.tags.iter().any(|t| t.name == "planning"));

    let tag_id = updated.tags[0].id.clone();
    let cleared = PromptService::remove_tag(&db, &id, &tag_id).expect("remove");
    assert!(cleared.tags.is_empty());
}

// PROMPT-09
#[test]
#[serial]
fn prompt_09_favorite_unfavorite() {
    let db = temp_db();
    let id = PromptService::create(&db, "Star me", None).expect("create");
    PromptService::set_favorite(&db, &id).expect("favorite");

    let favs = PromptService::list(
        &db,
        ListPromptFilters {
            favorites_only: Some(true),
            ..Default::default()
        },
    )
    .expect("list favs");
    assert_eq!(favs.len(), 1);

    PromptService::unset_favorite(&db, &id).expect("unfavorite");
    let none = PromptService::list(
        &db,
        ListPromptFilters {
            favorites_only: Some(true),
            ..Default::default()
        },
    )
    .expect("list");
    assert!(none.is_empty());
}

// PROMPT-13
#[test]
#[serial]
fn prompt_13_list_and_filter() {
    let db = temp_db();
    let folder = PromptService::create_folder(&db, "Filter folder", None).expect("folder");
    let a = PromptService::create(&db, "Alpha", Some(&folder.id)).expect("a");
    let b = PromptService::create(&db, "Beta", None).expect("b");
    PromptService::add_tag(&db, &a, "urgent").expect("tag a");
    PromptService::set_favorite(&db, &a).expect("fav a");

    let by_folder = PromptService::list(
        &db,
        ListPromptFilters {
            folder_id: Some(folder.id.clone()),
            ..Default::default()
        },
    )
    .expect("by folder");
    assert_eq!(by_folder.len(), 1);
    assert_eq!(by_folder[0].id, a);

    let tags = PromptService::list_tags(&db).expect("tags");
    let urgent_id = tags.iter().find(|t| t.name == "urgent").unwrap().id.clone();
    let by_tag = PromptService::list(
        &db,
        ListPromptFilters {
            tag_id: Some(urgent_id),
            ..Default::default()
        },
    )
    .expect("by tag");
    assert_eq!(by_tag.len(), 1);

    let by_fav = PromptService::list(
        &db,
        ListPromptFilters {
            favorites_only: Some(true),
            ..Default::default()
        },
    )
    .expect("by fav");
    assert_eq!(by_fav.len(), 1);
    assert_eq!(by_fav[0].id, a);

    let _ = b;
}

#[test]
#[serial]
fn trash_excluded_from_list() {
    let db = temp_db();
    let id = PromptService::create(&db, "To trash", None).expect("create");
    PromptService::trash(&db, &id).expect("trash");
    let list = PromptService::list(&db, ListPromptFilters::default()).expect("list");
    assert!(list.iter().all(|p| p.id != id));
}

#[test]
#[serial]
fn reject_empty_title() {
    let db = temp_db();
    let err = PromptService::create(&db, "   ", None).unwrap_err();
    assert!(err.contains("title"));
}

// PROMPT-01
#[test]
#[serial]
fn prompt_01_version_on_save() {
    let db = temp_db();
    let id = PromptService::create(&db, "Versioned", None).expect("create");
    let versions = PromptService::list_prompt_versions(&db, &id).expect("list");
    assert_eq!(versions.len(), 1);
    assert!(versions[0].is_head);

    PromptService::save_prompt_content(&db, &id, "first body").expect("save");
    let versions = PromptService::list_prompt_versions(&db, &id).expect("list");
    assert_eq!(versions.len(), 2);
    assert!(versions.iter().filter(|v| v.is_head).count() == 1);

    let v1_id = versions
        .iter()
        .find(|v| v.version_number == 1)
        .expect("v1")
        .id
        .clone();
    let v1_body = PromptService::get_prompt_version_content(&db, &v1_id).expect("v1 body");
    assert_eq!(v1_body, "");

    PromptService::save_prompt_content(&db, &id, "second body").expect("save");
    let versions = PromptService::list_prompt_versions(&db, &id).expect("list");
    assert_eq!(versions.len(), 3);

    // metadata-only edit does not bump version
    PromptService::update(&db, &id, Some("Renamed"), None, None).expect("rename");
    let after_rename = PromptService::list_prompt_versions(&db, &id).expect("list");
    assert_eq!(after_rename.len(), 3);

    // unchanged content does not bump version
    PromptService::save_prompt_content(&db, &id, "second body").expect("noop save");
    let after_noop = PromptService::list_prompt_versions(&db, &id).expect("list");
    assert_eq!(after_noop.len(), 3);
}

// PROMPT-02
#[test]
#[serial]
fn prompt_02_version_list() {
    let db = temp_db();
    let id = PromptService::create(&db, "History", None).expect("create");
    PromptService::save_prompt_content(&db, &id, "a").expect("save a");
    PromptService::save_prompt_content(&db, &id, "b").expect("save b");

    let versions = PromptService::list_prompt_versions(&db, &id).expect("list");
    assert_eq!(versions.len(), 3);
    assert_eq!(versions.iter().filter(|v| v.is_head).count(), 1);
    assert!(versions[0].is_head);
    assert!(versions[0].version_number > versions[1].version_number);
}

// PROMPT-15
#[test]
#[serial]
fn prompt_15_diff_and_restore() {
    let db = temp_db();
    let id = PromptService::create(&db, "Restore me", None).expect("create");
    PromptService::save_prompt_content(&db, &id, "original").expect("save");
    PromptService::save_prompt_content(&db, &id, "updated").expect("save");

    let versions = PromptService::list_prompt_versions(&db, &id).expect("list");
    let v2 = versions
        .iter()
        .find(|v| v.version_number == 2)
        .expect("v2");
    let v2_body = PromptService::get_prompt_version_content(&db, &v2.id).expect("body");
    assert_eq!(v2_body, "original");

    let head_before = versions.iter().find(|v| v.is_head).expect("head");
    let err = PromptService::restore_prompt_version(&db, &id, &head_before.id).unwrap_err();
    assert!(err.contains("already current"));

    let restored = PromptService::restore_prompt_version(&db, &id, &v2.id).expect("restore");
    assert!(restored.is_head);
    assert_eq!(restored.version_number, 4);

    let all = PromptService::list_prompt_versions(&db, &id).expect("list");
    assert_eq!(all.len(), 4);
    let head_body =
        PromptService::get_prompt_version_content(&db, &restored.id).expect("head body");
    assert_eq!(head_body, "original");
}

#[test]
#[serial]
fn content_syntax_update_without_version_bump() {
    let db = temp_db();
    let id = PromptService::create(&db, "Syntax", None).expect("create");
    PromptService::save_prompt_content(&db, &id, "hello").expect("save");
    let before = PromptService::list_prompt_versions(&db, &id).expect("list");
    assert_eq!(before.len(), 2);

    let updated = PromptService::update(&db, &id, None, None, Some("markdown")).expect("syntax");
    assert_eq!(updated.content_syntax, "markdown");

    let after = PromptService::list_prompt_versions(&db, &id).expect("list");
    assert_eq!(after.len(), 2);
}

#[test]
#[serial]
fn trashed_prompt_denies_version_access() {
    let db = temp_db();
    let id = PromptService::create(&db, "Trash me", None).expect("create");
    PromptService::save_prompt_content(&db, &id, "body").expect("save");
    let versions = PromptService::list_prompt_versions(&db, &id).expect("list");
    let v1 = versions
        .iter()
        .find(|v| v.version_number == 1)
        .expect("v1");

    PromptService::trash(&db, &id).expect("trash");

    assert!(PromptService::list_prompt_versions(&db, &id).is_err());
    assert!(PromptService::get_prompt_version_content(&db, &v1.id).is_err());
    assert!(PromptService::save_prompt_content(&db, &id, "new").is_err());
    assert!(PromptService::restore_prompt_version(&db, &id, &v1.id).is_err());
}

// PROMPT-10
#[test]
#[serial]
fn prompt_10_archive_unarchive() {
    let db = temp_db();
    let id = PromptService::create(&db, "Archive me", None).expect("create");

    PromptService::archive(&db, &id).expect("archive");
    let archived = PromptService::list(
        &db,
        ListPromptFilters {
            lifecycle_filter: Some("archived".into()),
            ..Default::default()
        },
    )
    .expect("list archived");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].lifecycle_status, "archived");

    let active = PromptService::list(&db, ListPromptFilters::default()).expect("list active");
    assert!(active.is_empty());

    PromptService::unarchive(&db, &id).expect("unarchive");
    let active_after = PromptService::list(&db, ListPromptFilters::default()).expect("list");
    assert_eq!(active_after.len(), 1);
    assert_eq!(active_after[0].lifecycle_status, "active");
}

// PROMPT-11
#[test]
#[serial]
fn prompt_11_trash_restore() {
    let db = temp_db();
    let id = PromptService::create(&db, "Trash restore", None).expect("create");

    PromptService::trash(&db, &id).expect("trash");
    let trashed = PromptService::list(
        &db,
        ListPromptFilters {
            lifecycle_filter: Some("trashed".into()),
            ..Default::default()
        },
    )
    .expect("list trashed");
    assert_eq!(trashed.len(), 1);

    PromptService::restore_from_trash(&db, &id).expect("restore");
    let active = PromptService::list(&db, ListPromptFilters::default()).expect("list");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].lifecycle_status, "active");
}

// PROMPT-17
#[test]
#[serial]
fn prompt_17_authz_deny_write() {
    let _stub = authz_stub_deny_write();
    let db = temp_db();
    assert!(PromptService::create(&db, "Denied", None).is_err());
}

// PROMPT-18
#[test]
#[serial]
fn prompt_18_authz_deny_execute() {
    use velumia_lib::authz::{authorize, AuthzResult, Permission};
    use velumia_lib::sessions::SessionService;
    use velumia_lib::state::principal;

    let _stub = authz_stub_deny_execute();
    let db = temp_db();
    let id = PromptService::create(&db, "Execute denied", None).expect("create");
    let p = principal(&db).expect("principal");
    assert!(matches!(
        authorize(&p, Permission::PromptExecute),
        AuthzResult::Denied { .. }
    ));
    assert!(SessionService::create(&db, &id, "instructions", "model").is_err());
}

// X-01
#[test]
#[serial]
fn x_01_durability_reopen_database() {
    let dir = std::env::temp_dir().join(format!("velumia-dur-{}", uuid::Uuid::new_v4()));
    let id = {
        let db = AppDatabase::open(&dir).expect("open");
        db.bootstrap_owner(None).expect("bootstrap");
        let id = PromptService::create(&db, "Durable", None).expect("create");
        PromptService::save_prompt_content(&db, &id, "persisted body").expect("save");
        id
    };
    let db2 = AppDatabase::open(&dir).expect("reopen");
    let prompt = PromptService::get(&db2, &id).expect("get");
    assert_eq!(prompt.title, "Durable");
    let body = PromptService::head_content_for_run(&db2, &id).expect("head");
    assert_eq!(body, "persisted body");
}
