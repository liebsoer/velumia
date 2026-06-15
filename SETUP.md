# Submodule setup

```bash
cd ~/workspace/private/velumia
git submodule update --init --recursive
pnpm install
```

Fresh clone:

```bash
git clone --recurse-submodules git@github.com:liebsoer/velumia.git
cd velumia
pnpm install
```

Dev: `pnpm nx run desktop:dev`

BDD specs: `velumia-pm/bdd/` · runner: `e2e/bdd/` · `pnpm nx run bdd:test`
