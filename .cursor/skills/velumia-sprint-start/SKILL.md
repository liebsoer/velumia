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

1. **SM** — create `.ai/velumia-sprints/LIE-NNN/` from `_templates/` (includes carry-over and review templates)
2. **SM + PO + dev subagents** — **retro carry-over (first planning work)**
   - Read the prior completed sprint's `retro.md` (use Basic Memory **Velumia — Status** for sequence; path `.ai/velumia-sprints/LIE-*/retro.md`)
   - For each action whose **Next sprint** column matches **LIE-NNN**, names this issue, says **Before LIE-NNN**, or is otherwise overdue — decide **how to integrate**
   - Record in `retro-carryover.md` with **Category** and **Reflected in** (include `.cursor/agents/*`, `.cursor/skills/*`, `AGENTS.md` when applicable)
   - For due `tooling` actions: apply per **velumia-retro-tooling-sync** or confirm prior sprint commit ref; deferrals must name target sprint
   - Escalate undecided items to **stakeholder** via `decisions.md` **Stakeholder — retro carry-over** (max 5 rounds)
   - Gate: all due actions **integrated** or **stakeholder-closed**; all due `tooling` actions have **commit ref** or explicit deferral
3. **SM + PO + stakeholder** — **security carry-over** (before sprint PRD)
   - Read prior sprint `security-review.md` § Implementation for **open** findings (especially Critical/High)
   - Record in `security-carryover.md`; **stakeholder disposition required** for open Critical/High (fix / waiver / defer) — team does not close without stakeholder answer
   - Gate: all open Critical/High **stakeholder-dispositioned**
4. **SM + PO + dev subagents** — **architecture carry-over** (before sprint PRD)
   - Read prior sprint `architecture-review.md` § Implementation for open findings due this sprint
   - Record in `architecture-carryover.md`; if SM + PO + devs cannot agree within **5 rounds** → ask **stakeholder**; log in `decisions.md` **Stakeholder — architecture carry-over**
   - Gate: all due findings **integrated** or **stakeholder-closed**
5. **PO** — `create_document` sprint PRD in ChatPRD → link on Linear → sync down. Include integrated carry-over items or link to carry-over files
6. **PO + dev subagents** — refine using sprint PRD; **mandatory topic: Architecture and security impact** in `refinement.md` (max 5 rounds/topic)
7. **PO** — merge refinement into sprint PRD → `update_document` → sync down
8. **PO + devs agree** on sprint PRD; PO records story points on Linear issue
9. **Devs** — `create_document` Implementation Spec from repo template. Section 5 must include **Architecture and security impact** (maps Planning IDs from `architecture-review.md` / `security-review.md`), sub-agent ownership + handoffs → link on Linear → sync down
10. **velumia-dev-security** — `security-review.md` § Planning (skill **velumia-security-review**, phase `planning`)
11. **velumia-dev-architect** — `architecture-review.md` § Planning (skill **velumia-architecture-review**, phase `planning`)
12. **Planning gate** passes → SM moves Linear to **In Progress** → Implementation

**Before Review (Implementation complete):**

- **velumia-dev-qa** — `dod-checklist.md`
- **velumia-dev-security** — `security-review.md` § Implementation (skill **velumia-security-review**, phase `implementation`)
- **velumia-dev-architect** — `architecture-review.md` § Implementation (skill **velumia-architecture-review**, phase `implementation`)

Do **not** start Implementation until Planning gate passes. Do **not** move to Review until both Implementation reviews and QA DoD are complete.

## Planning gate checklist

- [ ] Prior sprint `retro.md` reviewed; due actions integrated or stakeholder-closed in `retro-carryover.md` + `decisions.md`
- [ ] Due `tooling` retro actions applied or commit-ref'd per **velumia-retro-tooling-sync** (agents/skills/`AGENTS.md`/ceremony plan)
- [ ] Prior sprint open security findings dispositioned in `security-carryover.md` (stakeholder for Critical/High)
- [ ] Prior sprint open architecture findings integrated or stakeholder-closed in `architecture-carryover.md` + `decisions.md`
- [ ] Sprint PRD created in ChatPRD **before** refinement; updated after refinement; synced to `sprint-prd.md`
- [ ] Refinement includes **Architecture and security impact** topic
- [ ] Implementation Spec created in ChatPRD after PRD agreement from [`templates/chatprd/chatprd_feature-implementation-spec.tpl.md`](../../templates/chatprd/chatprd_feature-implementation-spec.tpl.md); synced to `implementation-plan.md`
- [ ] Implementation Spec § **Architecture and security impact** maps Planning review IDs
- [ ] Both ChatPRD documents **linked on the Linear issue** (`save_issue` → `links`)
- [ ] Implementation Spec lists **sub-agent ownership** and **handoffs** per subtask
- [ ] Implementation Spec subtasks declare **lib placement** (`libs/ui/*`, `libs/desktop/*`, or inline-in-app)
- [ ] File paths use `apps/ui`, `apps/desktop`, `e2e/bdd` (not legacy root `src/` / `src-tauri/`)
- [ ] ≤5 refinement rounds or stakeholder cleared escalations
- [ ] Story points recorded on Linear issue (not subtasks)
- [ ] `security-review.md` Planning section complete
- [ ] `architecture-review.md` Planning section complete

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
| Architect | `velumia-dev-architect` |

## Skills

- **velumia-nx-monorepo** — Nx layout, lib placement policy, default code homes
- **velumia-dev-verify** — role-specific `pnpm nx` verify commands before handoff/PR
- **velumia-planning-chatprd-sync** — sync sprint PRD or Implementation Spec from ChatPRD to local + velumia-pm mirror + BM
- **velumia-security-review** — on-demand `/security-review LIE-NNN` (Planning + Implementation phases)
- **velumia-architecture-review** — on-demand `/architecture-review LIE-NNN` (Planning + Implementation phases)
- **velumia-retro-tooling-sync** — classify retro actions and update agents/skills/`AGENTS.md`/BM playbook

## Inputs

- Linear issue ID
- `velumia-pm/bdd/*.feature.md` scenario IDs listed in issue
- Prior completed sprint `retro.md` — drives step 2 retro carry-over
- Prior sprint `security-review.md` / `architecture-review.md` — drives steps 3–4 carry-over when open findings exist

## Stop

Planning complete: agreed sprint PRD + Implementation Spec in ChatPRD (both linked on Linear), local mirrors synced, story points on issue.

## Reference

Canonical ceremony: [`.cursor/plans/delivery/velumia-sprint-ceremony.plan.md`](../../plans/delivery/velumia-sprint-ceremony.plan.md)
