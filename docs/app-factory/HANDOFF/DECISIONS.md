# Décisions à ne pas rouvrir

## RackCraft

- **RackCraft reste hors du portefeuille ArtCraft.** C’est le projet d’un ami,
  trop spécifique (baies, Visio, câblage) pour être publié dans ArtCraft.
- Il reste la référence technique et l’origine de CraftKit.
- Ne pas y ajouter `craft-app.json`, ne pas le tagger, ne pas le migrer.
- Des correctifs ciblés sont possibles à la demande du propriétaire. Dernier :
  canevas auto-extensible pour les stencils 42U, commit `8515ee6` (24/11/2026).

## Concurrence des agents

- **5 sous-agents maximum** en parallèle sur `hy4-flash-free`.
- Aucune limite de concurrence officielle n’est publiée ; le quota journalier
  est le vrai risque.
- 15 sous-agents ont tourné sans `429`, mais 5 reste la marge de sécurité.
- Tester 6 avec une petite tâche avant d’augmenter durablement.

## Produit

- Les 10 apps sont des **produits génériques**, pas des clones.
- Pas de parité avec Ableton, OBS, iZotope, KiCad, ArcGIS, MATLAB, COMSOL,
  MS Project, Visio ou Access.
- Pas de code, assets ou wording propriétaire copié.
- Chaque app garde son identité et son disclaimer d’indépendance.

## Releases

- `v0.1.0` = snapshots MVP des 10 apps.
- La productisation produira des commits puis des `v0.2.0`.
- Ne pas déplacer un tag publié ; nouvelle version pour un correctif.
- Les workflows GitHub Actions publics construisent les binaires ; pas de
  release locale.

## Launcher

- Chemin prioritaire : **Craft Unofficial Launcher** (`craft-app.json`).
- Le launcher officiel ArtCraft ne lit pas encore ce manifeste (#989).
- Sans certificats Apple/Windows, pas d’installation officielle complète sur
  macOS/Windows.
- Ne pas solliciter l’index officiel sans le pilote DiagramCraft et l’accord des
  mainteneurs.

## Architecture

- CraftKit reste la base commune ; les apps ne doivent pas dépendre les unes des
  autres.
- `core <- io <- ui <- app` ; pas de UI dans `engine`.
- Un agent = un dépôt ; ne pas modifier CraftKit depuis une app.
