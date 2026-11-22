# DiagramCraft — productisation v0.2

Tu reprends DiagramCraft (`/home/emilio_developpeur/diagramcraft`) après la
tranche 1. Objectif : passer d'un MVP à un **éditeur de diagrammes réellement
utilisable**, pas à un clone de Visio.

## Lecture

- `README.md`, `CHANGELOG.md`, `AGENTS.md`, `docs/decisions.md`
- `src/document.rs`, `src/app.rs`, `src/svg.rs`
- Les limites connues du rapport de tranche 1.

## Priorités

1. **Édition des labels** : double-clic ou Entrée sur une forme, édition en
   place, validation, une étape d'undo.
2. **Redimensionnement** : poignées aux coins, minimum de taille, une étape
   d'undo par drag.
3. **Sélection multiple** : Shift+clic, déplacement groupé, suppression groupée,
   copier/coller/dupliquer.
4. **Connecteurs utiles** : routage droit ou orthogonal au choix, flèches,
   suppression propre, ancrage sur les bords.
5. **Confort** : ajuster à la fenêtre, grille et snapping activables, titre du
   document éditable, annuler/rétablir visible, états vides.

## Qualité

- Préserver la sauvegarde JSON, le format `diagramcraft`, le schéma 1 et le
  manifeste launcher.
- Tests : édition de label, resize, multi-sélection, copy/paste, routage,
  round-trip JSON, undo.
- Mettre à jour README, CHANGELOG et ADR.
- Pas de tag, pas de release, pas de sous-agent, ne pas toucher à `craftkit`.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked`, puis push `main` et CI verte.

## Rapport

Hash, CI, fonctions livrées, limites restantes.
