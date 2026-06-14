# Velumia IPC contract

Tauri 2 invoke commands exposed to the Vue frontend via `@tauri-apps/api/core`.

## Commands

| Command | Description |
|---------|-------------|
| `ping` | Returns app version string |
| `bootstrap_owner` | Creates solo owner user + role seed |
| `is_first_launch` | True until wizard completed |
| `complete_wizard` | Marks wizard complete; optional skip |
| `get_connection_widget` | Connection status + message |
| `list_langdock_profiles` | Profile metadata (no secrets) |
| `save_langdock_profile` | Create/update profile; keychain for API key |
| `test_langdock_connection` | Probe LangDock models endpoint |
| `set_default_langdock_profile` | Set default BYOK profile |
| `delete_langdock_profile` | Remove profile + keychain entry |
| `check_authorize` | Returns allow/deny for permission action |
| `create_prompt` | Create library prompt (requires `prompt:write`) |
| `can_run_prompt` | True when LangDock connected |
| `seed_starter_samples` | Seed sample library content |
| `library_counts` | Active entity counts |

## Security

- API keys stored in OS keychain only (`keyring` service `velumia.langdock`).
- `authorize()` invoked before credential and prompt mutations.
- LangDock HTTP from Rust (`reqwest`); Vue never receives secrets.

See [roles-rights-architecture.md](../velumia-pm/prd/roles-rights-architecture.md) and [data-model-and-storage.md](../velumia-pm/prd/data-model-and-storage.md).
