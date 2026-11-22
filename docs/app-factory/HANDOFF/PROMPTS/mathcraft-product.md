# MathCraft — productisation v0.2

Tu reprends MathCraft (`/home/emilio_developpeur/mathcraft`) après la tranche 1.
Objectif : un cahier de calcul scientifique utilisable, pas un clone de MATLAB.

## Lecture

- `README.md`, `CHANGELOG.md`, `AGENTS.md`, `docs/decisions.md`
- `src/document.rs`, `src/engine/`, `src/app.rs`
- Les limites connues du rapport de tranche 1.

## Priorités

1. **Feuilles de calcul** : plusieurs expressions/feuilles par document,
   résultats persistés, sélection d'une feuille.
2. **Variables et portée** : édition stable des variables, persistance,
   références entre expressions, messages d'erreur clairs.
3. **Tracé** : zoom/pan, plage X/Y éditable, plusieurs courbes, export PNG ou
   SVG.
4. **Import/export CSV** : importer des séries de points, exporter résultats et
   variables.
5. **Confort** : historique cliquable, aide syntaxique, exemples intégrés,
   gestion des erreurs sans panique.

## Qualité

- Préserver le format `mathcraft`, le schéma 1 et le manifeste launcher.
- Tests : feuilles multiples, portée des variables, plages de tracé, CSV,
  round-trip, undo.
- README/CHANGELOG/ADR à jour.
- Pas de tag/release/sous-agent ; `craftkit` intact.
- fmt, clippy `-D warnings`, tests, push `main`, CI verte.

## Rapport

Hash, CI, fonctions, limites.
