---
name: velumia-retro-tooling-sync
description: Classify retro actions and route updates to agents, skills, AGENTS.md, ceremony plan, or Basic Memory Team playbook. Use at retrospective, retro carry-over, or when closing tooling actions from retro.md.
---

# Velumia retro tooling sync

## Trigger

- SM at **Retrospective** (after stakeholder accept)
- SM at **retro carry-over** (Planning step 2) for due `tooling` actions
- User asks to apply retro tooling updates

## Prerequisites

- Sprint folder: `.ai/velumia-sprints/LIE-NNN/`
- `retro.md` with actions and **Category** column filled

## Decision tree

For each retro action:

```
Ceremony, agent behavior, or skill workflow change?
  → edit .cursor/agents/*.md, .cursor/skills/*/SKILL.md,
    or .cursor/plans/delivery/velumia-sprint-ceremony.plan.md

Durable workspace fact (IPC, layout, toolchain, UX convention)?
  → edit AGENTS.md (or run continual-learning if fact emerged in chat)

Process-only learning (no file change needed)?
  → append Basic Memory Velumia — Team playbook (not agents/skills)

Product/code scope?
  → sprint PRD, Implementation Spec, or Linear — not this skill
```

## Workflow

1. Read `retro.md` actions; confirm **Category** per row.
2. For `tooling` actions, list target paths before editing.
3. Apply minimal edits — match surrounding agent/skill style; no scope creep.
4. Commit with convention:
   - `docs(agents): <summary> (LIE-NNN retro)` — agent/skill/ceremony changes
   - `docs(agents): <fact> (LIE-NNN retro)` — `AGENTS.md` only
5. Record **Commit ref** in retro **Tooling sign-off** and in `retro-carryover.md` **Reflected in**.
6. Append one-line summary to BM **Velumia — Team playbook** § **Tooling changes** (date, LIE-NNN, paths, commit).

## Carry-over gate

At next sprint Planning, due `tooling` actions must show either:

- **integrated** with commit ref in `retro-carryover.md`, or
- **deferred** with named target sprint in retro source

Do not pass Planning gate with open `tooling` actions lacking commit ref or deferral.

## Stop

All `tooling` actions have target files updated (or deferred with target sprint) and commit ref recorded in retro sign-off.

## Reference

- Retro template: `.ai/velumia-sprints/_templates/retro.md`
- Carry-over template: `.ai/velumia-sprints/_templates/retro-carryover.md`
- SM agent: `.cursor/agents/velumia-scrum-sm.md`
- Ceremony: `.cursor/skills/velumia-sprint-start/SKILL.md`
