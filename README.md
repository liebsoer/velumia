# Velumia

Local-first desktop app (Tauri 2 + Vue 3) for prompts, agents, and skills with LangDock BYOK.

## Repos

| Repo | Role |
|------|------|
| **velumia** (this) | Application source |
| **velumia-pm** | Specs, BDD Gherkin, PM plans (git submodule `velumia-pm/`) |

## Start a sprint

```text
/sprint-start LIE-54
```

Invoke skill `.cursor/skills/velumia-sprint-start/SKILL.md` in Cursor.

## Context for AI

1. Basic Memory `projects/velumia/` (Vision, Dev Guide, V1 Features, Team playbook)
2. Linear issue for current sprint
3. `velumia-pm/bdd/` for Gherkin (submodule)
4. Sub-agents: `.cursor/agents/` (Scrum + dev disciplines)
5. Sprint PRDs: ChatPRD Velumia project → `.ai/velumia-sprints/LIE-NNN/sprint-prd.md`

## Remote

GitHub: `liebsoer/velumia` (add when created)

## Stack

Tauri 2 · Vue 3 · SQLite · macOS dogfood (V1)
