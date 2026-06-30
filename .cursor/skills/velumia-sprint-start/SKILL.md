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

1. **SM** — create `.ai/velumia-sprints/LIE-NNN/` from `_templates/` (includes `retro-carryover.md`)
2. **SM + PO + dev subagents** — **retro carry-over (first planning work; before anything else)**
   - Read the prior completed sprint's `retro.md` (use Basic Memory **Velumia — Status** for sequence; path `.ai/velumia-sprints/LIE-*/retro.md`)
   - For each action whose **Next sprint** column matches **LIE-NNN**, names this issue, says **Before LIE-NNN**, or is otherwise overdue — decide **how to integrate** (sprint PRD scope, refinement topic, Implementation Spec subtask, ceremony/process change, Linear hygiene, etc.)
   - Record decisions in `retro-carryover.md` (integration + where it will appear)
   - If SM + PO + devs **cannot agree** how to integrate within **5 rounds** on an action → **stop planning** and **ask the stakeholder**; log in `decisions.md` under **Stakeholder — retro carry-over** until closed
   - Do **not** create the sprint PRD, refine, or sync ChatPRD until all due actions are **integrated** or **stakeholder-closed**
3. **PO** — `create_document` sprint PRD in ChatPRD → link on Linear → sync down (`velumia-planning-chatprd-sync`, type `sprint-prd`). **Include integrated retro actions** in PRD body or link to `retro-carryover.md`
4. **PO + dev subagents** — refine **using sprint PRD as input**; log in `refinement.md` (max 5 rounds/topic)
5. **PO** — merge refinement into sprint PRD → `update_document` in ChatPRD → sync down
6. **PO + devs agree** on sprint PRD; PO records story points on Linear issue
7. **Devs** — `create_document` Implementation Spec in ChatPRD using **only** [`templates/chatprd/chatprd_feature-implementation-spec.tpl.md`](../../templates/chatprd/chatprd_feature-implementation-spec.tpl.md) (ChatPRD template name: **ChatPRD: Feature Implementation Spec**). Include sub-agent ownership + handoffs in Section 5; **reflect integrated retro actions** where they affect implementation → link on Linear → sync down (type `implementation-spec`)
8. **Planning gate** passes → SM moves Linear to **In Progress** → Implementation

Do **not** start Implementation until Planning gate passes.

## Planning gate checklist

- [ ] Prior sprint `retro.md` reviewed; due actions integrated or stakeholder-closed in `retro-carryover.md` + `decisions.md`
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
- Prior completed sprint `retro.md` — **required** when present; drives step 2 retro carry-over

## Stop

Planning complete: agreed sprint PRD + Implementation Spec in ChatPRD (both linked on Linear), local mirrors synced, story points on issue.

## Reference

Canonical ceremony: [`.cursor/plans/delivery/velumia-sprint-ceremony.plan.md`](../../plans/delivery/velumia-sprint-ceremony.plan.md)
