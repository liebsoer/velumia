---
name: velumia-dev-verify
description: Pre-handoff and QA verification commands for the Velumia Nx monorepo. Use before subtask handoff, PR, DoD sign-off, or when the user asks to run checks.
---

# Velumia dev verify

Run from **repo root** after `git submodule update --init --recursive` and `pnpm install`.

## When to use

- Sub-agent handoff (done-when includes green verify)
- QA `dod-checklist.md` before Review
- Local CI parity check before push
- User asks "run tests" or "verify the sprint"

## Commands by role

| Role | Command |
|------|---------|
| Frontend | `pnpm nx run ui:build` |
| Backend | `pnpm nx run desktop:test` |
| BDD | `BDD_TAGS="@mock-langdock" pnpm nx run bdd:test` |
| DevOps / CI parity (build job) | `pnpm nx run ui:build && pnpm nx run desktop:test` |
| DevOps / CI parity (bdd job) | `BDD_TAGS="@mock-langdock" pnpm nx run bdd:test` |
| Optional desktop bundle | `pnpm nx run desktop:bundle` |

Use issue-specific `BDD_TAGS` when the sprint lists slice tags (e.g. `@slice1`).

## Full sprint verify (QA)

```bash
pnpm nx run ui:build
pnpm nx run desktop:test
BDD_TAGS="@mock-langdock" pnpm nx run bdd:test
```

Match tags to the Linear issue scenario set when verifying slice work.

## PR hygiene (LIE-61 retro)

- Open feature branch `lie-NNN-*` before first implementation commit.
- Open PR with `LIE-NNN` in title before stakeholder Review — even for solo dogfood.
- Run full sprint verify on the PR branch before requesting Review.
- Do not merge feature work directly to `main`.

## Notes

- Nx may serve cached results for unchanged projects (`[local cache]` in output).
- `desktop:dev` is manual smoke only — not required for CI parity.
- CI definition: `.github/workflows/ci.yml`

## Related

- Layout and targets: `.cursor/skills/velumia-nx-monorepo/SKILL.md`
