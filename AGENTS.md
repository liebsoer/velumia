## Learned User Preferences

- Prefer `velumia-pm` as a proper git submodule at `velumia-pm/`, not a sibling symlink.
- Create Sprint PRD and Implementation Spec **in ChatPRD** (MCP `create_document` / `update_document`); do not author local `sprint-prd.md` or `implementation-plan.md` first and publish upward — local files are mirrors synced via `velumia-planning-chatprd-sync`.
- Prefer Nx-native dev commands (`pnpm nx run ui:dev`, `pnpm nx run desktop:dev`); no root npm/pnpm script aliases.
- Implementation Specs must declare shared code placement in `libs/ui/*` or `libs/desktop/*`, or explicitly inline-in-app.

## Learned Workspace Facts

- `velumia-pm/` is a git submodule at repo root pointing to `git@github.com:liebsoer/velumia-pm.git`; tracked in `.gitmodules`; listed in `nx.json` `.nxignore` (not an Nx project).
- Fresh clone: `git clone --recurse-submodules git@github.com:liebsoer/velumia.git`; existing checkout: `git submodule update --init --recursive` (see `SETUP.md`).
- BDD Gherkin specs: `velumia-pm/bdd/`; local runner: `e2e/bdd/` (`pnpm nx run bdd:test`; CI tag `@mock-langdock`).
- **Stack (V1):** Tauri 2 + Vue 3 + SQLite; local-first desktop app for prompts, agents, and skills with LangDock BYOK; macOS dogfood.
- IPC contract: `docs/ipc.md`.
- **Sprint ceremony (ChatPRD-first):** PO creates Sprint PRD in ChatPRD **before** refinement; after refinement PO updates PRD; devs create Implementation Spec (template *Feature Implementation Spec*) with sub-agent handoffs **after** PRD agreement; both docs linked on Linear; local mirrors under `.ai/velumia-sprints/LIE-NNN/` synced via `velumia-planning-chatprd-sync`. Start with `/sprint-start LIE-NNN`. See `.cursor/plans/delivery/velumia-sprint-ceremony.plan.md`.
- **Nx monorepo (LIE-76):** pnpm workspace with `apps/ui` (Vue/Vite), `apps/desktop` (Tauri/Rust), `e2e/bdd` (BDD harness); shared code in `libs/ui/*` and `libs/desktop/*` when extracted.
- Package manager: **pnpm** (`pnpm nx run …`).
- Tauri `beforeDevCommand` / `beforeBuildCommand` run from the **repo root** — use `pnpm nx run ui:dev` / `ui:build` directly (no `cd ../..`).
