# Workflow complet de création

Ce document décrit le cycle réutilisable, indépendamment du domaine. Chaque
phase a un **gate** : on ne passe pas à la suivante sans preuve écrite ou
commande verte.

## Phase 0 — Cadrage

Objectif : éviter de construire la mauvaise application.

### Brief produit (1 page)

- **Utilisateurs** : qui, dans quel contexte, quelles compétences ?
- **Problème** : quelle tâche actuelle est lente, chère, fragile ?
- **Promesse** : une phrase, par exemple « éditer X hors ligne sans Y ».
- **Plateformes** : Linux d'abord ? Windows/macOS obligatoires ? Web ?
- **MVP** : 3 à 5 fonctions, pas 20.
- **Hors-scope** : ce qu'on refuse explicitement dans cette version.
- **Entrées/sorties** : formats de fichiers, import/export, presse-papiers, drag & drop.
- **Volume de données** : quelques Mo, centaines de Mo, flux temps réel ?
- **Réseau** : offline-first, synchronisation, API distante ?
- **Licences** : assets, polices, codecs, stencils, samples audio/image.
- **Accessibilité/ergonomie** : raccourcis, thème, taille de police.
- **Budget matériel** : RAM/CPU/GPU minimum.

### Décisions d'architecture initiales

- UI : egui/eframe, Slint, Tauri, terminal (`ratatui`) ?
- Rendu : CPU, wgpu, GPU fragment/offscreen ?
- Jobs lourds : thread pool, processus séparé, service ?
- Persistance : JSON, binaire custom, SQLite, projet en dossier ?
- Undo : commandes inverses ou snapshots ?
- Plugins : nécessaires au MVP ou non ?

### Gate 0

- [ ] Brief relu, hors-scope explicite.
- [ ] Maquette filaire en texte des 3 écrans principaux.
- [ ] Liste des dépendances lourdes et de leurs licences.
- [ ] Critère de succès mesurable (temps, export, nombre de cas supportés).

## Phase 1 — Environnement reproductible

Voir [ENVIRONMENT.md](ENVIRONMENT.md).

- toolchain Rust dans un emplacement avec assez de disque ;
- `CARGO_TARGET_DIR` sur un volume adapté ;
- libs systèmes pour la GUI compilées **avant** de coder l'UI ;
- test headless disponible (Xvfb) ou stratégie de test sans fenêtre ;
- note des versions : Rust, eframe/egui, OSCI, etc.

### Gate 1

```bash
rustc --version
cargo --version
echo "$CARGO_TARGET_DIR"
cargo new /tmp/smoke-app
cd /tmp/smoke-app && cargo run -q
```

## Phase 2 — Dépôt et cadre juridique

Voir [BOOTSTRAP.md](BOOTSTRAP.md).

- dépôt public **vide** ;
- `LICENSE-MIT`, `LICENSE-APACHE`, README et disclaimer d'indépendance ;
- `.gitignore` Rust complet ;
- AGENTS.md avec règles d'environnement et conventions ;
- premier commit sur `main`.

### Gate 2

- [ ] `gh repo view` confirme le dépôt public.
- [ ] `git remote -v` pointe le bon `origin`.
- [ ] README contient le disclaimer.
- [ ] aucun secret, asset lourd, `target/` ou fichier personnel.

## Phase 3 — Squelette du workspace

Voir [ARCHITECTURE.md](ARCHITECTURE.md).

Structure générique :

```
crates/core        modèle et logique métier, sans UI/I/O
crates/io          formats, disque, réseau, exports
crates/ui-egui     présentation native
apps/<slug>        binaire, flags CLI
```

Pour un domaine audio, on peut nommer `crates/audio-engine`, etc. Garder la
règle de dépendance `core <- io <- ui <- app`.

### Gate 3

```bash
cargo check --workspace
cargo test --workspace
```

- [ ] les crates se compilent ;
- [ ] `core` n'a aucune dépendance UI/disque ;
- [ ] une doc d'architecture courte est écrite.

## Phase 4 — Cœur métier d'abord

