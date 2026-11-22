# CI, release et distribution

## Principe

Environnement de développement contraint ⇒ **la CI construit les binaires
distribués**. En local : formatage, lint, tests. En CI : compilation release
multi-plateforme, packaging, checksums, publication.

## Gate de qualité (CI push/PR)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Ajouter selon le domaine :

```bash
cargo test -p <crate-core> --release        # si les tests sont lents en debug
cargo build --locked                        # vérifier le lockfile
```

Toujours `--locked` en CI : le `Cargo.lock` doit être commité pour une
application (pas pour une bibliothèque).

## Runner Linux et glibc

`ubuntu-latest` **change de version au fil du temps** (22.04 → 24.04 → 26.04).
Un binaire compilé sur un runner récent exige une glibc récente et ne démarre
pas sur une distribution plus ancienne.

```yaml
runs-on: ubuntu-22.04   # épingler pour un binaire public
```

Vérifier le besoin réel :

```bash
objdump -T target/release/<app> | grep GLIBC | sort -Vu | tail
```

Si le binaire doit tourner partout, compiler sur la plus vieille glibc
acceptable, ou fournir une AppImage (qui embarque aussi les bibliothèques
nécessaires, mais **pas** la glibc).

## Matrice de release

Minimum pour une app desktop :

| Cible | Runner | Format |
| --- | --- | --- |
| Linux x86_64 | `ubuntu-22.04` | `.tar.gz` + `.AppImage` |
| Windows x86_64 | `windows-latest` | `.zip` |
| macOS arm64 | `macos-latest` | `.tar.gz` |
| macOS x86_64 | `macos-13` (Intel) ou cross-build | `.tar.gz` |

Archiver le binaire **et** les fichiers utiles (README, licences, assets
runtime). Nommer avec la version et la cible :

```text
<app>-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
<app>-v0.1.0-x86_64-pc-windows-msvc.zip
<app>-v0.1.0-aarch64-apple-darwin.tar.gz
<app>-v0.1.0-x86_64.AppImage
SHA256SUMS.txt
```

## AppImage

Une AppImage est un binaire + un système de fichiers SquashFS + un runtime
FUSE. Points essentiels :

- embarquer les bibliothèques non garanties sur toutes les distributions ;
- ne pas embarquer la glibc ;
- fournir un `.desktop` et une icône ;
- tester avec et sans FUSE 2 ;
- vérifier `--appimage-extract-and-run` (machines sans FUSE) ;
- ne jamais extraire dans le dépôt (`squashfs-root/` doit être gitignored).

```bash
./<app>.AppImage --version
./<app>.AppImage --appimage-extract-and-run --version
```

Si FUSE manque :

```bash
sudo apt-get install -y libfuse2
# ou
./<app>.AppImage --appimage-extract-and-run
```

Vérifier les dépendances réellement résolues :

```bash
ldd squashfs-root/usr/bin/<app>
```

Une AppImage doit rester autonome : si `ldd` montre une bibliothèque non
embarquée et non standard, l'inclure dans l'AppDir.

## Permissions GitHub

Symptôme classique à la publication :

```text
403 Resource not accessible by integration
```

Causes :

1. le workflow n'a pas `permissions: contents: write` ;
2. la permission par défaut du dépôt est `read`.

```yaml
permissions:
  contents: write
```

```bash
gh api -X PUT "repos/$OWNER/$SLUG/actions/permissions/workflow" \
  -f default_workflow_permissions=write \
  -F can_approve_pull_request_reviews=false
```

Le workflow CI garde `contents: read` (principe du moindre privilège).

## Version résolue

Éviter de dupliquer la version dans plusieurs fichiers. Une seule source :

- `Cargo.toml` (`[workspace.package] version`) ;
- ou un input `version` du workflow avec défaut lu depuis `Cargo.toml`.

Si `Cargo.toml` est la source, le script d'AppImage et le workflow doivent la
lire, pas la recopier. Une version divergente produit des archives mal nommées
et des checksums incohérents.

## Tag et release

```bash
git tag -a v0.1.0 -m "<App> v0.1.0"
git push origin v0.1.0
```

- tag **annoté** ;
- jamais déplacer un tag publié ;
- un correctif = nouvelle version (`0.1.1`), pas un remplacement d'asset ;
- la release peut aussi être déclenchée manuellement (`workflow_dispatch`) pour
  rejouer un build sans changer le code.

## Vérification après publication

Ne jamais considérer « workflow vert » comme une release valide. Télécharger et
tester :

```bash
rm -rf /tmp/release-check && mkdir -p /tmp/release-check
cd /tmp/release-check
gh release download v0.1.0 --repo <owner>/<slug>
sha256sum -c SHA256SUMS.txt

tar -xzf <app>-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
./<app>/<app> --version

chmod +x <app>-v0.1.0-x86_64.AppImage
./<app>-v0.1.0-x86_64.AppImage --version
```

Puis, si l'app a une UI :

```bash
LIBGL_ALWAYS_SOFTWARE=1 timeout 8s xvfb-run -a \
  -s "-screen 0 1440x920x24" ./<app>.AppImage --sample
```

Code 124 = l'app est restée ouverte (attendu). Un crash imprime une erreur.

## Signature et intégrité

- `SHA256SUMS.txt` est le minimum ;
- pour une distribution sérieuse, signer les checksums (minisign/GPG) et
  publier la clé publique ;
- ne pas promettre de mise à jour automatique sans mécanisme signé ;
- documenter la provenance des binaires (commit, tag, runner).

## Checklist release

- [ ] `cargo fmt --check` vert
- [ ] `clippy -D warnings` vert
- [ ] `cargo test --workspace` vert
- [ ] version cohérente partout
- [ ] CHANGELOG à jour
- [ ] README/captures à jour si l'UI a changé
- [ ] runner Linux épinglé
- [ ] permissions `contents: write`
- [ ] tag annoté poussé
- [ ] tous les jobs verts
- [ ] assets téléchargés et `sha256sum -c` OK
- [ ] binaire/AppImage lancé réellement
- [ ] `--version` correct
- [ ] cas d'usage réel testé sur le binaire distribué
