# Environnement de développement

## Contraintes Cloud Shell

Le terminal de référence a :

- ~5 Go de disque persistant dans `$HOME` (souvent moins libre) ;
- ~8 Go de RAM et 2 vCPU ;
- un `/tmp` plus large mais **non persistant** ;
- aucun affichage graphique natif ;
- réseau sortant parfois filtré ;
- GitHub CLI authentifié.

Conséquences :

- ne jamais construire dans `$HOME` ;
- cloner les références uniquement dans `/tmp/refs` ;
- ne pas committer de binaires, caches, assets lourds ou secrets ;
- déléguer les builds release à GitHub Actions ;
- réserver Xvfb aux tests visuels courts.

## Toolchain Rust dans `/tmp`

```bash
export CARGO_HOME=/tmp/cargo-home
export RUSTUP_HOME=/tmp/rustup-home
export PATH=/tmp/cargo-home/bin:$PATH
export CARGO_TARGET_DIR=/tmp/cargo-target
```

Installation si la toolchain n'existe pas :

```bash
mkdir -p /tmp/cargo-home /tmp/cargo-target /tmp/rustup-home
export RUSTUP_HOME=/tmp/rustup-home CARGO_HOME=/tmp/cargo-home
curl -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
sh /tmp/rustup-init.sh -y --profile minimal --no-modify-path
export PATH=/tmp/cargo-home/bin:$PATH
rustup component add rustfmt clippy
rustc --version && cargo --version
```

Fichier de session :

```bash
cat > /tmp/rust-env.sh <<'EOF'
export CARGO_HOME=/tmp/cargo-home
export RUSTUP_HOME=/tmp/rustup-home
export PATH=/tmp/cargo-home/bin:$PATH
export CARGO_TARGET_DIR=/tmp/cargo-target
EOF
source /tmp/rust-env.sh
```

### Réinitialisation de `/tmp`

Une nouvelle session peut vider `/tmp`. Symptômes :

```text
rustup could not choose a version of rustc to run
/tmp/cargo-home: No such file or directory
/tmp/cargo-target: No such file or directory
```

Relancer l'installation ci-dessus. `Cargo.lock` garantit les mêmes versions de
dépendances ; la recompilation est simplement plus longue.

## Bibliothèques système

### Base GUI (egui/eframe, rfd, winit)

```bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  build-essential pkg-config \
  libxkbcommon-dev libxkbcommon-x11-0 libxkbcommon-x11-dev \
  libwayland-dev libx11-dev libxcb1-dev libxcb-render0-dev \
  libxcb-shape0-dev libxcb-xfixes0-dev \
  libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev \
  libfontconfig1-dev libfreetype6-dev
```

Au lancement, l'erreur suivante signifie qu'il manque la lib **runtime** :

```text
Library libxkbcommon-x11.so could not be loaded
```

```bash
sudo apt-get install -y libxkbcommon-x11-0
```

### Selon le domaine

- **audio** : `libasound2-dev`, `libpipewire-0.3-dev`, `libjack-jackd2-dev`
  selon la pile ; vérifier le backend choisi.
- **image** : `libpng-dev`, `libjpeg-dev`, `libwebp-dev`, éventuellement
  LittleCMS pour la colorimétrie.
- **vidéo** : `ffmpeg`, `libavcodec-dev`, `libavformat-dev`, `libavutil-dev`,
  `libswscale-dev` si FFmpeg est utilisé (attention à la licence).
- **3D/GPU** : `mesa-vulkan-drivers`, `libvulkan-dev`, pilotes GPU.
- **PDF/impression** : `libcairo2-dev` uniquement si une dépendance système
  l'exige ; préférer une crate Rust pure.
- **tests visuels** : `xvfb`, `x11-apps`, `imagemagick`, `xdotool`.

## Tests headless d'une UI

```bash
Xvfb :99 -screen 0 1440x920x24 >/tmp/xvfb.log 2>&1 &
export DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1

./target/debug/<app> --sample >/tmp/app.log 2>&1 &
APP_PID=$!
sleep 5
xwd -root -out /tmp/app.xwd
convert /tmp/app.xwd /tmp/app.png
kill "$APP_PID"
```

Ou :

```bash
LIBGL_ALWAYS_SOFTWARE=1 timeout 8s xvfb-run -a \
  -s "-screen 0 1440x920x24" ./target/debug/<app> --sample
```

- Code 124 = l'app est restée ouverte (attendu).
- Un crash imprime une erreur avant le timeout.
- La capture permet de vérifier le rendu sans écran physique.

### Piège `pkill -f`

Ne pas lancer :

```bash
pkill -f '<app> --sample'
```

si la commande shell elle-même contient cette chaîne : elle peut tuer son propre
shell. Préférer un PID dans `/tmp/app.pid` ou un motif restreint.

## Disque et cache

```bash
df -h /tmp
du -sh /tmp/cargo-target
du -sh /tmp/cargo-target/debug/* | sort -h | tail
```

En cas d'urgence :

```bash
rm -rf /tmp/cargo-target/debug/incremental
rm -rf /tmp/refs
```

Un `cargo clean` complet n'est pas recommandé en Cloud Shell : les dépendances
de GUI/GPU mettent longtemps à recompiler.

## Tests locaux raisonnables

```bash
cargo check -p <crate-core> -p <crate-io>
cargo test  -p <crate-core> -p <crate-io>
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Éviter :

```bash
cargo build --release --workspace
```

Un build debug local suffit pour une capture. Les binaires distribués sont
construits par la CI.

## Partager une UI locale

Pour une démo à distance :

```bash
curl -fL https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 \
  -o /tmp/cloudflared
chmod +x /tmp/cloudflared
/tmp/cloudflared tunnel --no-autoupdate --url http://127.0.0.1:8080
```

- URL `*.trycloudflare.com` publique et temporaire.
- Ne jamais exposer sans auth une UI qui contrôle des fichiers ou un agent.
- Pour une URL stable : tunnel Cloudflare nommé avec domaine/token.
- Ne jamais committer le token.
