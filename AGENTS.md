## Learned User Preferences

- Prefer `velumia-pm` as a proper git submodule at `velumia-pm/`, not a sibling symlink.

## Learned Workspace Facts

- `velumia-pm/` is a git submodule pointing to `git@github.com:liebsoer/velumia-pm.git`; tracked in `.gitmodules`.
- Fresh clone: `git clone --recurse-submodules git@github.com:liebsoer/velumia.git`; existing checkout: `git submodule update --init --recursive` (see `SETUP.md`).
- BDD specs path: `velumia-pm/bdd/`.
- **Sprint ceremony (ChatPRD-first):** PO creates Sprint PRD in ChatPRD before refinement; after refinement PO updates PRD; devs create Implementation Spec (template *Feature Implementation Spec*) with sub-agent handoffs; both docs linked on Linear; local mirrors synced via `velumia-planning-chatprd-sync`. See `.cursor/plans/delivery/velumia-sprint-ceremony.plan.md`.
