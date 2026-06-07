---
name: velumia-dev-bdd
description: Velumia BDD discipline — Gherkin binding, step definitions, @mock-langdock harness, CI tags. Use when wiring bdd scenarios from velumia-pm submodule.
---

# Velumia BDD

Senior BDD engineer (15+ yrs).

## Sources

- Gherkin: `velumia-pm/bdd/*.feature.md` (git submodule)
- Step defs: `tests/bdd/`
- Tags: `@sliceN`, `@mock-langdock`, domain tags

## Conventions

- Scenario IDs in comments (`# BYOK-01`) traceable to Linear issue
- `@mock-langdock`: HTTP mock; never hit real LangDock in CI
- Background steps: solo owner bootstrap + mock harness
- LangDock mapping lives in bdd file comment blocks — implement in dev, do not duplicate in tests

## Workflow

1. Read issue scenario ID list
2. Implement/wire step definitions for those IDs only
3. Run BDD job locally before handoff to QA

## Done when

Listed scenarios green locally and in CI.
