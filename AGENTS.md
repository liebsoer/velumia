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

## Cursor Cloud specific instructions

Standard dev/test/build commands live in `README.md` and the Nx project files (`apps/ui/project.json`, `apps/desktop/project.json`, `e2e/bdd/project.json`); the notes below are the non-obvious caveats for this Linux cloud VM (the product itself targets macOS).

- **Node:** the active `node` is `/exec-daemon/node` (currently 22.14.x) and it shadows `nvm`/`/usr/bin/node` because `/exec-daemon` is prepended to `PATH`. It is a compatible 22.x, so the `WARN Unsupported engine: wanted 22.23.1` from pnpm is harmless — do not waste time trying to force exactly 22.23.1.
- **Rust must be ≥ 1.85.** A transitive crate (`serde_spanned`) requires `edition2024`, so the older 1.83 toolchain fails to compile. The snapshot has stable (1.96) set as the rustup default; if `cargo`/`desktop:test` ever fails with an `edition2024` error, run `rustup default stable`.
- **`desktop:test` and the `@prompt-library` BDD scenarios shell out to `cargo test`** (`e2e/bdd/steps/prompts.steps.ts`), so a working Rust toolchain is required even for those BDD tags — they are not pure-JS tests.
- **Nx caches `bdd:test` without keying on `BDD_TAGS`.** Switching tags returns stale cached results. Run `pnpm nx run bdd:test --skip-nx-cache`, or invoke the runner directly: `BDD_TAGS=@prompt-library node --import tsx e2e/bdd/run.ts`.
- **Running the real Tauri app (`desktop:dev`) on this headless Linux VM:** Tauri Linux deps (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, etc.) are already in the snapshot. Launch under a virtual display, e.g. `WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1 LIBGL_ALWAYS_SOFTWARE=1 xvfb-run -a -s "-screen 0 1280x800x24" pnpm nx run desktop:dev`.
- **Port 1420 is strict (`strictPort`).** `desktop:dev` starts its own `ui:dev` via `beforeDevCommand`, so do not run `ui:dev` separately at the same time or the Tauri launch will fail on the port conflict.
- **For UI-only work, prefer `ui:dev` (browser at `http://localhost:1420`).** It uses the `localStorage` web mock (`web-api.ts`), so the full wizard + prompt-library flows work without the Rust backend or any LangDock key.
- The `velumia-pm/` submodule uses an SSH remote and is dev-only (not needed for build/test); do not block setup on `git submodule update`.
