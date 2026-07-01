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
| `list_prompts` | List prompts; optional `folderId`, `tagId`, `favoritesOnly`, `lifecycleFilter` (`active` default, `archived`, `trashed`) |
| `get_prompt` | Single prompt summary with tags, favorite flag, and `lifecycleStatus` |
| `create_prompt` | Create prompt; optional `folderId`; requires `prompt:write` |
| `update_prompt` | Update title, folder, and/or `contentSyntax`; requires `prompt:write` |
| `archive_prompt` | Archive active prompt (`lifecycle_status = archived`); requires `prompt:write` |
| `unarchive_prompt` | Restore archived prompt to active; requires `prompt:write` |
| `restore_prompt` | Restore trashed prompt to active (clears `trashed_at`); requires `prompt:write` |
| `trash_prompt` | Soft-delete prompt (`lifecycle_status = trashed`; from active or archived); requires `prompt:write` |
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
| `start_prompt_run` | Start new prompt run session; optional `userMessage`, `variables`, `allowEmptyVariables`; requires `prompt:execute` |
| `send_prompt_message` | Send follow-up in existing session; streams assistant reply; requires `prompt:execute` |
| `stop_prompt_run` | Abort in-flight stream for `{ promptId, sessionId, runId }`; requires `prompt:execute` |
| `list_prompt_sessions` | List sessions for a prompt; requires `prompt:read` |
| `get_session_transcript` | Read session transcript lines; requires `prompt:read` |
| `delete_prompt_session` | Delete session row and transcript file; requires `prompt:execute` |
| `list_agents` | List agents; optional `lifecycleFilter` (`active` default); requires `agent:read` |
| `get_agent` | Agent detail with instructions and attachments; requires `agent:read` |
| `create_agent` | Create agent; returns id; requires `agent:write` |
| `update_agent` | Update title, instructions, model, webSearch; requires `agent:write` |
| `set_agent_prompts` | Replace ordered prompt attachments; requires `agent:write` |
| `set_agent_skills` | Replace ordered skill attachments; requires `agent:write` |
| `set_agent_subagents` | Replace one-level sub-agent links; requires `agent:write` |
| `seed_starter_samples` | Seed sample library content |
| `library_counts` | Active entity counts |

## Events (Tauri `listen`)

| Event | Payload | Description |
|-------|---------|-------------|
| `prompt-run-chunk` | `{ session_id, run_id, chunk, done }` | Streaming assistant text delta |
| `prompt-run-done` | `{ session_id, run_id }` | Run completed successfully |
| `prompt-run-error` | `{ session_id, run_id, message }` | Run failed (no secrets in message) |
| `prompt-run-stopped` | `{ session_id, run_id }` | User stopped run; partial content retained |

## Async streaming (prompt runs)

Long-running LangDock completions use a **spawn-and-event** pattern (see `apps/desktop/src/prompt_runs.rs`):

1. **Invoke returns immediately** — `start_prompt_run` / `send_prompt_message` validate authz, create or update the session, register the run in `RunRegistry`, then return `{ session_id, run_id }`.
2. **`Arc<AppState>`** — Tauri commands hold `State<'_, Arc<AppState>>`; the spawned task clones the `Arc` so `AppHandle` + DB + registry outlive the invoke handler.
3. **`RunRegistry`** — keyed by `prompt_id`; at most one active run per prompt. `ActiveRunHandle` carries an `AtomicBool` cancel flag; `stop_prompt_run` sets it and emits `prompt-run-stopped`.
4. **`tokio::spawn`** — HTTP streaming runs in a background task. The SQLite lock is **released before** the LangDock request; the task re-acquires briefly to read transcript/config and append lines.
5. **Events** — Chunks and terminal states are pushed via Tauri `emit` (`prompt-run-chunk`, `prompt-run-done`, `prompt-run-error`, `prompt-run-stopped`). Vue listens; no polling.

**Web dev parity:** `apps/ui/src/lib/web-api.ts` mirrors invoke + events via `emitPromptRunEvent` and `setTimeout`-based chunk simulation so standalone Vite dev does not require Tauri.

## Security

- API keys stored in OS keychain only (`keyring` service `velumia.langdock`).
- `authorize()` invoked before credential and prompt mutations.
- LangDock HTTP from Rust (`reqwest`); Vue never receives secrets.
- **Authz test stubs (dev/CI only):** set `VELUMIA_AUTHZ_STUB_DENY=1` to deny `prompt:write`; `VELUMIA_AUTHZ_STUB_DENY_EXECUTE=1` to deny `prompt:execute`. Web dev mirrors these via `sessionStorage` keys of the same name.

See [roles-rights-architecture.md](../velumia-pm/prd/roles-rights-architecture.md) and [data-model-and-storage.md](../velumia-pm/prd/data-model-and-storage.md).
