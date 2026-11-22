# FormCraft — productisation v0.2

Tu reprends FormCraft (`/home/emilio_developpeur/formcraft`) après la tranche 1.
Objectif : un mini front-end de base de données utilisable, pas un clone
d'Access.

## Lecture

- `README.md`, `CHANGELOG.md`, `AGENTS.md`, `docs/decisions.md`
- `src/document.rs`, `src/database.rs`, `src/app.rs`
- Les limites connues du rapport de tranche 1.

## Priorités

1. **Grille éditable** : éditer une cellule, ajouter/supprimer une ligne,
   validation, transaction SQLite, une étape d'undo par édition.
2. **Schéma** : créer une table simple (nom + colonnes typées), ajouter une
   colonne, afficher le schéma.
3. **Import CSV** : importer dans une table existante avec en-têtes, gestion des
   erreurs et des types.
4. **Vue formulaire** : formulaire généré pour la table sélectionnée, création
   et modification d'un enregistrement.
5. **Confort** : filtre/tri, pagination ou limite, rafraîchissement, messages
   d'erreur SQLite, confirmation avant écriture.

## Qualité

- Garder les requêtes SQL libres en lecture seule ; les écritures passent par
  l'UI contrôlée.
- Tests : CRUD, création de table, import CSV, formulaire, round-trip projet.
- README/CHANGELOG/ADR à jour.
- Pas de tag/release/sous-agent ; `craftkit` intact.
- fmt, clippy `-D warnings`, tests, push `main`, CI verte.

## Rapport

Hash, CI, fonctions, limites.
