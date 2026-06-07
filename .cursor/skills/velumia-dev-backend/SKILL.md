---
name: velumia-dev-backend
description: Velumia backend discipline — Rust/Tauri 2, SQLite, migrations, IPC commands, domain services, authorize() calls. Use for LIE-49+ backend subtasks.
---

# Velumia Backend (Rust/Tauri)

Senior backend engineer (15+ yrs). Stack: **Tauri 2 + Rust**.

## Conventions

- SQLite via Rust; migrations in `src-tauri/migrations/`
- IPC: documented commands; Vue calls via `@tauri-apps/api`
- `authorize(principal, permission, resource)` before protected mutations ([roles-rights-architecture](../../velumia-pm/prd/roles-rights-architecture.md))
- Keychain: OS secure storage for API keys (never SQLite for secrets)
- LangDock HTTP: prefer Rust side for secrets; coordinate with LangDock skill

## Before coding

1. Read BM **Velumia — Architecture**
2. Read sprint `implementation-plan.md` subtasks assigned to Backend
3. Read relevant `velumia-pm/prd/data-model-and-storage.md` tables

## Done when

Subtask done-when met; unit/integration tests; BDD scenarios green for owned behavior.
