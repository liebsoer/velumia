use velumia_lib::db::AppDatabase;
use velumia_lib::prompts::{ListPromptFilters, PromptService};

fn temp_db() -> AppDatabase {
    let dir = std::env::temp_dir().join(format!("velumia-prompt-{}", uuid::Uuid::new_v4()));
    let db = AppDatabase::open(&dir).expect("open");
    db.bootstrap_owner(None).expect("bootstrap");
    db
}

// PROMPT-06
#[test]
fn prompt_06_create_in_library() {
    let db = temp_db();
    let id = PromptService::create(&db, "Daily standup", None).expect("create");
    let list = PromptService::list(&db, ListPromptFilters::default()).expect("list");
    assert!(list.iter().any(|p| p.id == id && p.title == "Daily standup"));
}

// PROMPT-07
#[test]
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
fn trash_excluded_from_list() {
    let db = temp_db();
    let id = PromptService::create(&db, "To trash", None).expect("create");
    PromptService::trash(&db, &id).expect("trash");
    let list = PromptService::list(&db, ListPromptFilters::default()).expect("list");
    assert!(list.iter().all(|p| p.id != id));
}

#[test]
fn reject_empty_title() {
    let db = temp_db();
    let err = PromptService::create(&db, "   ", None).unwrap_err();
    assert!(err.contains("title"));
}
