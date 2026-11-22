# Reprise après compaction — CraftKit / portfolio Craft

**Date du snapshot :** 22 novembre 2026 (soir)
**Objet :** tout ce qu'un nouvel agent doit savoir pour reprendre sans
l'historique du chat.

## Règle absolue

**Ne pas modifier RackCraft.** C'est le projet d'un ami, trop spécifique
(baies, Visio, câblage) pour être publié dans ArtCraft. Il a servi de référence
technique, il reste à part. Le dépôt RackCraft est à `372f81a` et ne doit plus
évoluer pour ce portefeuille.

## Ce qui existe

- **CraftKit** : base Rust native universelle, `emilio-n8n/craftkit`, tag
  bibliothèque `v0.1.0`, `main` `8b88798`.
- **10 applications** générées depuis CraftKit, toutes sur GitHub, CI verte,
  avec `craft-app.json`, icône 128×128, validateur et workflow de release.
- **5 releases `v0.1.0` déjà publiées** : DiagramCraft, ProjectCraft, FormCraft,
  MathCraft, MasterCraft.
- **4 releases `v0.1.0` en cours** au moment du snapshot : CircuitCraft,
  GeoCraft, MusicCraft, StreamCraft, SimCraft (certaines peuvent être finies
  depuis ; vérifier avec `gh`).
- **1 release réelle validée par le launcher unofficial** : DiagramCraft
  `v0.1.0`.

## Travail en cours au moment du snapshot

- **Vague A de productisation** (5 agents `hy4-flash-free`, en cours) :
  DiagramCraft, ProjectCraft, FormCraft, MathCraft, MasterCraft.
- **Vague B prête mais non lancée** : CircuitCraft, GeoCraft, MusicCraft,
  StreamCraft, SimCraft.
- Les 9 releases `v0.1.0` tournent ou viennent de finir sur GitHub Actions.

## Ordre de lecture

1. `README.md` (ce fichier)
2. `STATUS.md` — état synthétique
3. `APPS.md` — détail par application
4. `WORKFLOW.md` — commandes, environnement, vagues d'agents
5. `LAUNCHER.md` — contrat `craft-app.json` et chemins de soumission
6. `DECISIONS.md` — décisions à ne pas rouvrir
7. `INCIDENTS.md` — erreurs déjà rencontrées
8. `NEXT.md` — actions immédiates
9. `PROMPTS/` — prompts de productisation prêts à relancer

## Raccourcis

- CraftKit : <https://github.com/emilio-n8n/craftkit>
- DiagramCraft release :
  <https://github.com/emilio-n8n/diagramcraft/releases/tag/v0.1.0>
- Craft Unofficial Launcher :
  <https://github.com/YalKouassi/craft-unofficial-launcher>
- Issue officielle `craft-app.json` :
  <https://github.com/storytold/craft-launcher/issues/989>

## État git des dépôts

Voir `state.json` pour les commits exacts. Les commits de productisation vont
évoluer : toujours vérifier `origin/main` et la CI avant d'agir.
