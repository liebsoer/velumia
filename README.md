# Velumia

Local-first desktop app (Tauri 2 + Vue 3) for prompts, agents, and skills with LangDock BYOK.

Nx + pnpm monorepo: `apps/ui` (Vue), `apps/desktop` (Tauri/Rust), `e2e/bdd` (Cucumber).

## Prerequisites

- macOS 12+ (V1 dogfood)
- [Node.js](https://nodejs.org/) 22.23.1 (see `.node-version`; fnm/nvm/asdf supported)
- [pnpm](https://pnpm.io/) 10.12.1 (`packageManager` in `package.json`; `corepack enable` recommended)
- [Rust](https://rustup.rs/)
- Tauri system dependencies: https://tauri.app/start/prerequisites/

## Development

```bash
git submodule update --init --recursive
pnpm install
pnpm nx run desktop:dev
```

## Nx targets

| Command | Description |
|---------|-------------|
| `pnpm nx run ui:dev` | Vue dev server (port 1420) |
| `pnpm nx run desktop:dev` | Tauri desktop app in dev mode |
| `pnpm nx run ui:build` | Build Vue frontend |
| `pnpm nx run desktop:test` | Rust unit + integration tests |
| `pnpm nx run desktop:bundle` | Full Tauri release bundle |
| `pnpm nx run bdd:test` | BDD scenarios (`BDD_TAGS=@mock-langdock`) |

## Layout

```text
apps/ui/          Vue 3 + Vite
apps/desktop/     Tauri 2 + Rust
e2e/bdd/          Cucumber harness
libs/             Future shared libs (libs/ui/*, libs/desktop/*)
velumia-pm/       Specs submodule (ignored by Nx)
```

## Repos

| Repo | Role |
|------|------|
| **velumia** (this) | Application source |
| **velumia-pm** | Specs, BDD Gherkin, PM plans (git submodule `velumia-pm/`) |

## Start a sprint

```text
/sprint-start LIE-NNN
```

See [`.cursor/plans/delivery/velumia-sprint-ceremony.plan.md`](.cursor/plans/delivery/velumia-sprint-ceremony.plan.md).

## Context for AI

1. Basic Memory `projects/velumia/`
2. Linear issue (Sprint PRD + Implementation Spec linked)
3. `velumia-pm/bdd/` for Gherkin
4. Sub-agents: `.cursor/agents/`
5. IPC contract: [`docs/ipc.md`](docs/ipc.md)

## Stack

Tauri 2 · Vue 3 · SQLite · Nx · pnpm · macOS dogfood (V1)

## Linear

Current sprint: [LIE-76](https://linear.app/liebich-dev/issue/LIE-76) — Nx monorepo migration
