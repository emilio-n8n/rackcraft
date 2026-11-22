# Productisation — suivi des tranches v0.2

## Vague A

| App | Commit | CI | État |
| --- | --- | --- | --- |
| ProjectCraft | `847ec5d` | ✅ | Livré |
| DiagramCraft | en cours | — | En cours |
| FormCraft | en cours | — | En cours |
| MathCraft | en cours | — | En cours |
| MasterCraft | en cours | — | En cours |

## Vague B

À lancer après la vague A : CircuitCraft, GeoCraft, MusicCraft, StreamCraft,
SimCraft. Prompts dans `PROMPTS/`.

## ProjectCraft — 847ec5d

- Gantt interactif : flèches de dépendance, sélection par ligne, drag des
  barres avec recalcul CPM et un pas d'undo par gesture.
- Zoom Jour / Semaine / Mois / Fit, défilement horizontal et vertical.
- Jalons, notes, ressource, coût estimé/total, duplication, tri, états vides.
- Import CSV (quoting, BOM, erreurs par ligne) + export CSV.
- Export PDF vectoriel du Gantt (une page A4 paysage, ASCII).
- Dates ouvrées/week-ends à l'affichage, sans changer le CPM en jours entiers.
- 34 tests, schéma 1 conservé (champs additifs `serde(default)`).
- Limites : PDF mono-page ASCII, week-ends non intégrés aux durées, pas de
  nivellement de ressources, drag gauche contraint par les prédécesseurs.

## DiagramCraft — en cours

Objectif : labels inline, resize, multi-sélection, copier/coller, routage
orthogonal, PDF, zoom-to-fit. Voir `PROMPTS/diagramcraft-product.md`.

## FormCraft — en cours

Objectif : grille éditable, schéma, formulaire généré, import CSV, historique.
Voir `PROMPTS/formcraft-product.md`.

## MathCraft — en cours

Objectif : matrices persistées, feuilles multiples, notation scientifique,
tracé interactif, CSV. Voir `PROMPTS/mathcraft-product.md`.

## MasterCraft — en cours

Objectif : playback, sélection, trim/fade, batch, PCM16, chargement asynchrone.
Voir `PROMPTS/mastercraft-product.md`.