Objectif : rendre le domaine correct et testable sans interface.

### Méthode

1. Écrire les types et invariants.
2. Écrire les tests avant l'UI : cas nominal, limites, erreurs.
3. Fixer les unités, les ids, les invariants de sérialisation.
4. Implémenter undo/redo selon le choix (snapshots souvent plus simple).
5. Définir un jeu de données de démonstration intégré (`--sample`) sans
   dépendance réseau.

### Gate 4

- [ ] `cargo test -p core` vert.
- [ ] aucun `unwrap()` sur entrée utilisateur/externe.
- [ ] les tests couvrent undo/redo, sérialisation et cas limites.
- [ ] le cœur ne connaît ni chemins de fichiers ni contexte UI.

## Phase 5 — I/O, formats et persistance

- un module par format ;
- détection du format par signature, pas seulement l'extension ;
- parsing défensif : limites, erreurs actionnables, aucun panic ;
- fixtures générées et commitées, ou assets libres de droits ;
- migration de schéma dès la première version ;
- tests qui ouvrent réellement les fichiers produits.

### Gate 5

- [ ] import/export aller-retour testé ;
- [ ] fichier corrompu → erreur propre ;
- [ ] limites de taille/mémoire documentées ;
- [ ] licence de chaque fixture/assets vérifiée.

## Phase 6 — UI et interactions

Voir [UI-EGUI.md](UI-EGUI.md) pour egui.

- chaque action utilisateur passe par un état central (session) ;
- les mutations persistées passent par undo/redo ;
- les opérations longues ne bloquent pas le thread UI ;
- drag & drop, pan/zoom, recherche, dialogues fichiers ;
- feedback : status bar, notifications, barres de progression, annulation ;
- thème sombre/clair si pertinent ;
- raccourcis clavier documentés ;
- états vides et erreurs visibles.

### Gate 6

- [ ] test visuel manuel (capture) ;
- [ ] aucune action destructive sans confirmation si données non sauvées ;
- [ ] opérations lourdes annulables ou exécutées en arrière-plan ;
- [ ] l'UI ne parse pas les formats métier directement.

## Phase 7 — Packaging, CI et release

Voir [RELEASE.md](RELEASE.md).

- fmt, clippy -D warnings, tests ;
- CI sur push/PR ;
- release sur tag et workflow_dispatch ;
- Linux tar.gz + AppImage (si app desktop Linux) ;
- Windows zip, macOS ARM/Intel ;
- checksums ;
- assets testés après upload.

### Gate 7

- [ ] CI verte.
- [ ] tag poussé.
- [ ] release verte sur toutes les cibles.
- [ ] binaire Linux lancé, version attendue.
- [ ] AppImage testée (si fournie).
- [ ] `SHA256SUMS.txt` vérifié.

## Phase 8 — Durcissement et maintenance

- profiler mémoire/CPU sur les cas réels ;
- limiter les allocations dans les boucles critiques ;
- journalisation locale sans secrets ;
- mises à jour/deprecations de dépendances ;
- accessibilité (contraste, focus, raccourcis) ;
- documentation utilisateur et contribution ;
- issues de suivi et roadmap ;
- sauvegarde du contexte via [HANDOFF.md](HANDOFF.md).

## Boucle de développement d'une fonctionnalité

1. Décrire le comportement attendu en une phrase et les erreurs possibles.
2. Écrire ou ajuster le test du cœur (pas l'UI d'abord).
3. Implémenter dans le crate le plus profond possible.
4. Ajouter l'I/O si nécessaire, avec fixture.
5. Câbler l'UI.
6. `cargo fmt`, `clippy`, tests ciblés.
7. Test manuel/capture si UI.
8. Docs + changelog si visible.
9. Commit atomique.

## Journal de décisions (à tenir)

Créer `docs/decisions.md` dans chaque app :

```text
## ADR-001 — egui pour le MVP
Date :
Décision :
Alternatives :
Raisons :
Conséquences :
```

Cela évite de rediscuter les mêmes choix après compaction du contexte.
