# Checklist — bootstrap

## Environnement

- [ ] `rustc --version` et `cargo --version` fonctionnent.
- [ ] `CARGO_TARGET_DIR=/tmp/cargo-target` exporté.
- [ ] `df -h /tmp` montre assez d'espace.
- [ ] libs système du domaine installées.
- [ ] `xvfb-run` disponible si UI.
- [ ] `gh auth status` valide.

## Dépôt

- [ ] `gh repo create` public (ou privé selon le projet).
- [ ] `git init -b main`, remote `origin` correct.
- [ ] `LICENSE-MIT` et `LICENSE-APACHE` présents.
- [ ] README avec description, build, licence, disclaimer.
- [ ] AGENTS.md avec les règles du projet.
- [ ] `.gitignore` adapté (target, assets lourds, secrets, `squashfs-root/`).
- [ ] `rustfmt.toml` présent.

## Workspace

- [ ] `Cargo.toml` racine avec members et `[workspace.package]`.
- [ ] crates `core`, `io`, `ui-egui`, `apps/<slug>`.
- [ ] `core` sans dépendance UI/disque.
- [ ] `cargo check --workspace` vert.
- [ ] un test minimal passe.
- [ ] `--sample` ou données de démo prévues.

## GitHub

- [ ] permission workflow `write` si release automatique.
- [ ] workflow CI avec fmt/clippy/test.
- [ ] premier push sur `main`.
- [ ] première CI verte.
- [ ] topics et description du dépôt.

## Gate

- [ ] aucun secret ni asset lourd dans le premier commit.
- [ ] `git log --stat` relu.
- [ ] CI verte avant tout tag.
