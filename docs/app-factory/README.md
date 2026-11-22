# App Factory — playbook générique

Ce dossier est la **mémoire opérationnelle réutilisable** pour créer une
application desktop native en Rust, quel que soit son domaine : édition d'image,
musique, vidéo, 3D, CAD, notes, données, outils développeur, etc.

Il ne dépend pas de RackCraft ni d'un format de fichier particulier. RackCraft
sert uniquement de première implémentation de référence : le workflow, les
couches et les pièges système sont réutilisables.

> Objectif : qu'un nouvel agent, après compaction du contexte, puisse reprendre
> la création d'une app en lisant ce dossier, sans redécouvrir l'environnement,
> l'architecture, les gates de qualité ni les problèmes de packaging.

## Ce que contient le playbook

| Fichier | Contenu |
| --- | --- |
| [PORTFOLIO.md](PORTFOLIO.md) | Les dix apps prévues : MVP, risques, réutilisation, ordre de démarrage |
| [WORKFLOW.md](WORKFLOW.md) | Cycle complet : cadrage → architecture → cœur → UI → CI → release → maintenance |
| [HANDOFF.md](HANDOFF.md) | Prompt de reprise compact et checklist d'état |
| [ENVIRONMENT.md](ENVIRONMENT.md) | Cloud Shell, toolchain Rust non persistante, disque, GUI headless, libs systèmes |
| [BOOTSTRAP.md](BOOTSTRAP.md) | Création du dépôt, licences, git, permissions GitHub, premier CI |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Patterns de workspace, frontières de crates, modèle, undo, jobs, tests |
| [UI-EGUI.md](UI-EGUI.md) | UI native egui/eframe 0.37 : APIs récentes, pièges, interactions, tests visuels |
| [DOMAINS.md](DOMAINS.md) | Risques et choix techniques par famille d'app (audio, image, vidéo, 3D, texte…) |
| [RELEASE.md](RELEASE.md) | CI, matrices, AppImage, glibc, permissions, checksums, vérifications |
| [ERRORS.md](ERRORS.md) | Catalogue générique des erreurs probables et de leurs correctifs |
| [AUTOMATION.md](AUTOMATION.md) | Recette et prompt pour automatiser la création d'une nouvelle app |
| [checklists/](checklists/) | Listes de contrôle cadrage / bootstrap / feature / release |
| [templates/](templates/) | Fichiers de départ génériques (AGENTS, README, CI, release, AppImage) |

Les particularités métier de RackCraft (Visio, EMF, câbles, BOM) restent dans le
reste du dépôt (`docs/visio-format.md`, code et tests). Ce dossier-ci reste
volontairement **domain-agnostic**.

## Workflow en 9 étapes

1. **Cadrer** — utilisateurs, problème, 3 à 5 fonctionnalités MVP, formats,
   contraintes, licences des assets. Voir [WORKFLOW.md](WORKFLOW.md#phase-0--cadrage).
2. **Préparer l'environnement** — Rust dans `/tmp`, target dir, disque, libs GUI.
   Voir [ENVIRONMENT.md](ENVIRONMENT.md).
3. **Créer le dépôt** — public, licence duale, README, AGENTS, git remote.
   Voir [BOOTSTRAP.md](BOOTSTRAP.md).
4. **Poser l'architecture** — séparer le cœur métier, l'I/O, l'UI et le binaire.
   Voir [ARCHITECTURE.md](ARCHITECTURE.md).
5. **Coder le cœur d'abord** — modèle pur, tests unitaires, aucun accès UI.
6. **Ajouter l'I/O et la persistance** — formats, limites, fixtures, migrations.
7. **Construire l'UI** — état de session, interactions, ergonomie, accessibilité.
   Voir [UI-EGUI.md](UI-EGUI.md) si egui est choisi.
8. **CI + release** — fmt/clippy/tests, binaires multi-OS, checksums, AppImage.
   Voir [RELEASE.md](RELEASE.md).
9. **Durcir et maintenir** — performance, erreurs, packaging, docs, versions.
   Voir [ERRORS.md](ERRORS.md) et [DOMAINS.md](DOMAINS.md).

## Règles non négociables

- **Licence duale** `MIT OR Apache-2.0` pour les nouveaux projets du playbook.
- **Projet indépendant** : ne pas prétendre à une affiliation officielle avec
  l'écosystème dont on suit les conventions (ArtCraft, etc.).
- **Pas de builds release locaux** en environnement contraint : CI/GitHub
  Actions construit les binaires finaux.
- **Toujours `CARGO_TARGET_DIR=/tmp/cargo-target`** et toolchain dans `/tmp`.
- **Aucun asset sans droits** : les fichiers vendeur restent hors du dépôt.
- **Le cœur métier ne dépend jamais de l'UI ni du disque.**
- **Pas de secret dans le dépôt.**
- **Chaque version publiée est vérifiée après upload** : checksums, binaire
  lancé, et si possible un cas d'usage réel.

## Utiliser ce playbook avec un agent

1. Donner [HANDOFF.md](HANDOFF.md) et les documents de la ligne directrice.
2. Répondre au questionnaire de [AUTOMATION.md](AUTOMATION.md#1-brief-d-entrée).
3. Faire exécuter les checklists dans l'ordre.
4. Exiger une preuve à chaque gate : commandes, résultats, assets vérifiés.

## Exemples de domaines

- **Image** : colorimétrie, gros fichiers, tiles, GPU, undo, formats.
- **Musique** : temps réel, thread audio, latence, plugins, sample rates.
- **Vidéo** : codecs, jobs longs, mémoire, accélération matérielle.
- **3D/CAD** : précision, unités, wgpu, formats, scène.
- **Texte/docs** : shaping, pagination, PDF, polices, i18n.
- **Données/diagrammes** : géométrie, snapping, imports/exports, gros documents.

Les risques propres à chaque famille sont détaillés dans [DOMAINS.md](DOMAINS.md).
