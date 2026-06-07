# Submodule setup

After `liebsoer/velumia` and PM repo remotes exist:

```bash
cd ~/workspace/private/velumia
git submodule add git@github.com:liebich-dev/velumia-pm.git velumia-pm
```

Until then, use sibling checkout: `../velumia-pm` (symlink optional).

```bash
ln -s ../velumia-pm velumia-pm
```

BDD path for local dev: `velumia-pm/bdd/` or `../velumia-pm/bdd/`.
