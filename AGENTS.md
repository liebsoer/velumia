## Learned User Preferences

- Prefer `velumia-pm` as a proper git submodule at `velumia-pm/`, not a sibling symlink.
- Create Sprint PRD and Implementation Spec **in ChatPRD** (MCP `create_document` / `update_document`); do not author local `sprint-prd.md` or `implementation-plan.md` first and publish upward — local files are mirrors synced via `velumia-planning-chatprd-sync`.

## Learned Workspace Facts

- `velumia-pm/` is a git submodule pointing to `git@github.com:liebsoer/velumia-pm.git`; tracked in `.gitmodules`.
- Fresh clone: `git clone --recurse-submodules git@github.com:liebsoer/velumia.git`; existing checkout: `git submodule update --init --recursive` (see `SETUP.md`).
- BDD Gherkin specs: `velumia-pm/bdd/`; local runner and steps: `tests/bdd/` (`npm run test:bdd`; CI tag `@mock-langdock`).
- **Stack (V1):** Tauri 2 + Vue 3 + SQLite; local-first desktop app for prompts, agents, and skills with LangDock BYOK; macOS dogfood.
- IPC contract: `docs/ipc.md`.
- **Sprint ceremony (ChatPRD-first):** PO creates Sprint PRD in ChatPRD **before** refinement; after refinement PO updates PRD; devs create Implementation Spec (template *Feature Implementation Spec*) with sub-agent handoffs **after** PRD agreement; both docs linked on Linear; local mirrors under `.ai/velumia-sprints/LIE-NNN/` synced via `velumia-planning-chatprd-sync`. Start with `/sprint-start LIE-NNN`. See `.cursor/plans/delivery/velumia-sprint-ceremony.plan.md`.
