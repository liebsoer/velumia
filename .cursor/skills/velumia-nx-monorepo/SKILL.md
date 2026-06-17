---
name: velumia-nx-monorepo
description: Velumia Nx + pnpm monorepo layout, project targets, lib placement policy, and Tauri hook cwd rules. Use when implementing or planning work under apps/, e2e/, libs/, nx.json, or when an Implementation Spec names files to touch.
---

# Velumia Nx monorepo

Canonical layout and conventions for the Velumia dev repo (post LIE-76).

## When to use

- Implementing or reviewing code under `apps/`, `e2e/`, or `libs/`
- Authoring or reviewing **Implementation Spec** subtasks (file paths, lib placement)
- Adding Nx projects or moving code between app and lib

## Repository layout

```text
apps/ui/          Vue 3 + Vite (Nx project: ui)
apps/desktop/     Tauri 2 + Rust (Nx project: desktop)
e2e/bdd/          Cucumber harness (Nx project: bdd)
libs/ui/*         Future shared Vue/TS (not created until spec says so)
libs/desktop/*    Future shared IPC/Tauri helpers (not created until spec says so)
velumia-pm/       Git submodule — specs/BDD Gherkin; NOT an Nx project
```

## Nx projects and targets

| Project | Config | Key targets |
|---------|--------|-------------|
| `ui` | `apps/ui/project.json` | `dev`, `build`, `typecheck` |
| `desktop` | `apps/desktop/project.json` | `dev`, `test`, `bundle` |
| `bdd` | `e2e/bdd/project.json` | `test` |

**Package manager:** pnpm only (`packageManager` in root `package.json`). No root npm script aliases — use `pnpm nx run …`.

## Dev workflow

| Command | Purpose |
|---------|---------|
| `pnpm nx run desktop:dev` | Tauri app + Vue dev server (port 1420) |
| `pnpm nx run ui:dev` | Vue only |
| `pnpm nx run ui:build` | Production frontend build → `apps/ui/dist` |
| `pnpm nx run desktop:test` | `cargo test` in `apps/desktop` |
| `pnpm nx run desktop:bundle` | Full Tauri release bundle |
| `pnpm nx run bdd:test` | Cucumber runner (`BDD_TAGS` env for tags) |

## Tauri hook cwd (important)

`beforeDevCommand` and `beforeBuildCommand` in `apps/desktop/tauri.conf.json` run from the **repo root**, not `apps/desktop`.

Use directly:

- `pnpm nx run ui:dev`
- `pnpm nx run ui:build`

Do **not** use `cd ../..` — it escapes to the wrong directory.

`frontendDist` is `../ui/dist` relative to `apps/desktop/`.

## velumia-pm submodule

- Checked out at repo root: `velumia-pm/`
- Listed in `nx.json` `.nxignore` — excluded from Nx project graph
- Gherkin sources: `velumia-pm/bdd/*.feature.md` (not in `e2e/bdd/`)

## Default code homes

| Kind | Path |
|------|------|
| Vue UI | `apps/ui/src/` |
| IPC client (TS) | `apps/ui/src/lib/` |
| Rust / Tauri / IPC commands | `apps/desktop/src/` |
| SQLite migrations | `apps/desktop/migrations/` |
| BDD steps/support | `e2e/bdd/` |
| Gherkin specs | `velumia-pm/bdd/` |

## Lib placement policy

Every **Implementation Spec** subtask must declare one of:

- `libs/ui/<name>` — shared Vue/TS
- `libs/desktop/<name>` — shared IPC/Tauri-adjacent TS
- **inline-in-app** — stays in `apps/ui` or `apps/desktop`

Author specs from [`templates/chatprd/chatprd_feature-implementation-spec.tpl.md`](../../templates/chatprd/chatprd_feature-implementation-spec.tpl.md); paths and lib placement go in **Section 5**.

Do not create `libs/*` packages without an explicit spec subtask. New libs need an Nx `project.json` under the lib folder.

## Related

- Verify commands: `.cursor/skills/velumia-dev-verify/SKILL.md`
- Sprint ceremony: `.cursor/skills/velumia-sprint-start/SKILL.md`
- Lib readme: `libs/README.md`
