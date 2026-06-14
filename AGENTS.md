## Learned User Preferences

- Prefer `velumia-pm` as a proper git submodule at `velumia-pm/`, not a sibling symlink.

## Learned Workspace Facts

- `velumia-pm/` is a git submodule pointing to `git@github.com:liebsoer/velumia-pm.git`; tracked in `.gitmodules`.
- Fresh clone: `git clone --recurse-submodules git@github.com:liebsoer/velumia.git`; existing checkout: `git submodule update --init --recursive` (see `SETUP.md`).
- BDD specs path: `velumia-pm/bdd/`.
