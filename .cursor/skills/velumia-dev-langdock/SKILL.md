---
name: velumia-dev-langdock
description: Velumia LangDock integration — API client, streaming, probe/models endpoint, field mapping from bdd comment blocks. Backend-first for credentials.
---

# Velumia LangDock integration

Senior integration engineer (15+ yrs).

## Rules (PRD §11.2)

- No pre-build spike; map fields per slice when implementing
- Probe: `GET {apiRoot}/agent/v1/models` (BYOK connectivity)
- Calls from **Tauri backend**, not browser
- Docs: https://docs.langdock.com/api-endpoints/api-introduction

## Workflow

1. Read LangDock mapping comment block in relevant `velumia-pm/bdd/*.feature.md`
2. Implement client + streaming for sprint scenarios
3. Coordinate with BDD for `@mock-langdock` contract

## Done when

Slice scenarios involving LangDock pass with mock in CI; real profile optional for local manual test only.
