# Applications — état et périmètre

Chaque app est un dépôt public `emilio-n8n/<slug>`. Toutes partagent :

- le shell CraftKit, le thème, les panneaux, la status bar ;
- `DocumentHost<T>`, JSON versionné, sauvegarde atomique ;
- `History<T>`, undo/redo et gestures ;
- `JobRunner`, progression/annulation ;
- `--sample`, `--open`, `--version` ;
- `craft-app.json`, icône 128×128, `tools/launcher_check.py` ;
- CI GitHub et workflow de release.

Aucune n’est un clone complet d’un logiciel commercial.

## DiagramCraft — diagrammes

- Dépôt : `emilio-n8n/diagramcraft`
- MVP : formes (rectangle, ellipse, note, losange), connecteurs, pan/zoom,
  sélection/déplacement, export SVG, JSON.
- Limites MVP : labels non éditables, pas de resize/rotation, connecteurs
  droits uniquement, pas de multi-sélection, pas de PDF.
- Productisation A livrée (`19ac4ba`) : labels inline, 8 poignées de resize,
  multi-sélection, copier/coller/dupliquer, connecteurs orthogonaux avec
  waypoints, alignement/distribution, PDF vectoriel, zoom-to-fit, 46 tests.
- Limites : PDF Helvetica (hors Latin-1 → `?`), pas de suppression de waypoint
  dans l’UI, pas de test visuel Xvfb.
- Release `v0.1.0` publiée, validée par le launcher unofficial ; la
  productisation ira vers `v0.2.0`.
- C’est le pilote le plus proche de RackCraft.

## ProjectCraft — planification

- Dépôt : `emilio-n8n/projectcraft`
- MVP : tâches, dépendances, CPM, Gantt, CSV, undo.
- Limites : pas de flèches de dépendance, Gantt fixe, pas de drag des barres,
  pas de ressources/coûts, pas d’import CSV, table autrefois étroite (corrigée).
- **Productisation A livrée** : commit `847ec5d`, CI verte, 34 tests.
  Gantt interactif, flèches de dépendance, drag des barres, zoom
  jour/semaine/mois/fit, jalons, notes, ressources/coûts, import CSV, export
  PDF vectoriel, dates ouvrées.
- Release `v0.1.0` publiée ; le travail de productisation n’est pas encore
  taggé (futur `v0.2.0`).
- Limites : PDF ASCII sur une page, week-ends seulement à l’affichage, pas de
  nivellement de ressources.

## FormCraft — base de données

- Dépôt : `emilio-n8n/formcraft`
- MVP : SQLite, tables, grille de lecture, requêtes SELECT/WITH/PRAGMA/EXPLAIN,
  export CSV, base mémoire `--sample`.
- Limites : pas d’édition de cellules, pas de création de table, pas de
  formulaire, pas d’import CSV, requêtes volontairement en lecture seule.
- Productisation A livrée (`59f0850`) : grille éditable, éditeur de schéma,
  formulaire généré, import CSV transactionnel, historique, filtre, schéma 2.
- Limites : `.accdb` non supporté, undo des données en session, schéma non
  annulable, `WITHOUT ROWID` non éditable.
- Release `v0.1.0` publiée ; la productisation ira vers `v0.2.0`.

## MathCraft — calcul scientifique

- Dépôt : `emilio-n8n/mathcraft`
- MVP : parseur, variables, matrices, tracé 2D, historique.
- Limites : matrices non persistées, une seule expression, pas de notation
  scientifique/multiplication implicite, tracé fixe.
- Productisation A : matrices persistées (schéma 2 + migration), feuilles
  multiples, notation scientifique, tracé zoom/pan, import/export CSV.
- Release `v0.1.0` publiée.

## MasterCraft — audio offline

- Dépôt : `emilio-n8n/mastercraft`
- MVP : WAV, waveform, gain, normalisation, export float 32, jobs.
- Limites : pas de lecture audio, pas de sélection/trim/fade, pas de batch, pas
  de PCM16, chargement WAV synchrone.
- Productisation A livrée (`bb09a91`) : playback rodio/CPAL, sélection/zoom,
  trim/silence/fade/gain/normalisation, batch, PCM16, chargement asynchrone.
- Limites : échantillons non sérialisés, pas de spectral/ML, pas de monitoring
  temps réel, fades/trim synchrones.
- Release `v0.1.0` publiée ; la productisation ira vers `v0.2.0`.

## CircuitCraft — schéma électronique

- Dépôt : `emilio-n8n/circuitcraft`
- MVP : composants, pins, fils, netlist union-find, BOM, SVG, canevas.
- Limites : fils droits, pas de jonctions, symboles fixes, pas de PCB/SPICE.
- Productisation B : rotation, labels/valeurs, fils orthogonaux + jonctions,
  netlist nommée, BOM CSV, SVG/PDF, zoom-to-fit.
- Release `v0.1.0` publiée.

## GeoCraft — cartographie vectorielle

- Dépôt : `emilio-n8n/geocraft`
- MVP : GeoJSON, couches, projection Web Mercator, pan/zoom, attributs, SVG/CSV.
- Limites : pas de raster/GeoTIFF, pas de CRS complet, pas d’édition de
  géométrie, trous peints avec la couleur de fond.
- Productisation B : dessin/édition de features, table attributaire éditable,
  import CSV/GeoJSON, export carte PNG, styles de couche.
- Release `v0.1.0` publiée.

## MusicCraft — DAW offline

- Dépôt : `emilio-n8n/musiccraft`
- MVP : MIDI interne, 3 instruments, piano roll, mixdown WAV offline, jobs.
- Limites : pas de son temps réel, pas d’import/export MIDI, édition de notes
  limitée, mono 44,1 kHz float32.
- Productisation B : édition de notes, import/export MIDI, effets simples,
  playback `cpal` si l’environnement le permet.
- Release `v0.1.0` publiée.

## StreamCraft — composition de scènes

- Dépôt : `emilio-n8n/streamcraft`
- MVP : scènes, sources couleur/mire/texte, composition, drag, export PNG/SVG.
- Limites : pas de capture écran/webcam/micro, pas d’encodage vidéo.
- Productisation B : capture écran/fenêtre, micro, enregistrement local,
  transitions, plein écran.
- Release `v0.1.0` publiée.
- Ne pas prétendre que la capture fonctionne sans test réel.

## SimCraft — FEM thermique 2D

- Dépôt : `emilio-n8n/simcraft`
- MVP : grille rectangulaire, différences finies 5 points, Gauss-Seidel/SOR,
  colormap, job progression.
- Limites : pas de multiphysique, pas de transitoire, pas de maillage
  triangulaire, un seul matériau.
- Productisation B : matériaux/régions, flux/convection, maillage plus fin,
  export CSV/PNG, presets, validation.
- Release `v0.1.0` publiée.

## RackCraft — hors périmètre

- Dépôt : `emilio-n8n/rackcraft`, `372f81a`.
- Application de référence, mais **ne pas modifier** : cas d’usage trop
  spécifique (baies, Visio, EMF, BOM), projet d’un ami, non destiné à ArtCraft.
- Ne pas ajouter `craft-app.json`, ne pas tagger, ne pas migrer vers CraftKit
  sans demande explicite.
