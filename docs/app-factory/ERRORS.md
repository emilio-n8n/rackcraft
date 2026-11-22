# Catalogue d'erreurs et correctifs

Erreurs génériques rencontrées ou probables lors de la création d'une app
desktop. Chercher le symptôme exact avant de changer l'architecture : la plupart
de ces blocages sont environnementaux ou de packaging, pas des bugs du métier.

## 1. Toolchain et environnement

### `rustup could not choose a version of rustc to run`

Le répertoire de toolchain a disparu (souvent `/tmp` vidé) ou `RUSTUP_HOME`
n'est pas exporté.

```bash
ls /tmp/cargo-home/bin/rustc /tmp/rustup-home 2>/dev/null
# si absent, réinstaller (voir ENVIRONMENT.md)
```

Toujours exporter les quatre variables dans chaque nouveau shell :

```bash
export CARGO_HOME=/tmp/cargo-home RUSTUP_HOME=/tmp/rustup-home
export PATH=/tmp/cargo-home/bin:$PATH CARGO_TARGET_DIR=/tmp/cargo-target
```

### `error: failed to parse manifest ... feature 'edition2024' is required`

La toolchain est trop ancienne pour l'édition 2024 ou pour la MSRV du projet.

```bash
rustc --version
rustup update stable
```

Si la MSRV est imposée par une dépendance GUI, vérifier sa documentation avant
de la baisser : une MSRV trop basse casse la résolution.

### `No space left on device`

Le disque est plein, souvent à cause de `target/` ou de plusieurs jeux de
dépendances.

```bash
df -h /tmp
du -sh /tmp/cargo-target
du -sh /tmp/cargo-target/debug/* | sort -h | tail
rm -rf /tmp/cargo-target/debug/incremental
```

Ne pas lancer `cargo clean` sans nécessité : il force la recompilation de toute
la pile GUI (plusieurs minutes). Si le disque reste plein, supprimer les anciens
profils ou les cibles inutilisées.

### Le linker est tué (`Killed`, signal 9, `ld terminated`)

Mémoire insuffisante pendant le link, fréquent avec les profils debug et
beaucoup de code génériques.

- réduire la parallélisation : `CARGO_BUILD_JOBS=1` ;
- linker plus léger : `-C link-arg=-fuse-ld=lld` (si `lld` installé) ;
- désactiver le debug info : `[profile.dev] debug = 0` ;
- ne pas builder la release complète en local.

### Téléchargement crates.io qui bloque

Réseau filtré ou lent.

```bash
export CARGO_NET_RETRY=5
export CARGO_HTTP_TIMEOUT=120
cargo fetch --locked
```

Si un miroir est imposé, le configurer dans `~/.cargo/config.toml` (hors dépôt).
Ne jamais committer de token de registre.

### `failed to select a version for ...` après ajout d'une dépendance

