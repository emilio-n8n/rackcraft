# Productisation — suivi des tranches v0.2

## Vague A

| App | Commit | CI | État |
| --- | --- | --- | --- |
| ProjectCraft | `847ec5d` | ✅ | Livré |
| FormCraft | `59f0850` | ✅ | Livré |
| MasterCraft | `bb09a91` | ✅ | Livré |
| DiagramCraft | `19ac4ba` | ✅ | Livré |
| MathCraft | `efc1f7d` | ✅ | Livré |

## Vague B — lancée

| App | Session | État |
| --- | --- | --- |
| CircuitCraft | `ses_dfba3a723ffeXoD9LipEYl1iAe` | ✅ `cd71ed6` |
| GeoCraft | `ses_dfba3a720ffecf0RUkKEJXx11q` | ✅ `a169b10` |
| MusicCraft | `ses_dfba3a70effehu7sWKCqt7WTgQ` | ✅ `64615c8` |
| StreamCraft | `ses_dfba3a70affeoebUmv5Qv5Uvi2` | ✅ `a0aca05` |
| SimCraft | `ses_dfba3a706ffe7Q5ZBUuq49i1I8` | ✅ `07e56a1` |

Prompts : `PROMPTS/<slug>-product.md`. Ne pas lancer de 6e agent.

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

## CircuitCraft — cd71ed6

- Rotation par pas de 90° (`R`, barre d'outils, inspecteur), pins et symboles
  pivotent ; champ `rotation` rétrocompatible (`serde(default)`).
- Fils orthogonaux, jonctions aux pins partagés et croisements, suppression
  individuelle.
- Netlist nommée `GND` / `V+` / `N<n>` par union-find.
- Exports CSV (BOM), SVG et PDF one-page ; palette à 10 symboles ; grille et
  snapping ; zoom-to-fit (`F`).
- 24 tests ; schéma 1 conservé, ouverture des documents v0.1.0 testée.
- Limites : routage sans waypoints, croisements toujours connectés, PDF
  Helvetica non embarquée (non-ASCII → `?`), pas de test visuel, pas de PCB
  ni de SPICE.

## SimCraft — 07e56a1

- Régions rectangulaires multi-matériaux, conductance d'interface par moyenne
  harmonique ; conditions Dirichlet, flux, convection, bord isolé.
- Solveur SOR quasi-optimal (ω ≈ 1,99), résidu, déséquilibre finite-volume,
  estimation des balayages restants, contrôle de grille par demi-résolution.
- Grille jusqu'à 401×401 ; export CSV et PNG avec légende.
- Formulaire à 5 presets, validation par champ, carte de température
  interactive, persistance JSON v0.1.0 compatible.
- 52 tests. Limites : conduction stationnaire uniquement, régions
  axis-alignées, pas de transitoire ni de maillage non structuré, SOR lent sur
  matériaux très hétérogènes.

## GeoCraft — a169b10

- Outils point, ligne, polygone, suppression, édition de sommets par
  glisser-déposer, raccourcis V/P/L/G.
- Table attributaire éditable : ajout, modification, suppression, filtre
  insensible à la casse sur couche/géométrie/id/attributs.
- Imports GeoJSON et CSV de points (`lon/lat`, alias `x/y`) ; exports GeoJSON,
  CSV, PNG cartographique et SVG.
- Styles de couche (couleur, épaisseur, opacité, visibilité), zoom-to-layer,
  pan/zoom Web Mercator, fit-to-bounds.
- 42 tests ; schéma 1 et manifeste launcher inchangés ; champs de style avec
  valeurs par défaut serde.
- Limites : WGS84 → Web Mercator seulement, géométries Multi* aplaties, pas de
  GeoTIFF, styles non exportés en GeoJSON, PNG plafonné à 2048 px, édition de
  sommets sans topologie ni snapping.

## StreamCraft — a0aca05

- Capture écran/fenêtre Linux/X11 réelle (`x11rb`, GetImage), aperçu live,
  énumération de cibles, fallback « unsupported » hors Linux/sans display ;
  test fumée Xvfb exécuté (ignoré par défaut).
- Monitoring audio cpal (F32/I16/U8), niveau RMS, message clair sans micro.
- Webcam : détection `/dev/video*` ; décodage de frames explicitement non
  implémenté.
- Enregistrement : MKV/MP4 via ffmpeg si présent, sinon séquence PNG testée.
- Transitions cut/fade, PageUp/PageDown, F11, Ctrl+R, export PNG/SVG.
- 31 tests + 1 test Xvfb ignoré ; schéma 1 conservé (`transition` optionnel).
- Limites : capture validée sous Xvfb uniquement, micro/webcam non testés sur
  matériel réel, encodage ffmpeg non testé (binaire absent), PNG avec police
  bitmap 3×5, pas de streaming ni de clone OBS.

## MusicCraft — 64615c8

- Piano roll : déplacement et redimensionnement par glisser, duplication (`D`),
  quantification (`Q`), une gesture = une étape d'undo.
- MIDI SMF type 1 : import/export tempo, pistes, programmes, NoteOn/Off,
  vélocités ; round-trip testé.
- 5 oscillateurs, ADSR, delay et reverb par piste, mute/solo/volume par piste
  et volume master.
- Lecture temps réel CPAL quand un périphérique existe, sinon « offline
  preview » affiché explicitement ; mixdown WAV 44,1 kHz float32.
- 30 tests ; schéma 1 conservé, nouveaux champs additifs (`serde(default)`).
- Limites : pas de VST, pas d'entrée MIDI live, pas d'automation, lecture d'un
  buffer pré-rendu, pas de pan ni d'export multipiste, aucun son en CI
  headless.
