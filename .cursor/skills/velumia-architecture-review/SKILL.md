---
name: velumia-architecture-review
description: Trigger a Velumia architecture review for one sprint. Delegates to velumia-dev-architect to complete architecture-review.md Planning or Implementation section. Use at Planning gate, before Review, or when user runs /architecture-review LIE-NNN.
---

# Velumia architecture review

## Trigger

- `/architecture-review LIE-NNN`
- User asks to run architecture review for a sprint
- SM at **Planning gate** (Planning section) or **before Review** (Implementation section)

## Prerequisites

- Sprint folder exists: `.ai/velumia-sprints/LIE-NNN/`
- For **Planning** phase: Sprint PRD synced (`sprint-prd.md`); Implementation Spec draft or outline available for impact mapping
- For **Implementation** phase: code changes on branch; `implementation-plan.md` synced

## Workflow

1. Open `.ai/velumia-sprints/LIE-NNN/architecture-review.md`.
2. Delegate to **velumia-dev-architect** with phase:
   - **planning** — fill § Planning (scope, boundaries, ID table, Planning sign-off)
   - **implementation** — fill § Implementation (findings table, sign-off)
3. Architect comments in `refinement.md` Architect row when Planning runs during refinement.
4. For Implementation phase: read git diff vs `main` for paths under `apps/`, `e2e/`, `libs/`, `docs/ipc.md`.

## Stop conditions

- **Planning:** ID table populated; Planning sign-off set; cross-check Implementation Spec will include § Architecture and security impact
- **Implementation:** findings table complete; **Sign-off:** Critical/High resolved or waived — yes

## Ceremony placement

| When | Section | Gate |
|------|---------|------|
| After Implementation Spec draft | Planning | Planning gate (with security Planning) |
| Before stakeholder Review | Implementation | Review gate (with security Implementation + QA DoD) |

## Reference

- Agent: `.cursor/agents/velumia-dev-architect.md`
- Template: `.ai/velumia-sprints/_templates/architecture-review.md`
- Ceremony: `.cursor/skills/velumia-sprint-start/SKILL.md`
