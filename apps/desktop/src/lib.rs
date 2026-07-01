pub mod authz;
mod commands;
pub mod db;
pub mod keychain;
pub mod langdock;
pub mod agents;
pub mod prompts;
pub mod prompt_runs;
pub mod sessions;
pub mod state;
pub mod variables;

use db::AppDatabase;
use prompt_runs::RunRegistry;
use state::AppState;
use std::sync::{Arc, Mutex};
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

            app.manage(Arc::new(AppState {
                db: Arc::new(Mutex::new(database)),
                runs: RunRegistry::new(),
            }));
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
            commands::list_prompts,
            commands::get_prompt,
            commands::create_prompt,
            commands::update_prompt,
            commands::trash_prompt,
            commands::archive_prompt,
            commands::unarchive_prompt,
            commands::restore_prompt,
            commands::list_prompt_folders,
            commands::create_prompt_folder,
            commands::move_prompt_to_folder,
            commands::list_tags,
            commands::set_prompt_tags,
            commands::add_prompt_tag,
            commands::remove_prompt_tag,
            commands::set_prompt_favorite,
            commands::unset_prompt_favorite,
            commands::save_prompt_content,
            commands::list_prompt_versions,
            commands::get_prompt_version_content,
            commands::restore_prompt_version,
            commands::can_run_prompt,
            commands::start_prompt_run,
            commands::send_prompt_message,
            commands::stop_prompt_run,
            commands::list_prompt_sessions,
            commands::get_session_transcript,
            commands::delete_prompt_session,
            commands::list_agents,
            commands::get_agent,
            commands::create_agent,
            commands::update_agent,
            commands::set_agent_prompts,
            commands::set_agent_skills,
            commands::set_agent_subagents,
            commands::seed_starter_samples,
            commands::library_counts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
