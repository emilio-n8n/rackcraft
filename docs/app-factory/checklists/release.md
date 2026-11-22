# Checklist — release

## Avant le tag

- [ ] version cohérente (`Cargo.toml`, lockfile, UI, packaging).
- [ ] CHANGELOG à jour.
- [ ] README et captures à jour.
- [ ] `cargo fmt --all -- --check` vert.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` vert.
- [ ] `cargo test --workspace` vert.
- [ ] CI verte sur le commit à tagger.
- [ ] runner Linux épinglé (glibc).
- [ ] permissions `contents: write` dans le workflow release.

## Tag et workflow

- [ ] tag annoté `vX.Y.Z`.
- [ ] tag poussé sur `origin`.
- [ ] workflow release déclenché.
- [ ] tous les jobs verts (Linux, Windows, macOS ARM/Intel).

## Vérification des assets

- [ ] `gh release download vX.Y.Z` dans un dossier vide.
- [ ] `sha256sum -c SHA256SUMS.txt` OK.
- [ ] archive Linux contient le binaire et les licences.
- [ ] archive Windows contient l'exe.
- [ ] archives macOS contiennent le binaire.
- [ ] AppImage : `--version` OK.
- [ ] AppImage : lancement GUI sous Xvfb OK.
- [ ] binaire Linux : `--version` OK.
- [ ] cas d'usage réel testé sur l'asset distribué.

## Après

- [ ] release visible avec la bonne description.
- [ ] aucun asset manquant ou mal nommé.
- [ ] issue/notes pour les problèmes connus.
- [ ] prochaine version planifiée si correctif nécessaire.
