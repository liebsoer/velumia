# Velumia

Local-first desktop app (Tauri 2 + Vue 3) for prompts, agents, and skills with LangDock BYOK.

## Prerequisites

- macOS 12+ (V1 dogfood)
- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/)
- Tauri system dependencies: https://tauri.app/start/prerequisites/

## Development

```bash
git submodule update --init --recursive
npm install
npm run tauri dev
```

## Scripts

| Command | Description |
|---------|-------------|
| `npm run tauri dev` | Run desktop app in dev mode |
| `npm run build` | Build Vue frontend |
| `npm run test:bdd` | Run BDD scenarios (`@mock-langdock`) |

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

Tauri 2 · Vue 3 · SQLite · macOS dogfood (V1)

## Linear

Current sprint: [LIE-54](https://linear.app/liebich-dev/issue/LIE-54) — platform scaffold
