# Productisation — suivi des tranches v0.2

## Vague A

| App | Commit | CI | État |
| --- | --- | --- | --- |
| ProjectCraft | `847ec5d` | ✅ | Livré |
| FormCraft | `59f0850` | ✅ | Livré |
| MasterCraft | `bb09a91` | ✅ | Livré |
| DiagramCraft | `19ac4ba` | ✅ | Livré |
| MathCraft | `efc1f7d` | ✅ | Livré |
| MasterCraft | `bb09a91` | ✅ | Livré |

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

## DiagramCraft — 19ac4ba

- Labels éditables en place (double-clic/Entrée/F2), undo.
- 8 poignées de resize, déplacement groupé, multi-sélection Shift.
- Copier/coller/dupliquer avec remap des ids et liens internes.
- Connecteurs droits + orthogonaux avec waypoints, sélection/suppression.
- Alignement 6 modes, distribution 2 axes.
- Export PDF vectoriel et SVG, zoom-to-fit, grille/snapping, titre éditable.
- Schéma 2 avec champs optionnels ; migration schéma 1 testée. 46 tests.
- Limites : PDF Helvetica WinAnsi (hors Latin-1 → `?`), pas de suppression
  individuelle de waypoint dans l’UI, pas de test visuel Xvfb.

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

## MathCraft — efc1f7d

- Schéma 2 : feuilles multiples, résultats/erreurs persistés, matrices nommées,
  séries CSV, variables, feuille active ; migration schéma 1 testée.
- Références de cellules `A1`, `A2` avec détection de cycles.
- Parseur : notation scientifique, multiplication implicite, 22 fonctions.
- Tracé interactif pan/zoom, plages X/Y, plusieurs courbes, export CSV/PNG.
- Import/export CSV, exemples, aide syntaxique.
- 46 tests lib + 3 bin. Limites : PNG sans graduations textuelles, `x(x+1)`
  n’est pas une multiplication implicite, références inter-feuilles limitées,
  champs matrices A/B non persistés, CSV 2 colonnes, pas de test Xvfb.

## MasterCraft — bb09a91

- Lecture rodio/CPAL : play/pause/stop/seek, playhead, message clair sans device.
- Waveform zoom 1×–64×, sélection, marqueurs, durée.
- Trim, silence, fade in/out, gain et normalisation sur sélection, undo.
- Traitement par lot avec progression/annulation.
- Export float32 ou PCM16, chargement WAV asynchrone.
- `libasound2-dev` ajouté aux workflows Linux.
- Limites : échantillons non sérialisés, pas de spectral/ML, pas de monitoring
  temps réel, fades/trim synchrones. 31 tests.
