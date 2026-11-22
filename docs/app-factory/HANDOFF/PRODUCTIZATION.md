# Productisation — suivi des tranches v0.2

## Vague A

| App | Commit | CI | État |
| --- | --- | --- | --- |
| ProjectCraft | `847ec5d` | ✅ | Livré |
| FormCraft | `59f0850` | ✅ | Livré |
| MasterCraft | `bb09a91` | ✅ | Livré |
| DiagramCraft | en cours | — | En cours |
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

## FormCraft — 59f0850

- Grille éditable (édition inline, ajout/suppression de ligne, undo par édition).
- Éditeur de schéma : création de table, ajout de colonne, suppression avec
  confirmation.
- Formulaire généré depuis le schéma, validation des types.
- Import CSV transactionnel (rollback), en-têtes insensibles à la casse.
- Historique de requêtes borné, persistance schéma 2 avec migration v1.
- Filtre texte, limite de lignes, export CSV, SQL libre en lecture seule.
- Limites : `.accdb` non supporté, undo données en session, pas d'undo sur le
  schéma, `WITHOUT ROWID` non éditable, filtre LIKE global. 16 tests.

## MathCraft — en cours

Objectif : matrices persistées, feuilles multiples, notation scientifique,
tracé interactif, CSV. Voir `PROMPTS/mathcraft-product.md`.

## MasterCraft — bb09a91

- Lecture rodio/CPAL : play/pause/stop/seek, playhead, message clair sans device.
- Waveform zoom 1×–64×, sélection, marqueurs, durée.
- Trim, silence, fade in/out, gain et normalisation sur sélection, undo.
- Traitement par lot avec progression/annulation.
- Export float32 ou PCM16, chargement WAV asynchrone.
- `libasound2-dev` ajouté aux workflows Linux.
- Limites : échantillons non sérialisés, pas de spectral/ML, pas de monitoring
  temps réel, fades/trim synchrones. 31 tests.
