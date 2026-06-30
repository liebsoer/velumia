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
| `list_prompts` | List active prompts; optional `folderId`, `tagId`, `favoritesOnly` filters |
| `get_prompt` | Single prompt summary with tags and favorite flag |
| `create_prompt` | Create prompt; optional `folderId`; requires `prompt:write` |
| `update_prompt` | Update title, folder, and/or `contentSyntax`; requires `prompt:write` |
| `trash_prompt` | Soft-delete prompt (`lifecycle_status = trashed`) |
| `list_prompt_folders` | Flat folder list for tree UI |
| `create_prompt_folder` | Create folder; optional `parentId` (max 2 levels) |
| `move_prompt_to_folder` | Move prompt to folder or root (`folderId` null) |
| `list_tags` | Owner tag dictionary |
| `set_prompt_tags` | Replace prompt tags by name list |
| `add_prompt_tag` | Add one tag by name |
| `remove_prompt_tag` | Remove tag by id |
| `set_prompt_favorite` | Star prompt |
| `unset_prompt_favorite` | Unstar prompt |
| `save_prompt_content` | Save prompt body; appends version when content changes; requires `prompt:write` |
| `list_prompt_versions` | List linear versions for a prompt; requires `prompt:read` |
| `get_prompt_version_content` | Fetch version body by version id; requires `prompt:read` |
| `restore_prompt_version` | Copy version body as new head; requires `prompt:write` |
| `can_run_prompt` | True when LangDock connected |
| `seed_starter_samples` | Seed sample library content |
| `library_counts` | Active entity counts |

## Security

- API keys stored in OS keychain only (`keyring` service `velumia.langdock`).
- `authorize()` invoked before credential and prompt mutations.
- LangDock HTTP from Rust (`reqwest`); Vue never receives secrets.

See [roles-rights-architecture.md](../velumia-pm/prd/roles-rights-architecture.md) and [data-model-and-storage.md](../velumia-pm/prd/data-model-and-storage.md).
