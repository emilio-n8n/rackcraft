# État du portefeuille — 22 novembre 2026

Ce fichier sert de point de reprise après compaction. Les commits évoluent : la
référence stable est toujours `origin/main` de chaque dépôt.

## Base commune

- **CraftKit** : <https://github.com/emilio-n8n/craftkit>
- Tag de bibliothèque : `v0.1.0`
- Dernier `main` : `8b88798` — pack launcher (manifeste, icône, validateur,
  nommage des assets, contrat Windows `-portable.zip`).
- Crates : `craftkit-core`, `craftkit-io`, `craftkit-jobs`, `craftkit-ui`.
- 35 tests Rust verts, CI verte.

## Vague 1 — terminée

| App | Dépôt | Commit launcher | CI | Fonctions |
| --- | --- | --- | --- | --- |
| DiagramCraft | `emilio-n8n/diagramcraft` | `367cdb1` | ✅ | Formes, connecteurs, pan/zoom, SVG, JSON |
| ProjectCraft | `emilio-n8n/projectcraft` | `3b9df23` | ✅ | Tâches, dépendances, CPM, Gantt, CSV |
| FormCraft | `emilio-n8n/formcraft` | `a0c15e6` | ✅ | SQLite, tables, requêtes lecture seule, CSV |
| MathCraft | `emilio-n8n/mathcraft` | `aea4a49` | ✅ | Expressions, matrices, tracé 2D |
| MasterCraft | `emilio-n8n/mastercraft` | `06faf1c` | ✅ | WAV, waveform, gain, normalisation |

**DiagramCraft v0.1.0** est publié :
<https://github.com/emilio-n8n/diagramcraft/releases/tag/v0.1.0>

- `craft-release-check` (Craft Unofficial Launcher) : **0 warning**
- `sha256sum -c SHA256SUMS.txt` OK
- binaire Linux `--version` OK
- assets : Linux tar.gz, Windows `-portable.zip`, macOS ARM/Intel tar.gz,
  `craft-app.json`, icône 128×128, checksums.

## Vague 2 — terminée

| App | Dépôt | Commit | CI | Périmètre |
| --- | --- | --- | --- | --- |
| CircuitCraft | `emilio-n8n/circuitcraft` | `d86ad8a` | ✅ | Schéma, composants, fils, netlist, BOM, SVG |
| GeoCraft | `emilio-n8n/geocraft` | `cbde169` | ✅ | GeoJSON, couches, carte, table attributaire |
| MusicCraft | `emilio-n8n/musiccraft` | `c971588` | ✅ | MIDI, synthé offline, mixdown WAV |
| StreamCraft | `emilio-n8n/streamcraft` | `76147a7` | ✅ | Scènes, sources, compositeur PNG/SVG |
| SimCraft | `emilio-n8n/simcraft` | `981b128` | ✅ | FEM 2D thermique, Gauss-Seidel |

**Les dix applications sont livrées et leurs CI sont vertes.**

## Launcher

Deux chemins distincts :

1. **Craft Unofficial Launcher** (`YalKouassi/craft-unofficial-launcher`) lit
   réellement `craft-app.json`. C’est le chemin validé par
   `craft-release-check` et visé pour l’instant.
2. **ArtCraft Launcher officiel** (`storytold/craft-launcher`) ne lit pas encore
   `craft-app.json` (proposition #989). Les apps communautaires comme SolidCraft
   et MailCraft sont revues et épinglées par les mainteneurs ; macOS/Windows
   demandent des paquets signés.

Sans certificats Apple/Windows, le chemin officiel reste bloqué pour une
installation complète sur macOS/Windows. Les assets Linux/Windows portables
sont disponibles.

## Rate limit

- `hy4-flash-free` : pas de limite de concurrence publiée ; quota journalier de
  requêtes non exposé.
- 5 agents simultanés = marge de sécurité.
- Usage constaté : ~775 steps Hy4, 15 sous-agents, aucun `429`.
- Tester 6 agents après la fin de la vague 2, avec une petite tâche, avant
  d’adopter 6.

## Prochaines actions

1. Attendre MusicCraft et StreamCraft, vérifier leurs CI.
2. Vérifier les binaires `--sample` des cinq apps de la vague 2.
3. Choisir un pilote (DiagramCraft) et préparer la soumission au launcher
   unofficial.
4. Décider du packaging additionnel : AppImage, `.deb`, `.rpm`, `.msi`, `.dmg`
   et certificats.
5. Mettre à jour ce fichier après chaque vague.
