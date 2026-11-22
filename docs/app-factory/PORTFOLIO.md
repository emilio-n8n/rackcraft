# Portfolio Craft — dix applications

Ce document transforme la liste des projets en **portefeuille exécutable** :
périmètre MVP, cœur technique, risques, dépendances et ordre de démarrage. Il
complète [WORKFLOW.md](WORKFLOW.md) et [DOMAINS.md](DOMAINS.md).

> Règle commune : chaque app est un dépôt séparé, construit avec le même
> squelette et les mêmes gates. On ne vise **pas** la parité avec le logiciel de
> référence : on livre une tranche utile, testée et distribuable, puis on
> élargit.

## Vue d'ensemble

| App | Alternative | Domaine | Complexité MVP | Première tranche |
| --- | --- | --- | --- | --- |
| DiagramCraft | Visio | Diagrammes | L (M avec RackCraft) | Formes, connecteurs, pages, export |
| ProjectCraft | MS Project | Planification | M | Tâches, dépendances, CPM, Gantt |
| FormCraft | Access | Bases de données | L | SQLite, tables, formulaires, rapports |
| MathCraft | MATLAB | Calcul | L | Matrices, scripts, tracés 2D |
| MasterCraft | iZotope RX | Audio offline | L–XL | Éditeur spectral, denoise, declick |
| CircuitCraft | KiCad | EDA | XL | Schéma, netlist, BOM |
| GeoCraft | ArcGIS | SIG | XL | Couches, projections, carte, table |
| MusicCraft | Ableton Live | DAW | XXL | Timeline, MIDI, synthé, mixage, WAV |
| StreamCraft | OBS Studio | Capture | XXL | Scènes, sources, enregistrement |
| SimCraft | COMSOL | FEM | XXL | FEM 2D thermique/structural |

Échelle : **M** = quelques semaines de travail focalisé ; **L** = plusieurs
mois ; **XL** = plusieurs mois à un an ; **XXL** = projet long, à découper en
sous-produits livrables.

## Socle commun « CraftKit »

Ne pas construire une plateforme avant d'avoir deux apps : copier le template,
puis extraire ce qui se répète. Candidats à l'extraction :

| Brique | Contenu |
| --- | --- |
| `craftkit-ui` | Shell de fenêtre, panneaux, thème, dialogues, notifications, palette de commandes, préférences |
| `craftkit-doc` | Ids stables, `History` borné, enveloppe de projet, `schema_version`, migrations |
| `craftkit-jobs` | Workers, progression, annulation, queue, priorités |
| `craftkit-io` | Sauvegarde atomique, autosave, fichiers récents, détection de format |
| `craftkit-test` | Fixtures déterministes, golden files, helpers Xvfb |
| `craftkit-release` | Workflows CI/release, script AppImage, modèles de README/AGENTS |

Tous les crates restent sous `MIT OR Apache-2.0`. Les formats et idées des
logiciels de référence sont étudiés, **jamais leur code** (voir Licences).

## Applications

### 1. DiagramCraft — diagrammes et modélisation visuelle

**MVP** : formes vectorielles, bibliothèque de stencils, connecteurs aimantés,
pages multiples, groupes, zoom/pan, export PDF/SVG/PNG, sauvegarde autonome.

**Cœur** : modèle de document générique (formes, ancres, connecteurs),
géométrie en unités réelles, snapping, historique, rendu vectoriel.

**Risques** : mode texte/étiquettes, alignement/distribution, import de formats
tiers, performance sur gros diagrammes.

**Réutilisation** : RackCraft fournit déjà le document, les connecteurs, le
snapping, le PDF vectoriel et le rendu egui. Extraire les parties génériques
plutôt que dupliquer.

**Crates** : `egui`, `serde`, `kurbo`, `rust_xlsxwriter`, export PDF maison ou
`printpdf`.

**Licence** : Visio est propriétaire ; aucun code à réutiliser. Les formats
d'échange ouverts (SVG, PDF, Draw.io XML) sont préférables.

### 2. ProjectCraft — gestion de projets et Gantt

**MVP** : tâches, durées, dépendances (FS/SS/FF/SF), chemin critique, Gantt,
ressources, coûts, export PDF/CSV, baseline simple.

**Cœur** : graphe de tâches, calendrier, CPM, calculs de marges, undo.

**Risques** : calendriers ouvrés, fuseaux, dépendances cycliques, gros projets,
arrondis de dates.

**Crates** : `chrono`, `serde`, `egui`, export PDF/CSV. Pas de base de données
au MVP : un projet = un fichier JSON versionné.

