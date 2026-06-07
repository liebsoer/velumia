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
3. Invoke **velumia-scrum-sm** to create `.ai/velumia-sprints/LIE-NNN/` from `_templates/`.
4. Invoke **velumia-scrum-po** to run **Planning ceremony** (steps in velumia-scrum-sm).
5. Do **not** start Implementation until Planning gate passes (≤5 rounds or stakeholder cleared escalations).

## Inputs

- Linear issue ID
- `velumia-pm/bdd/*.feature.md` scenario IDs listed in issue
- Previous sprint `retro.md` actions if any

## Stop

Planning complete with `implementation-plan.md` and issue story points recorded in Linear.
