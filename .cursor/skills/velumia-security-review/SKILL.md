---
name: velumia-security-review
description: Trigger a Velumia security review for one sprint. Delegates to velumia-dev-security to complete security-review.md Planning or Implementation section. Use at Planning gate, before Review, or when user runs /security-review LIE-NNN.
---

# Velumia security review

## Trigger

- `/security-review LIE-NNN`
- User asks to run security review for a sprint
- SM at **Planning gate** (Planning section) or **before Review** (Implementation section)

## Prerequisites

- Sprint folder exists: `.ai/velumia-sprints/LIE-NNN/`
- For **Planning** phase: Sprint PRD synced (`sprint-prd.md`); read `velumia-pm/prd/roles-rights-architecture.md`
- For **Implementation** phase: code changes on branch; `implementation-plan.md` synced

## Workflow

1. Open `.ai/velumia-sprints/LIE-NNN/security-review.md`.
2. Delegate to **velumia-dev-security** with phase:
   - **planning** — fill § Planning (scope, trust boundaries, risks / ID table)
   - **implementation** — fill § Implementation (findings table, sign-off)
3. Architect cross-reads `architecture-review.md` when both reviews run in the same phase.
4. For Implementation phase: read git diff vs `main`; review `authorize()` on new IPC commands.

## Stop conditions

- **Planning:** threat sketch complete; Planning requirements feed Implementation Spec § Architecture and security impact
- **Implementation:** findings table complete; **Sign-off:** Critical/High resolved or waived — yes

## Open findings carry-over

At **next sprint Planning**, open Implementation findings flow through `security-carryover.md` — team discusses **directly with stakeholder** (fix / waiver / defer). See **velumia-sprint-start** step 3.

## Ceremony placement

| When | Section | Gate |
|------|---------|------|
| After Implementation Spec draft | Planning | Planning gate (with architecture Planning) |
| Before stakeholder Review | Implementation | Review gate (with architecture Implementation + QA DoD) |

## Reference

- Agent: `.cursor/agents/velumia-dev-security.md`
- Template: `.ai/velumia-sprints/_templates/security-review.md`
- Ceremony: `.cursor/skills/velumia-sprint-start/SKILL.md`
