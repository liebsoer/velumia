---
name: velumia-sprint-start
description: Start a Velumia sprint for one Linear Feature issue. Use when the stakeholder runs /sprint-start LIE-NNN or asks to begin a sprint iteration.
---

# Velumia sprint start

## Trigger

`/sprint-start LIE-NNN` — one Linear issue = one sprint. **First sprint:** `LIE-54` (scaffold).

## Workflow

1. Read **Basic Memory** first: Velumia — Status, V1 Features, Dev Guide, Team playbook.
2. Fetch Linear issue `LIE-NNN` (Velumia project, V1 Launch).
3. Use the **velumia-scrum-sm** subagent to create `.ai/velumia-sprints/LIE-NNN/` from `_templates/`.
4. Use the **velumia-scrum-po** subagent to run **Planning ceremony** (refinement, ChatPRD sprint PRD, story points).
5. Do **not** start Implementation until Planning gate passes:
   - ChatPRD sprint PRD published and synced
   - ≤5 rounds or stakeholder cleared escalations
   - `implementation-plan.md` complete
   - Story points recorded in Linear

## Sub-agents (`.cursor/agents/`)

| Role | Subagent |
|------|----------|
| SM | `velumia-scrum-sm` |
| PO | `velumia-scrum-po` |
| Backend | `velumia-dev-backend` |
| Frontend | `velumia-dev-frontend` |
| BDD | `velumia-dev-bdd` |
| QA | `velumia-dev-qa` |
| LangDock | `velumia-dev-langdock` |
| DevOps | `velumia-dev-devops` |
| Security | `velumia-dev-security` |

## Skills

- **velumia-planning-chatprd-sync** — after ChatPRD `create_document`, mirror to local + BM

## Inputs

- Linear issue ID
- `velumia-pm/bdd/*.feature.md` scenario IDs listed in issue
- Previous sprint `retro.md` actions if any

## Stop

Planning complete with ChatPRD sprint PRD synced, `implementation-plan.md`, and issue story points recorded in Linear.
