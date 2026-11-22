# Automatiser la création d'une application

Objectif : à partir d'un brief court, produire un dépôt propre, compilable et
testé, sans redécouvrir l'environnement ni répéter les erreurs du catalogue.

## 1. Brief d'entrée

Répondre à ces questions avant de générer quoi que ce soit. Une réponse vide est
une décision à trancher, pas un détail.

```text
Nom :
Slug (kebab-case) :
Domaine : image | audio | vidéo | 3D | CAD | documents | données | dev | autre
Utilisateur cible :
Problème résolu :
Promesse en une phrase :
Plateformes : linux | windows | macos | les trois
MVP (3 à 5 fonctions) :
Hors-scope :
Formats d'entrée :
Formats de sortie :
Volume de données typique :
Contraintes temps réel : oui/non
GPU nécessaire : oui/non
Réseau : offline | synchronisation | API
Persistance : JSON | binaire | SQLite | dossier projet
Undo : snapshots | commandes | aucun
Licence : MIT OR Apache-2.0 (défaut) | autre
Assets : libres | générés | fournis par l'utilisateur
Nom du binaire :
```

## 2. Arbre de décision technique

### Framework UI

| Situation | Choix |
| --- | --- |
| App desktop 2D, MVP rapide | egui/eframe |
| UI très custom, GPU | egui + wgpu, ou framework GPU dédié |
| App système légère | Slint ou Tauri selon l'écosystème |
| CLI uniquement | `clap` + `ratatui` si TUI |
| Web + desktop | Tauri (mais vérifier la taille et les dépendances) |

Ne pas mélanger deux frameworks UI dans le même binaire.

### Rendu

- 2D simple : `egui::Painter`, `tiny-skia`, ou `resvg` pour du SVG ;
- image : CPU par tuiles, GPU seulement si mesuré nécessaire ;
- 3D : `wgpu` avec un fallback d'erreur clair ;
- vidéo : décodage streaming, jamais tout en mémoire.

### Jobs

- calcul court (< 16 ms) : thread UI ;
- calcul long : worker + channel + progression + annulation ;
- audio temps réel : thread dédié, sans allocation ni lock ;
- batch : processus séparé si crash posible ou dépendance lourde.

### Persistance

- petit document : JSON + `serde` ;
- gros document : binaire versionné ou SQLite ;
- projet multi-fichiers : dossier avec manifest ;
- toujours : version de schéma + migration + test de round-trip.

### Undo

- MVP : snapshots bornés ;
- gros documents : commandes inverses ou diffs ;
- jamais : exposer un `document_mut()` sans passer par l'historique.

## 3. Prompt d'amorçage pour un agent

```text
Crée une nouvelle application desktop native en Rust.

Brief :
<coller le brief complété>

Contraintes :
- lire d'abord docs/app-factory/README.md, WORKFLOW.md, ENVIRONMENT.md,
  ARCHITECTURE.md et ERRORS.md ;
- workspace : crates/core, crates/io, crates/ui-egui, apps/<slug> ;
- dépendance core <- io <- ui <- app, sans cycle ;
- licence MIT OR Apache-2.0, README avec disclaimer de projet indépendant ;
- CARGO_TARGET_DIR=/tmp/cargo-target, toolchain dans /tmp ;
- pas de build release local ;
- fixtures générées ou libres de droits uniquement ;
- fmt + clippy -D warnings + tests workspace verts avant tout push.

Livrables de la première passe :
1. dépôt initialisé, fichiers de base, .gitignore ;
2. workspace qui compile avec un modèle minimal et un test ;
3. document de démonstration intégré (--sample) ;
4. CI fmt/clippy/test ;
5. README décrivant le MVP et les limites ;
6. docs/decisions.md avec les choix structurants.

Ne pas ajouter de fonctionnalité hors MVP. Ne pas créer de tag ni de release
avant que la CI soit verte et que je l'aie demandé.
```

## 4. Ordre d'exécution

1. **Brief** — compléter le questionnaire, valider le hors-scope.
2. **Préflight** — toolchain, disque, libs système du domaine.
3. **Dépôt** — licences, README, AGENTS, `.gitignore`, remote.
4. **Workspace** — crates vides qui compilent, `cargo check`.
5. **Cœur** — types, invariants, tests, `--sample`.
6. **I/O** — un format à la fois, fixtures, erreurs, limites.
7. **UI** — session, écrans, interactions, feedback.
8. **CI** — fmt, clippy, tests, cache.
9. **Release** — matrice, archives, checksums, vérification réelle.
10. **Durcissement** — perf sur cas réels, erreurs, docs, accessibilité.

Chaque étape se termine par une commande de vérification. Ne pas empiler
plusieurs étapes non vérifiées.

## 5. Squelette minimal

```text
<slug>/
  Cargo.toml
  Cargo.lock
  rustfmt.toml
  LICENSE-MIT
  LICENSE-APACHE
  README.md
  AGENTS.md
  CHANGELOG.md
  .gitignore
  .github/workflows/ci.yml
  .github/workflows/release.yml
  crates/core/
    Cargo.toml
    src/lib.rs
  crates/io/
    Cargo.toml
    src/lib.rs
  crates/ui-egui/
    Cargo.toml
    src/lib.rs
  apps/<slug>/
    Cargo.toml
    src/main.rs
  docs/
    decisions.md
    app-factory/   (copie ou lien vers ce playbook)
  samples/
    README.md
```

Racine `Cargo.toml` :

```toml
[workspace]
resolver = "2"
members = ["crates/core", "crates/io", "crates/ui-egui", "apps/<slug>"]
default-members = ["crates/core", "crates/io", "crates/ui-egui", "apps/<slug>"]

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

`apps/<slug>/src/main.rs` :

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let options = <slug>_ui::parse_cli()?;
    <slug>_ui::run(options)?;
    Ok(())
}
```

## 6. Definition of done d'une première version

- [ ] le brief est écrit et le hors-scope explicite ;
- [ ] le workspace compile sans warning ;
- [ ] `core` n'a aucune dépendance UI/disque ;
- [ ] un document de démonstration s'ouvre sans fichier externe ;
- [ ] import/export testés, fichiers corrompus gérés ;
- [ ] undo/redo testé (si présent) ;
- [ ] UI utilisable au clavier et à la souris ;
- [ ] opérations longues non bloquantes ;
- [ ] CI verte sur push ;
- [ ] release verte sur toutes les plateformes visées ;
- [ ] assets téléchargés, checksums vérifiés, binaire lancé ;
- [ ] README, CHANGELOG et licences à jour.

## 7. Ce qui doit rester manuel

- le choix du produit et du hors-scope ;
- les décisions de licence des assets et codecs ;
- la validation d'un cas d'usage réel ;
- la publication d'une release ;
- la suppression ou migration de données utilisateur.

Un agent peut préparer, proposer et exécuter ; ces décisions restent humaines.

## 8. Anti-patterns

- commencer par l'UI et découvrir le modèle en route ;
- ajouter 20 dépendances avant le premier test ;
- copier un template sans adapter le `.gitignore` (assets, binaires) ;
- créer un tag avant la première CI verte ;
- publier une release sans lancer le binaire téléchargé ;
- stocker un asset propriétaire dans le dépôt ;
- masquer une erreur de parsing avec `unwrap_or_default` ;
- bloquer le thread UI par un calcul long ;
- utiliser `f32` pour des unités métier ;
- promettre une compatibilité de format non testée.
