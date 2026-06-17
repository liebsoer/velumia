---
name: velumia-sprint-start
description: Start a Velumia sprint for one Linear Feature issue. Use when the stakeholder runs /sprint-start LIE-NNN or asks to begin a sprint iteration.
---

# Velumia sprint start

## Trigger

`/sprint-start LIE-NNN` — one Linear issue = one sprint. **First sprint:** `LIE-54` (scaffold).

## ChatPRD ceremony (every sprint)

ChatPRD is the **authoring surface**. Local files under `.ai/velumia-sprints/LIE-NNN/` are **mirrors** synced from ChatPRD.

Each sprint produces **two ChatPRD documents**, both **linked on the Linear issue**:

| # | Document | Owner | When |
|---|----------|-------|------|
| 1 | **Sprint PRD** | PO | **Before** refinement — input for PO + dev discussion |
| 2 | **Implementation Spec** | Devs (SM coordinates) | **After** PO + devs agree on updated sprint PRD |

**Velumia project:** `projectId: asst_WVuIAcqzH1O6ERmhWHE91UGL`

### Flow

1. **SM** — create `.ai/velumia-sprints/LIE-NNN/` from `_templates/`
2. **PO** — `create_document` sprint PRD in ChatPRD → link on Linear → sync down (`velumia-planning-chatprd-sync`, type `sprint-prd`)
3. **PO + dev subagents** — refine **using sprint PRD as input**; log in `refinement.md` (max 5 rounds/topic)
4. **PO** — merge refinement into sprint PRD → `update_document` in ChatPRD → sync down
5. **PO + devs agree** on sprint PRD; PO records story points on Linear issue
6. **Devs** — `create_document` Implementation Spec in ChatPRD using **only** [`templates/chatprd/chatprd_feature-implementation-spec.tpl.md`](../../templates/chatprd/chatprd_feature-implementation-spec.tpl.md) (ChatPRD template name: **ChatPRD: Feature Implementation Spec**). Include sub-agent ownership + handoffs in Section 5 → link on Linear → sync down (type `implementation-spec`)
7. **Planning gate** passes → SM moves Linear to **In Progress** → Implementation

Do **not** start Implementation until Planning gate passes.

## Planning gate checklist

- [ ] Sprint PRD created in ChatPRD **before** refinement; updated after refinement; synced to `sprint-prd.md`
- [ ] Implementation Spec created in ChatPRD after PRD agreement from [`templates/chatprd/chatprd_feature-implementation-spec.tpl.md`](../../templates/chatprd/chatprd_feature-implementation-spec.tpl.md); synced to `implementation-plan.md`
- [ ] Both ChatPRD documents **linked on the Linear issue** (`save_issue` → `links`)
- [ ] Implementation Spec lists **sub-agent ownership** and **handoffs** per subtask
- [ ] Implementation Spec subtasks declare **lib placement** (`libs/ui/*`, `libs/desktop/*`, or inline-in-app)
- [ ] File paths use `apps/ui`, `apps/desktop`, `e2e/bdd` (not legacy root `src/` / `src-tauri/`)
- [ ] ≤5 refinement rounds or stakeholder cleared escalations
- [ ] Story points recorded on Linear issue (not subtasks)
- [ ] `security-review.md` Planning section complete

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

- **velumia-nx-monorepo** — Nx layout, lib placement policy, default code homes
- **velumia-dev-verify** — role-specific `pnpm nx` verify commands before handoff/PR
- **velumia-planning-chatprd-sync** — sync sprint PRD or Implementation Spec from ChatPRD to local + velumia-pm mirror + BM

## Inputs

- Linear issue ID
- `velumia-pm/bdd/*.feature.md` scenario IDs listed in issue
- Previous sprint `retro.md` actions if any

## Stop

Planning complete: agreed sprint PRD + Implementation Spec in ChatPRD (both linked on Linear), local mirrors synced, story points on issue.

## Reference

Canonical ceremony: [`.cursor/plans/delivery/velumia-sprint-ceremony.plan.md`](../../plans/delivery/velumia-sprint-ceremony.plan.md)
