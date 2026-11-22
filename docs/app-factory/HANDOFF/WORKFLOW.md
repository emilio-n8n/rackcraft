# Runbook

## Environnement obligatoire

```bash
export CARGO_HOME=/tmp/cargo-home
export RUSTUP_HOME=/tmp/rustup-home
export PATH=/tmp/cargo-home/bin:$PATH
export CARGO_TARGET_DIR=/tmp/cargo-target
```

`/tmp` n’est pas persistant. Si `rustc` manque :

```bash
mkdir -p /tmp/cargo-home /tmp/rustup-home /tmp/cargo-target
export RUSTUP_HOME=/tmp/rustup-home CARGO_HOME=/tmp/cargo-home
curl -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
sh /tmp/rustup-init.sh -y --profile minimal --no-modify-path
export PATH=/tmp/cargo-home/bin:$PATH
rustup default stable
rustup component add rustfmt clippy
```

Ne jamais builder une release complète en local. Les binaires distribués sont
construits par GitHub Actions, y compris pour les apps.

## Qualité d’une app

```bash
cd /home/emilio_developpeur/<slug>
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
python3 tools/launcher_check.py .
```

## Vérifier une app

```bash
git -C /home/emilio_developpeur/<slug> pull --ff-only
git -C /home/emilio_developpeur/<slug> status --short
gh run list --repo emilio-n8n/<slug> --limit 3
```

## Releases

Un tag `v0.1.0` a déjà été poussé sur les 10 apps. Ne pas réutiliser un tag
publié. Pour une nouvelle version :

```bash
git -C /home/emilio_developpeur/<slug> pull --ff-only
git -C /home/emilio_developpeur/<slug> tag -a v0.2.0 -m "<App> v0.2.0"
git -C /home/emilio_developpeur/<slug> push origin v0.2.0
```

Vérifier ensuite :

```bash
gh run list --repo emilio-n8n/<slug> --workflow Release --limit 1
gh release view v0.2.0 --repo emilio-n8n/<slug>
```

Les workflows publics n’ont pas de limite de minutes GitHub Actions standard.

## Launcher check

Notre validateur local :

```bash
python3 tools/launcher_check.py .
python3 tools/launcher_check.py . --release-dir dist
```

Le vrai vérificateur du **Craft Unofficial Launcher** :

```bash
git clone --depth 1 https://github.com/YalKouassi/craft-unofficial-launcher.git \
  /tmp/refs/craft-unofficial-launcher
cd /tmp/refs/craft-unofficial-launcher
cargo run -q -p craft-release-check -- dir /chemin/dist \
  --repo emilio-n8n/<slug> --tag v0.1.0
```

Après publication :

```bash
cargo run -q -p craft-release-check -- github \
  --repo emilio-n8n/<slug> --tag v0.1.0
```

## Vagues d’agents

- Maximum **5 sous-agents simultanés**.
- Modèle : `opencode-go/hy4-flash-free`.
- Un agent = un dépôt.
- Ne pas réutiliser les sessions en double des MVP : une seule session par app.
- Les prompts de productisation sont dans `PROMPTS/`.

### Sessions de productisation vague A

| App | Session |
| --- | --- |
| DiagramCraft | `ses_dfbd118f0ffeSsfjz18tllRXRV` |
| ProjectCraft | `ses_dfbd118e4ffeIoAhDwCoz22Xs3` |
| FormCraft | `ses_dfbd118acffen0wyLUvUEMoxWt` |
| MathCraft | `ses_dfbd118a0ffe2cWJOBYDh0rEY2` |
| MasterCraft | `ses_dfbd11884ffe8FdWG81bOgFu4W` |

### Sessions de productisation vague B (lancées)

| App | Session |
| --- | --- |
| CircuitCraft | `ses_dfba3a723ffeXoD9LipEYl1iAe` |
| GeoCraft | `ses_dfba3a720ffecf0RUkKEJXx11q` |
| MusicCraft | `ses_dfba3a70effehu7sWKCqt7WTgQ` |
| StreamCraft | `ses_dfba3a70affeoebUmv5Qv5Uvi2` |
| SimCraft | `ses_dfba3a706ffe7Q5ZBUuq49i1I8` |

Prompts : `PROMPTS/<slug>-product.md`.

## Disque

```bash
df -h /tmp
du -sh /tmp/cargo-target
du -sh /tmp/cargo-target/debug/build/* | sort -h | tail
```

Nettoyage sûr des artefacts d’apps terminées :

```bash
rm -rf /tmp/cargo-target/debug/build/<slug>
rm -rf /tmp/cargo-target/debug/incremental
```

Ne pas supprimer `debug/deps` ni les artefacts partagés egui/eframe pendant
qu’un agent compile.

## Vérification visuelle

```bash
sudo apt-get install -y libxkbcommon-x11-0 xvfb x11-apps imagemagick
Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 & echo $! >/tmp/xvfb.pid
DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 /tmp/cargo-target/debug/<slug> --sample &
sleep 4
DISPLAY=:99 xwd -root -out /tmp/<slug>.xwd
convert /tmp/<slug>.xwd /tmp/<slug>.png
```

Ne jamais utiliser `pkill -f "Xvfb :99"` depuis une commande qui contient cette
chaîne : cela peut tuer le shell lui-même. Utiliser un PID.
