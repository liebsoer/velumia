use velumia_lib::authz::{authorize, Permission, Principal};
use velumia_lib::db::AppDatabase;
use velumia_lib::langdock::{normalize_base_url, ProfileInput, ProfileService};
use velumia_lib::state::principal;

#[test]
fn byok_profile_crud_and_normalization() {
    let dir = std::env::temp_dir().join(format!("velumia-byok-{}", uuid::Uuid::new_v4()));
    let db = AppDatabase::open(&dir).expect("open");
    db.bootstrap_owner(None).expect("bootstrap");
    let p = principal(&db).expect("principal");

    assert_eq!(
        normalize_base_url("https://langdock.example.com"),
        "https://langdock.example.com/api/public"
    );

    let profile = ProfileService::save(
        &db,
        &p,
        ProfileInput {
            name: "Work".into(),
            base_url: Some("https://api.langdock.com/".into()),
            api_key: Some("test-key".into()),
            is_default: Some(true),
        },
        None,
        false,
    )
    .expect("save");

    assert_eq!(profile.name, "Work");

    let rows: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM langdock_profiles WHERE name = 'Work'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(rows, 1);
}

#[test]
fn authz_credential_write_allowed_for_owner() {
    let p = Principal {
        user_id: "u1".into(),
    };
    assert!(matches!(
        authorize(&p, Permission::CredentialWrite),
        velumia_lib::authz::AuthzResult::Allowed
    ));
}

#[test]
fn reject_profile_without_api_key() {
    let dir = std::env::temp_dir().join(format!("velumia-byok-{}", uuid::Uuid::new_v4()));
    let db = AppDatabase::open(&dir).expect("open");
    db.bootstrap_owner(None).expect("bootstrap");
    let p = principal(&db).expect("principal");

    let err = ProfileService::save(
        &db,
        &p,
        ProfileInput {
            name: "Empty".into(),
            base_url: None,
            api_key: Some("".into()),
            is_default: None,
        },
        None,
        false,
    )
    .unwrap_err();
    assert!(err.to_string().contains("API key"));
}