**Licence** : MS Project propriétaire ; le format `.mpp` n'est pas ouvert.
Prévoir l'import/export CSV et éventuellement MS Project XML documenté.

### 3. FormCraft — bases de données et applications métier

**MVP** : connexion SQLite, navigateur de tables, éditeur de schéma, formulaire
généré, requêtes SQL, rapports simples, import/export CSV.

**Cœur** : couche SQLite, métadonnées de formulaire, validation, permissions
locales, migrations de schéma.

**Risques** : injection SQL dans les filtres, types SQLite dynamiques, gros
tables, sauvegarde/restauration, verrouillage fichier.

**Crates** : `rusqlite` (SQLite, domaine public), `serde`, `egui`, `csv`.

**Licence** : Access propriétaire ; aucun format `.accdb` à parser au MVP.
SQLite et CSV sont des fondations saines.

### 4. MathCraft — calcul scientifique et visualisation

**MVP** : matrices/vecteurs, expressions mathématiques, variables, scripts,
tracés 2D, import/export CSV, sauvegarde de feuilles de calcul.

**Cœur** : parseur/évaluateur typé, `nalgebra` ou `ndarray`, unités, erreurs
numériques, système de scripts.

**Risques** : sémantique du langage, précision, performance des boucles,
graphiques interactifs, compatibilité MATLAB (ne pas la promettre).

**Crates** : `nalgebra`, `ndarray`, `egui_plot`, `rhai` (script) ou parseur
maison, `csv`.

**Licence** : MATLAB propriétaire ; syntaxe inspirée mais implémentation
originale. Ne pas revendiquer la compatibilité `.m` sans tests dédiés.

### 5. MasterCraft — restauration audio

**MVP** : ouverture WAV/FLAC, éditeur spectral, suppression de bruit stationnaire,
declick, dehum, normalisation, fade, batch, export WAV/FLAC.

**Cœur** : STFT, fenêtrage, overlap-add, détection de clicks, filtres notch,
chaîne de traitement non destructive, undo par régions.

**Risques** : qualité DSP, artefacts, taille mémoire des spectrogrammes, temps
de traitement, annulation des traitements longs.

**Crates** : `rustfft`/`realfft`, `symphonia`, `hound`, `rubato` si resampling.

**Licence** : iZotope propriétaire ; les algorithmes classiques sont libres.
Éviter les modèles ML dont la licence ne permet pas la redistribution.

### 6. CircuitCraft — schémas et PCB

**MVP** : capture de schéma, symboles, fils, nœuds, netlist, BOM, export
PDF/SVG. Le PCB arrive dans une tranche ultérieure.

**Cœur** : modèle schématique, connectivité, règles électriques de base,
bibliothèques de symboles, géométrie.

**Risques** : connectivité complexe, conventions de nommage, import de
bibliothèques, routage et DRC (phase PCB), formats Gerber.

**Crates** : `kurbo`, `serde`, `egui`, parseur de symboles maison. Pour le PCB,
envisager `lyon`/`parry2d` et un moteur de routage dédié.

**Licence** : KiCad est GPL ; ne pas copier son code dans une app MIT/Apache.
Les formats de fichiers ouverts peuvent être implémentés proprement.

### 7. GeoCraft — cartographie et analyse spatiale

**MVP** : couches GeoJSON/Shapefile/GeoTIFF, projections, pan/zoom, styles,
table attributaire, filtre, export carte PNG/PDF.

**Cœur** : géométries, index spatial, reprojection, rendu de tuiles, lecture
raster, requêtes attributaires.

**Risques** : volumes de données, projections, CRS, licences des fonds de carte,
performance du rendu, formats raster.

**Crates** : `geo`, `geojson`, `shapefile`, `proj4rs` (pur Rust) ou `proj`
(lib C), `tiff`, `egui`/`wgpu` pour le rendu.

**Licence** : ArcGIS propriétaire. Les données cartographiques ont leurs propres
licences (OpenStreetMap, IGN, etc.) : ne jamais embarquer une couche sans
attribution claire.

### 8. MusicCraft — station audio numérique

**MVP** : timeline, pistes audio et MIDI, transport, éditeur de notes, un
instrument intégré, mixer, effets de base, export WAV. Pas de plugins au MVP.

**Cœur** : moteur audio temps réel, graphe de nœuds, horloge musicale, MIDI,
édition non destructive, undo.

**Risques** : latence et glitches, thread audio sans allocation, hôtes de
plugins (VST3/CLAP), time-stretching, formats de projet, très grande surface.

**Crates** : `cpal`, `symphonia`, `hound`, `midir`, `fundsp`, `rubato`.
Plugins : CLAP (licence permissive) avant VST3 (SDK GPL/propriétaire).

