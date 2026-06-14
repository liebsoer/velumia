pub mod authz;
mod commands;
pub mod db;
pub mod keychain;
pub mod langdock;
mod prompts;
pub mod state;

use db::AppDatabase;
use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .expect("app data dir");
            let database = AppDatabase::open(&dir).expect("database open");
            let _ = database.bootstrap_owner(None);

            app.manage(AppState {
                db: Mutex::new(database),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::bootstrap_owner,
            commands::is_first_launch,
            commands::complete_wizard,
            commands::get_connection_widget,
            commands::list_langdock_profiles,
            commands::save_langdock_profile,
            commands::test_langdock_connection,
            commands::set_default_langdock_profile,
            commands::delete_langdock_profile,
            commands::check_authorize,
            commands::create_prompt,
            commands::can_run_prompt,
            commands::seed_starter_samples,
            commands::library_counts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
