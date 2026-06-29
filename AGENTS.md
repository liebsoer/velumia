## Learned User Preferences

- Prefer `velumia-pm` as a proper git submodule at `velumia-pm/`, not a sibling symlink.
- Create Sprint PRD and Implementation Spec **in ChatPRD** (MCP `create_document` / `update_document`); Implementation Spec **must** use [`templates/chatprd/chatprd_feature-implementation-spec.tpl.md`](./templates/chatprd/chatprd_feature-implementation-spec.tpl.md) as the section outline. Do not author local `sprint-prd.md` or `implementation-plan.md` first — local files are mirrors synced via `velumia-planning-chatprd-sync`.
- Prefer Nx-native dev commands (`pnpm nx run ui:dev`, `pnpm nx run desktop:dev`); no root npm/pnpm script aliases.
- Implementation Specs must declare shared code placement in `libs/ui/*` or `libs/desktop/*`, or explicitly inline-in-app; do not create `libs/*` Nx packages without an explicit spec subtask.
- `apps/ui` must work in both Tauri desktop and standalone Vite web; `apps/ui/src/lib/api.ts` routes to Tauri `invoke` when the bridge is present, otherwise `webInvoke` in `web-api.ts`.
- Match Node and pnpm versions between local dev and CI — pin exact versions (no loose majors in CI).

## Learned Workspace Facts

- `velumia-pm/` is a git submodule at repo root pointing to `git@github.com:liebsoer/velumia-pm.git`; tracked in `.gitmodules`; listed in `nx.json` `.nxignore` (not an Nx project).
- Fresh clone: `git clone --recurse-submodules git@github.com:liebsoer/velumia.git`; existing checkout: `git submodule update --init --recursive` (see `SETUP.md`).
- BDD Gherkin specs: `velumia-pm/bdd/`; local runner: `e2e/bdd/` (`pnpm nx run bdd:test`; CI matrix tags `@mock-langdock`, `@prompt-library`). GitHub CI does **not** checkout `velumia-pm` (dev-only submodule; Bdd harness uses `e2e/bdd/`).
- **Stack (V1):** Tauri 2 + Vue 3 + SQLite; local-first desktop app for prompts, agents, and skills with LangDock BYOK; macOS dogfood.
- IPC contract: `docs/ipc.md`.
- **Sprint ceremony (ChatPRD-first):** PO creates Sprint PRD in ChatPRD **before** refinement; after refinement PO updates PRD; devs create Implementation Spec from [`templates/chatprd/chatprd_feature-implementation-spec.tpl.md`](./templates/chatprd/chatprd_feature-implementation-spec.tpl.md) with sub-agent handoffs **after** PRD agreement; both docs linked on Linear; local mirrors under `.ai/velumia-sprints/LIE-NNN/` synced via `velumia-planning-chatprd-sync`. Start with `/sprint-start LIE-NNN`. See `.cursor/plans/delivery/velumia-sprint-ceremony.plan.md`.
- **Dev agents:** read skills `velumia-nx-monorepo` (layout, lib policy) and `velumia-dev-verify` (pre-handoff `pnpm nx` checks) before implementing or handing off.
- **Nx monorepo (LIE-76):** pnpm workspace with `apps/ui` (Vue/Vite), `apps/desktop` (Tauri/Rust), `e2e/bdd` (BDD harness); shared code in `libs/ui/*` and `libs/desktop/*` when extracted.
- **Toolchain:** Node **22.23.1** (`.node-version`, `.nvmrc`, `engines.node`); pnpm **10.12.1** (`packageManager`, `engines.pnpm`); CI reads `.node-version` and `packageManager` via `actions/setup-node` + `pnpm/action-setup`.
- Tauri `beforeDevCommand` / `beforeBuildCommand` run from the **repo root** — use `pnpm nx run ui:dev` / `ui:build` directly (no `cd ../..`).
- **Standalone Vite web dev:** `apps/ui/src/lib/web-api.ts` implements IPC commands via `localStorage` (key `velumia.web-dev.v1`); shared DTOs in `ipc-types.ts`.
- **First-launch LangDock wizard:** `apps/ui/src/views/WizardView.vue` (credentials → optional starter samples); validate before step 2, show errors on all steps, busy state during connectivity test.