**Licence** : Ableton propriétaire. Ne pas copier Live. Les instruments/samples
embarqués doivent être libres ou générés.

### 9. StreamCraft — capture et diffusion

**MVP** : scènes, sources écran/fenêtre/webcam/micro, composition, enregistrement
local MKV/MP4. Le streaming RTMP arrive ensuite.

**Cœur** : capture plateforme, compositor GPU, pipeline d'encodage, audio mix,
scènes et transitions, gestion des périphériques.

**Risques** : Wayland/PipeWire, capture Windows/macOS, synchronisation A/V,
encodeurs matériels, licences des codecs, processus longs.

**Crates/outils** : `pipewire`, `windows` (capture), `ffmpeg-next` (vérifier
LGPL/GPL), `wgpu`, `cpal`, `rtmp` ou FFmpeg pour la sortie.

**Licence** : OBS est GPL-2.0 ; aucune copie de code. FFmpeg selon build :
préférer une configuration LGPL, documenter les codecs soumis à brevets.

### 10. SimCraft — simulation multiphysique

**MVP** : FEM 2D, maillage simple, conditions aux limites, thermique linéaire
et/ou élasticité, visualisation des résultats. Pas de couplage multiphysique au
départ.

**Cœur** : maillage, assemblage matriciel, solveur creux, post-traitement,
unités et matériaux.

**Risques** : qualité du maillage, convergence, performance, validation
physique, tentation de tout réimplémenter.

**Crates** : `nalgebra`, `faer` ou `sprs`, `mesh`/`delaunator`, `egui`/`wgpu`.
Envisager un solveur externe (CalculiX, FEniCS) via fichiers plutôt qu'un
solveur maison au début.

**Licence** : COMSOL propriétaire. Les solveurs externes ont leurs licences :
vérifier avant intégration et ne pas les présenter comme embarqués si ce n'est
pas le cas.

## Ordre de démarrage recommandé

L'ordre optimise le **risque** et la **réutilisation**, pas l'ordre de la liste.

1. **DiagramCraft** — réutilise RackCraft, valide le template générique et livre
   rapidement une app complète.
2. **ProjectCraft** — logique pure, peu de dépendances, très testable ; forge
   les briques Gantt/calendrier.
3. **FormCraft** — SQLite et formulaires ; valide la persistance et les
   migrations.
4. **MathCraft** — moteur de calcul et tracés ; prépare les briques scientifiques.
5. **MasterCraft** — DSP offline, progressif, sans temps réel.
6. **CircuitCraft** — schéma d'abord, PCB ensuite.
7. **GeoCraft** — SIG, gros morceau mais isolé.
8. **MusicCraft** — DAW : commencer par un moteur audio jouet, pas par l'UI.
9. **StreamCraft** — capture/encodage : commencer par l'enregistrement local.
10. **SimCraft** — FEM : commencer par un cas 2D validé, pas par COMSOL.

Il est permis de commencer par l'app qui motive le plus, à condition d'accepter
un MVP plus étroit et un délai plus long. L'important est de **ne pas lancer
trois XXL en parallèle**.

## Ce qu'on ne fait pas au MVP

- parité fonctionnelle avec le logiciel de référence ;
- import de formats propriétaires non documentés (`.mpp`, `.accdb`, `.vsd`) ;
- hosting de plugins audio ou vidéo ;
- modèles ML lourds pour le denoise ou la segmentation ;
- cloud, comptes, collaboration temps réel ;
- place de marché d'extensions ;
- mises à jour automatiques non signées.

## Licences et indépendance

- Chaque app est un projet communautaire indépendant, sans affiliation avec
  l'éditeur du logiciel de référence.
- Les noms des concurrents servent uniquement à décrire le positionnement, pas
  à nommer le produit.
- Aucun code GPL/AGPL (OBS, KiCad, etc.) n'est copié dans un dépôt MIT/Apache.
  Une réimplémentation propre à partir de spécifications et de tests est requise.
- Les codecs, polices, samples, cartes et bibliothèques de composants ont leurs
  propres licences : les documenter dans `packaging/THIRD_PARTY.md`.
- Les assets de démonstration sont générés ou libres ; les fichiers vendeur
  restent dans `/tmp`.

## Prochaine étape

Choisir **une** app, remplir son brief avec
[templates/project-brief.md.tmpl](templates/project-brief.md.tmpl), puis suivre
[WORKFLOW.md](WORKFLOW.md). Les autres restent au statut « backlog » jusqu'à ce
qu'une version soit publiée et stable.
