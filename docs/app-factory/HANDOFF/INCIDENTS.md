# Incidents et pièges rencontrés

## `/tmp` réinitialisé

Le toolchain Rust, `CARGO_HOME`, `RUSTUP_HOME` et `CARGO_TARGET_DIR` peuvent
disparaître entre deux sessions.

Symptômes : `rustc: command not found`, `rustup could not choose a version`.

Correctif : réinstaller (voir `WORKFLOW.md`) puis `cargo fetch`.

## Disque plein et `SIGBUS` du linker

Un build local a échoué avec :

```text
collect2: fatal error: ld terminated with signal 7 [Bus error]
```

Cause : `/tmp` à 100 %, `debug/build/<app>` contenant des binaires dupliqués de
~300 Mo et `debug/incremental`.

Correctif :

```bash
df -h /tmp
du -sh /tmp/cargo-target/debug/build/* | sort -h | tail
rm -rf /tmp/cargo-target/debug/build/<app-terminee>
rm -rf /tmp/cargo-target/debug/incremental
```

Ne pas supprimer `debug/deps` ni les artefacts partagés eframe/egui pendant un
build actif.

## `libxkbcommon-x11.so` manquant

Au lancement d’une app :

```text
Library libxkbcommon-x11.so could not be loaded
```

Installer la lib runtime :

```bash
sudo apt-get install -y libxkbcommon-x11-0
```

## Sessions d’agents en double

Des sessions MVP en double ont existé (DiagramCraft, ProjectCraft, MathCraft,
MasterCraft, FormCraft). Elles ont convergé vers les mêmes commits, mais il ne
faut **pas** les relancer en parallèle.

Une seule session de productisation par app est listée dans `WORKFLOW.md`.

## `pkill -f` qui tue son propre shell

Ne jamais lancer depuis une commande qui contient la chaîne :

```bash
pkill -f "Xvfb :99"
```

Cela peut tuer le shell courant. Utiliser un PID stocké dans `/tmp`.

## Warnings GitHub Actions Node.js 20

Les annotations `actions/checkout@v4` / `upload-artifact@v4` Node.js 20 sont des
avertissements, pas des échecs. Ne pas bloquer une release pour cela.

## Nom d’archive Windows

Le launcher unofficial exige :

```text
<id>-<version>-windows-x64-portable.zip
```

Un `.zip` sans `-portable` est ignoré. Les workflows ont été corrigés.

## `rust-version` du générateur

`tools/new-app.py` a déjà eu un bug : `version.workspace = true` remplaçait
aussi `rust-version.workspace = true`, donnant `rust-version = "0.1.0"`.
Corrigé dans CraftKit ; le tag `v0.1.0` de la bibliothèque reste utilisable.

## Cloud Shell / apt

`apt-get` peut être verrouillé par un autre processus ou perdre ses paquets
après une réinitialisation. Attendre le processus, puis réinstaller si besoin :
`xvfb`, `x11-apps`, `imagemagick`, `libxkbcommon-x11-0`, `libasound2-dev`
(pour un futur playback audio).

## Releases bloquées / en cours

Toujours vérifier :

```bash
gh run list --repo emilio-n8n/<slug> --workflow Release --limit 1
gh release view v0.1.0 --repo emilio-n8n/<slug>
```

Un workflow `in_progress` n’est pas un échec. Ne pas recréer le tag.

## Runners macOS indisponibles (23 novembre 2026)

Deux releases StreamCraft ont échoué car GitHub n’a pas pu acquérir un runner
macOS (`The job was not acquired by Runner of type hosted`) : d’abord macOS
x86_64, puis macOS arm64. Linux, Windows et l’autre macOS avaient réussi.

- Ce n’est pas une erreur de code ni de workflow.
- Relancer uniquement les jobs échoués :
  `gh run rerun <run-id> --failed`.
- Ne pas recréer le tag tant qu’aucune release n’est publiée. Si la release
  n’existe pas et que le tag doit porter un correctif, le déplacer est possible
  (`git tag -d v…`, `git push origin :refs/tags/v…`, re-tag, re-push).
- Un job `cancelled` dans la matrice empêche `checksums` et `release`, même
  avec `fail-fast: false`.

## Cache Cargo et artefacts de release

L’artefact Linux de StreamCraft `v0.2.1` (run 45200551665) se comportait comme
le binaire précédent : 73 frames / 2,43 s pour 10 s, alors que le code tagué
contenait la duplication de frames. Le cache `Swatinem/rust-cache` a
probablement restauré un binaire que Cargo a considéré à jour après le
déplacement du tag.

- Un simple déplacement de tag ne garantit pas une reconstruction.
- Bumper la version (`Cargo.toml` + `Cargo.lock`) force la recompilation de la
  crate applicative ; c’est la méthode retenue pour `v0.2.2`.
- Vérifier le comportement du binaire publié, pas seulement le SHA du run.
