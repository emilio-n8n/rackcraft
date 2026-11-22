# Bootstrap d'un nouveau dépôt

## 1. Préflight

```bash
gh auth status
git --version
rustc --version
cargo --version
df -h /tmp
```

```bash
OWNER=your-account
SLUG=my-app
TITLE="My App"
DESCRIPTION="Native Rust <domaine> application — independent community project."
WORKDIR="$HOME/$SLUG"
```

Vérifier le nom :

```bash
gh repo view "$OWNER/$SLUG" >/dev/null 2>&1 && echo "déjà utilisé" || echo "libre"
```

## 2. Dépôt public vide

```bash
gh repo create "$OWNER/$SLUG" --public --description "$DESCRIPTION"
mkdir -p "$WORKDIR" && cd "$WORKDIR"
git init -b main
git remote add origin "https://github.com/$OWNER/$SLUG.git"
```

Ne pas initialiser avec README : les premiers fichiers doivent être relus avant
le premier commit.

## 3. Fichiers de base

- `LICENSE-MIT` et `LICENSE-APACHE` (textes canoniques) ;
- `README.md` avec description, build, licence et **disclaimer** ;
- `AGENTS.md` avec environnement, architecture, conventions, tests ;
- `.gitignore` Rust ;
- `rustfmt.toml`.

### Disclaimer type

> **<App> is an independent, community open-source project.** It follows the
> conventions of the <écosystème> ecosystem, but it is not an official
> <éditeur> product and is not endorsed by them.

### `.gitignore` minimal

```gitignore
/target/
**/*.rs.bk
*.pdb
*.dSYM/
.DS_Store
Thumbs.db

.idea/
.vscode/
*.iml
*.swp

/dist/
/out/
*.tar.gz
*.zip
*.AppImage
/squashfs-root/

.env
.env.*
*.pem
*.key
secrets/
```

Adapter les extensions aux assets du domaine, mais garder les binaires et
archives hors Git.

## 4. AGENTS.md

Le fichier doit contenir au minimum :

- contraintes d'environnement (target dir, disque, pas de release locale) ;
- architecture et sens des dépendances ;
- conventions Rust/API/erreurs ;
- stratégie de tests ;
- procédure de release ;
- licence et indépendance ;
- règles sur les assets et secrets.

Voir [templates/AGENTS.md.tmpl](templates/AGENTS.md.tmpl).

## 5. Workspace Cargo

```text
Cargo.toml
rustfmt.toml
crates/core
crates/io
crates/ui-egui
apps/<slug>
```

Racine :

```toml
[workspace]
resolver = "2"
members = [
    "crates/core",
    "crates/io",
    "crates/ui-egui",
    "apps/<slug>",
]
default-members = [
    "crates/core",
    "crates/io",
    "crates/ui-egui",
    "apps/<slug>",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"
repository = "https://github.com/<owner>/<slug>"
rust-version = "1.95"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", features = ["float_roundtrip"] }
thiserror = "2"
```

Ajouter les dépendances spécialisées dans le crate concerné, pas toutes à la
racine si elles ne sont utilisées qu'une fois.

## 6. Premier commit

```bash
git add -A
git diff --cached --stat
git commit -m "feat: initial <app> workspace"
git push -u origin main
```

## 7. Permissions GitHub Actions

Un nouveau dépôt a souvent `default_workflow_permissions=read`. Une release
échouera alors à la publication avec :

```text
403 Resource not accessible by integration
```

Corriger :

```bash
gh api -X PUT "repos/$OWNER/$SLUG/actions/permissions/workflow" \
  -f default_workflow_permissions=write \
  -F can_approve_pull_request_reviews=false
```

Et déclarer explicitement dans `release.yml` :

```yaml
permissions:
  contents: write
```

Le workflow CI garde `contents: read`.

## 8. Description et topics

```bash
gh repo edit \
  --add-topic rust --add-topic desktop --add-topic native \
  --add-topic <domaine>
```

Ne pas utiliser le nom ou le logo d'une marque tierce comme identité du projet.

## 9. CI initiale

Copier [templates/ci.yml](templates/ci.yml), puis :

```bash
gh run list --limit 5
gh run watch <run-id> --exit-status
```

Le gate doit inclure :

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 10. Premier tag et release

Uniquement quand la CI est verte :

```bash
git tag -a v0.1.0 -m "<App> v0.1.0"
git push origin v0.1.0
```

Puis vérifier les assets (voir [RELEASE.md](RELEASE.md)).
