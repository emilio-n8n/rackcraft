# ProjectCraft — productisation v0.2

Tu reprends ProjectCraft (`/home/emilio_developpeur/projectcraft`) après la
tranche 1. Objectif : un outil de planification utilisable pour de vrais petits
projets, pas un clone de MS Project.

## Lecture

- `README.md`, `CHANGELOG.md`, `AGENTS.md`, `docs/decisions.md`
- `src/document.rs`, `src/schedule.rs`, `src/app.rs`, `src/csv.rs`
- Les limites connues du rapport de tranche 1.

## Priorités

1. **Gantt interactif** : flèches de dépendance, zoom jour/semaine/mois,
   défilement horizontal, ajustement à la fenêtre.
2. **Édition des barres** : déplacer une barre pour changer le début, une étape
   d'undo par drag ; recalcul CPM immédiat.
3. **Calendrier** : week-ends et jours ouvrés optionnels, affichage des dates
   réelles, sans casser le calcul en jours entiers existant.
4. **Tâches** : jalons, réordonnancement, duplication, import CSV en plus de
   l'export.
5. **Confort** : meilleure table (largeur, tri), messages d'erreur, états vides,
   titre et paramètres du projet.

## Qualité

- Préserver le format `projectcraft`, le schéma 1 et le manifeste launcher.
- Tests : dates ouvrées, drag/reschedule, dépendances, import CSV, round-trip,
  undo.
- README/CHANGELOG/ADR à jour.
- Pas de tag/release/sous-agent ; `craftkit` intact.
- fmt, clippy `-D warnings`, tests, push `main`, CI verte.

## Rapport

Hash, CI, fonctions, limites.
