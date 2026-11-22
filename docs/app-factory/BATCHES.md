# Batches de construction

Chaque batch lance **au plus cinq sous-agents** en parallèle sur le modèle
`opencode-go/hy4-flash-free`. Un agent possède un dépôt, une tranche verticale
et ne crée ni tag ni release.

## Règles communes

- Un agent = un dépôt = une application.
- La base `emilio-n8n/craftkit` est épinglée par tag dans `Cargo.toml`.
- L'agent ne modifie jamais les crates `craftkit-*`.
- Une tranche doit rester **compilable et démontrable** : mieux vaut une
  fonctionnalité complète que trois ébauches.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D
  warnings` et `cargo test --workspace` verts avant le push.
- Aucun asset propriétaire, aucune release, aucun tag.
- Rapport final : commit, état CI, ce qui marche, ce qui reste.

## Batch 1 — cinq applications

| Application | Dépôt | Tranche |
| --- | --- | --- |
| DiagramCraft | `emilio-n8n/diagramcraft` | Formes, sélection, déplacement, connecteurs, export SVG |
| ProjectCraft | `emilio-n8n/projectcraft` | Tâches, dépendances, chemin critique, Gantt, CSV |
| FormCraft | `emilio-n8n/formcraft` | SQLite, tables, lignes, requête, export CSV |
| MathCraft | `emilio-n8n/mathcraft` | Expressions, matrices, variables, tracé 2D |
| MasterCraft | `emilio-n8n/mastercraft` | WAV, waveform, normalisation, gain, export |

### DiagramCraft

- Document : formes (rectangle, ellipse, note, losange) et connecteurs.
- Canevas : ajout, sélection, déplacement, suppression, pan/zoom.
- Connecteurs aimantés aux bords, suppression propre avec la forme.
- Export SVG et sauvegarde/ouverture JSON.
- Tests : snapping, résolution des connecteurs, round-trip, undo.

### ProjectCraft

- Tâches avec durée, dépendances et progression.
- Calcul CPM : début/fin au plus tôt et au plus tard, marge, chemin critique.
- Vue Gantt et table éditable.
- Export CSV.
- Tests : graphe connu, cycle détecté, round-trip, undo.

### FormCraft

- Ouvrir/créer une base SQLite.
- Lister les tables, afficher et éditer les lignes.
- Exécuter une requête SQL simple.
- Export CSV de la table courante.
- Tests : création, insertion, requête, export.

### MathCraft

- Évaluateur d'expressions avec variables et fonctions usuelles.
- Matrices : addition, multiplication, déterminant, transposée.
- Tracé 2D d'une fonction.
- Tests : précédence, erreurs de syntaxe, matrices, échantillonnage.

### MasterCraft

- Charger un WAV mono/stéréo.
- Afficher la forme d'onde.
- Normaliser, appliquer un gain, annuler.
- Exporter en WAV.
- Tests : lecture/écriture, normalisation sans clipping, gain, undo.

## Batch 2 — après validation du batch 1

| Application | Dépôt | Tranche |
| --- | --- | --- |
| CircuitCraft | `emilio-n8n/circuitcraft` | Schéma, composants, fils, netlist, BOM |
| GeoCraft | `emilio-n8n/geocraft` | GeoJSON, projection, carte, table attributaire |
| MusicCraft | `emilio-n8n/musiccraft` | Timeline, pistes WAV, MIDI basique, export WAV |
| StreamCraft | `emilio-n8n/streamcraft` | Scènes, capture, enregistrement local |
| SimCraft | `emilio-n8n/simcraft` | FEM 2D thermique, maillage, résultats |

## Prompt type

Le prompt donné à chaque agent contient :

1. le dépôt et le périmètre exact de la tranche ;
2. l'obligation de lire `craftkit/README.md`, `AGENTS.md` et l'application de
   démonstration ;
3. l'interdiction de modifier `craftkit`, de créer un tag ou une release ;
4. les commandes de qualité et l'obligation de pousser sur `main` ;
5. le format du rapport final.

## Après un batch

1. Vérifier les cinq CI.
2. Lancer chaque binaire avec `--sample` sous Xvfb.
3. Corriger les défauts bloquants avant le batch suivant.
4. Mettre à jour [PORTFOLIO.md](PORTFOLIO.md).