Deux crates exigent des versions majeures incompatibles d'une même dépendance
(ex. deux versions d'egui via deux plugins).

```bash
cargo tree -d
cargo tree -i <crate>
```

Choisir un seul framework UI et une seule version majeure de chaque dépendance
lourde. Éviter d'ajouter un plugin qui traîne sa propre copie du framework.

## 2. Bibliothèques système

### `pkg-config` introuvable ou `Package ... was not found`

```text
The system library `xkbcommon` required by crate `xkbcommon-sys` was not found.
```

Installer les en-têtes de développement, pas seulement le runtime :

```bash
sudo apt-get install -y pkg-config libxkbcommon-dev libwayland-dev \
  libx11-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxcursor-dev libxrandr-dev libxi-dev \
  libgl1-mesa-dev libfontconfig1-dev libfreetype6-dev
```

### `Library libxkbcommon-x11.so could not be loaded` (au lancement)

Le build a réussi, mais la bibliothèque **runtime** manque :

```bash
sudo apt-get install -y libxkbcommon-x11-0
```

Même logique pour d'autres `-sys` : le paquet `-dev` sert à compiler, le paquet
runtime à exécuter.

### `error while loading shared libraries: lib...so: cannot open shared object`

```bash
ldd ./<app> | grep 'not found'
```

Installer la bibliothèque correspondante ou l'embarquer dans l'AppImage. Ne pas
corriger en modifiant `LD_LIBRARY_PATH` dans un script utilisateur : le binaire
doit être autonome ou documenter ses prérequis.

### Audio/GPU : backend absent

- audio Linux : `libasound2-dev`, PipeWire/JACK selon le backend ;
- GPU : `libvulkan-dev`, `mesa-vulkan-drivers` ;
- si aucun périphérique n'existe (CI, conteneur), l'app doit démarrer en mode
  dégradé ou refuser proprement, pas paniquer.

## 3. GUI et exécution headless

### `XOpenDisplayFailed` / `winit ... XNotSupported`

Aucun serveur X. En local headless :

```bash
sudo apt-get install -y xvfb
Xvfb :99 -screen 0 1440x920x24 >/tmp/xvfb.log 2>&1 &
export DISPLAY=:99
```

En CI, utiliser `xvfb-run` pour les smoke tests. Les tests unitaires ne doivent
pas ouvrir de fenêtre.

### L'app se lance puis se ferme immédiatement

Lire le log, pas seulement le code de sortie.

```bash
LIBGL_ALWAYS_SOFTWARE=1 ./target/debug/<app> --sample 2>&1 | head -40
```

Causes fréquentes : police/asset manquant, panique dans l'init, argument CLI
inconnu, erreur de chargement de document avalée par un `unwrap_or_default`.

### Rendu noir ou vide sous Xvfb

Forcer le rendu logiciel :

```bash
export LIBGL_ALWAYS_SOFTWARE=1
```

Vérifier aussi que la capture est prise après le premier rendu (attendre 3–5 s)
et que la fenêtre est bien celle de l'app (`xdotool search --name`).

### Capture d'écran impossible

```bash
sudo apt-get install -y x11-apps imagemagick
xwd -root -out /tmp/app.xwd
convert /tmp/app.xwd /tmp/app.png
```

Si `xwd` n'existe pas, installer `x11-apps`. Si `convert` manque, installer
`imagemagick`.

### Taille de fenêtre / HiDPI incohérente

Tester avec un facteur d'échelle :

```bash
WINIT_X11_SCALE_FACTOR=2 ./target/debug/<app>
```

Ne pas coder des tailles en pixels physiques : utiliser les points egui et
vérifier les tailles minimales.

## 4. Compilation et évolution des APIs

### Une API du framework UI a changé

Symptôme : `method not found`, `trait ... is not implemented`, `expected ...`.

Ne pas deviner à partir de tutoriels : lire la source de la version installée.

```bash
ls /tmp/cargo-home/registry/src/*/egui-*/
grep -rn "pub trait App" /tmp/cargo-home/registry/src/*/eframe-*/src/
```

Exemple réel : eframe 0.37 remplace `App::update` par `App::ui` et
`SidePanel`/`TopBottomPanel` par `Panel`. Ces changements cassent tous les
exemples plus anciens.

### `the trait bound ... is not satisfied` sur un type de rectangle/vecteur

Deux types homonymes (UI vs métier). Aliaser explicitement :

```rust
use my_core::Rect as DocRect;
```

Ne pas convertir à l'aveugle : vérifier les unités et l'origine (Y vers le haut
vs vers le bas).

### `cannot find attribute ...` / feature manquante

Une macro derive ou une fonctionnalité est derrière un flag :

```toml
serde = { version = "1", features = ["derive"] }
```

Vérifier `cargo tree -e features` avant d'ajouter une dépendance.

### Build local OK, CI rouge sur `clippy -D warnings`

Le CI compile tous les targets, pas seulement la lib :

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Corriger les warnings, ne pas les `allow` globalement. Un `#[allow]` doit être
local et justifié.

### Build local OK, CI rouge sur `fmt`

```bash
cargo fmt --all
cargo fmt --all -- --check
```

Le formatage doit être commité, pas seulement exécuté.

## 5. Git et GitHub

### `403 Resource not accessible by integration` à la publication

Le token Actions n'a pas le droit d'écrire une release.

```yaml
permissions:
  contents: write
```

et vérifier la permission par défaut du dépôt (voir RELEASE.md).

### `tag already exists`

Un tag publié ne se déplace pas.

```bash
git tag -d v0.1.0
git push origin :refs/tags/v0.1.0   # seulement si la release n'existe pas
```

Si la release existe déjà : incrémenter la version et publier un correctif.

### `remote: error: File ... is ...; this exceeds GitHub's file size limit`

Un asset lourd (vidéo, modèle, archive) a été commité.

```bash
git rm --cached <fichier>
```

Puis ajouter l'extension au `.gitignore`. Si le fichier est déjà dans
l'historique, réécrire l'historique (BFG/`git filter-repo`) **avant** de
pousser, ou repartir d'un dépôt propre. Ne jamais committer de secret : le
révoquer, pas seulement le supprimer.

### `detached HEAD` après checkout d'un tag

Créer une branche avant de committer :

```bash
git switch -c fix/<sujet>
```

### Fins de ligne et permissions

- `.gitattributes` pour forcer `text=auto` et éviter les diffs CRLF ;
- le bit exécutable n'est pas toujours conservé dans un ZIP Windows : fournir
  un `.tar.gz` pour Linux/macOS ;
- les noms de fichiers sensibles à la casse cassent entre Linux et macOS.

## 6. Packaging et distribution

### `version 'GLIBC_2.xx' not found` sur une distribution plus ancienne

Le binaire a été compilé sur un runner trop récent. Épingler le runner Linux
(voir RELEASE.md) ou compiler dans un conteneur avec l'ancienne glibc.

```bash
objdump -T <binaire> | grep GLIBC | sort -Vu | tail
```

### AppImage ne démarre pas : `FUSE` manquant

```bash
sudo apt-get install -y libfuse2
# ou, sans FUSE :
./<app>.AppImage --appimage-extract-and-run
```

Tester les deux chemins. Documenter le prérequis FUSE 2.

### AppImage : `AppRun` ou `.desktop` manquant

L'AppDir doit contenir :

```text
AppRun
<app>.desktop
<app>.png
usr/bin/<app>
usr/lib/...
```

Le `.desktop` doit avoir `Exec`, `Icon`, `Name`, `Categories`. Vérifier avec
`--appimage-extract` **hors du dépôt**.

### Windows : console noire au lancement

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

Garder la console en debug pour voir les erreurs.

### macOS : « application endommagée » ou bloquée par Gatekeeper

Un binaire non signé/non notarié téléchargé est mis en quarantaine.

```bash
xattr -dr com.apple.quarantine <app>.app
```

Pour une distribution publique, prévoir signature Developer ID + notarisation.
Ne pas promettre une absence d'avertissement sans ces étapes.

### L'archive ne contient pas le binaire ou perd le bit exécutable

Vérifier le contenu après upload, pas seulement le job :

```bash
tar -tzf <archive>.tar.gz | head
unzip -l <archive>.zip | head
```

Un `strip` ou un chemin de sortie erroné produit une archive valide mais vide.

## 7. Performance et plantages runtime

### L'UI gèle à l'ouverture d'un gros fichier

Le parsing se fait dans `ui()`. Déplacer le travail dans un worker, afficher une
progression, et ne rafraîchir l'UI qu'à la réception du résultat.

### `memory allocation of N bytes failed` / OOM

- charger les gros assets paresseusement ;
- borner les caches (textures, vignettes, conversions) ;
- éviter de cloner le document complet à chaque frame ;
- undo par snapshots : borner le nombre d'états et la taille.

Tester avec un fichier réaliste, pas seulement une fixture de 2 Ko.

### Panique dans un parseur sur un fichier utilisateur

```text
thread 'main' panicked at ... unwrap() on None
```

Règle : aucun `unwrap`/`expect` sur une entrée externe. Retourner une erreur
`thiserror` avec le fichier et l'entrée concernée. Un élément corrompu doit être
ignoré avec un warning, pas faire tomber l'app.

### Conversion très lente au démarrage

Différer la conversion au premier affichage/placement, mettre en cache le
résultat, et afficher un indicateur. Ne pas convertir 200 assets pour n'en
afficher qu'un.

### Débordement numérique / coordonnées négatives

- utiliser `f64` pour les unités métier, `f32` seulement à l'écran ;
- clamper les valeurs importées après validation ;
- tester les valeurs extrêmes (0, négatif, NaN, infini) dans les parseurs.

## 8. Tests

### Tests verts en local, rouges en CI

Causes fréquentes : fuseau horaire, locale, casse des noms de fichiers, absence
de police, accès réseau, chemin absolu codé en dur.

- ne pas dépendre du réseau dans les tests ;
- utiliser des `tempdir` ;
- comparer des dates en UTC ;
- éviter les sleeps : attendre un événement ou utiliser une horloge injectée.

### Test flaky sur des flottants

```rust
assert!((a - b).abs() < 1e-6);
```

Les conversions f32↔f64 et les approximations de courbes perdent de la
précision. Fixer une tolérance explicite par domaine.

### Test qui ouvre une fenêtre ou un périphérique audio en CI

Séparer la logique pure de l'initialisation. Les tests du cœur ne doivent
demander ni display, ni audio, ni GPU.

### `cargo test` très lent

- lancer d'abord les crates cœur/io ;
- garder les tests d'intégration lourds derrière `#[ignore]` et les lancer
  manuellement ;
- ne pas recompiler toute la pile GUI pour tester de la logique pure.

## 9. Données et formats

### Fichier ZIP malveillant ou énorme

- borner la taille décompressée, pas seulement la taille du fichier ;
- borner le nombre d'entrées ;
- refuser les chemins absolus et `..` (path traversal) ;
- ne pas extraire automatiquement toute l'archive dans un dossier utilisateur.

### Encodage et BOM

Un CSV/JSON avec BOM UTF-8 casse certain parseurs. Détecter et retirer
`EF BB BF`, ou utiliser un lecteur qui le gère. Tester avec des fichiers
produits par Excel.

### Migration de schéma oubliée

Un ancien document s'ouvre puis se sauvegarde en perdant des champs.

- versionner le format dès la v0.1 ;
- ajouter `#[serde(default)]` pour les nouveaux champs ;
- écrire un test qui charge une fixture de l'ancienne version ;
- ne jamais changer silencieusement la signification d'un champ.

### Le document dépend d'un fichier externe

Si le rendu exige encore l'asset d'origine, le document n'est pas autonome.
Embarquer les données nécessaires (chemins vectoriels, textures, polices) ou
documenter clairement la dépendance.

## 10. Release et assets

### Un job de la matrice est `skipped`

Une condition `if:` mal placée (souvent `matrix.os == ...` avec la mauvaise
valeur) saute le job. Lire le log du job, pas seulement le résumé.

### `SHA256SUMS.txt` ne correspond pas

- checksums calculés avant un renommage ;
- asset remplacé après calcul ;
- fins de ligne ou préfixe `./` différents.

Toujours vérifier depuis un dossier vide :

```bash
sha256sum -c SHA256SUMS.txt
```

### La release contient les assets d'une ancienne version

Le workflow a téléchargé un artifact mis en cache ou a publié dans un tag
existant. Vérifier `github.ref_name`, le tag résolu et la liste des fichiers
affichée par le job « Publish ».

### `Node.js 20 is deprecated` dans les logs

Avertissement des actions GitHub, pas une erreur du projet. Mettre à jour les
actions (`checkout@v4` → version courante) quand c'est possible. Ne pas bloquer
une release pour cet avertissement seul.

## 11. Sécurité

- aucun secret dans le dépôt, les logs ou les captures ;
- secrets fournis par variables d'environnement ou gestionnaire système ;
- URLs externes ouvertes via le navigateur, pas via un shell ;
- pas de `unsafe` sans justification écrite ;
- vérifier la licence de chaque dépendance et asset ;
- mettre à jour les dépendances pour les CVE, mais tester avant de publier.

## Méthode de diagnostic

1. Lire le **premier** message d'erreur, pas le dernier.
2. Reproduire avec la commande la plus petite possible.
3. Vérifier l'environnement avant le code : toolchain, disque, libs, display.
4. Vérifier la version de l'API installée avant de modifier le code.
5. Chercher dans ce catalogue le symptôme exact.
6. Corriger, puis ajouter un test ou une note pour ne pas répéter l'erreur.
