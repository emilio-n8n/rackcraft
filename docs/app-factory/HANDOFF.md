# Handoff — reprendre un projet après compaction

## Prompt générique à donner au nouvel agent

```text
Tu reprends une application desktop native en Rust.

Projet : <NOM>
Dépôt : <OWNER>/<SLUG>
Version actuelle : <VERSION>
Domaine : <image | audio | vidéo | 3D | documents | données | autre>
Objectif de cette session : <OBJECTIF ATOMIQUE>

Avant de coder, lis dans l'ordre :
1. docs/app-factory/README.md
2. docs/app-factory/WORKFLOW.md
3. docs/app-factory/ENVIRONMENT.md
4. docs/app-factory/ARCHITECTURE.md
5. docs/app-factory/ERRORS.md
6. docs/app-factory/DOMAINS.md (si le domaine est concerné)

Puis inspecte :
- git status / git log -5
- Cargo.toml workspace
- README/docs/decisions.md
- dernière CI et dernière release (gh run list)

Règles :
- CARGO_TARGET_DIR=/tmp/cargo-target
- pas de build release complet en local ; GitHub Actions s'en charge
- ne pas casser la frontière core <- io <- ui <- app
- fmt + clippy -D warnings + tests workspace avant push
- licence MIT OR Apache-2.0, disclaimer projet indépendant
- pas de secrets ni d'assets non libres dans le dépôt
- ne pas modifier une release publiée ; nouvelle version

Réponds d'abord par :
1. ce que tu as compris du produit ;
2. l'état réel du dépôt et de la CI ;
3. les 3 risques principaux ;
4. la prochaine action atomique, avec la commande exacte.
Ne code pas avant d'avoir fait cette synthèse.
```

## Fiche d'état à remplir

```text
App :
Slug :
Version :
Remote :
Branche :
Dernier commit :
Dernière CI :
Dernière release :
Tests locaux :
Fonctionnalités stables :
Fonctionnalités en cours :
Décisions ouvertes :
Assets / licences :
Secrets nécessaires (jamais dans le repo) :
```

## Pièges génériques à vérifier en premier

- **Toolchain non persistante** : `/tmp` peut être vidé entre deux sessions.
  Vérifier `rustc`, `cargo`, `cargo fmt`, `cargo clippy`.
- **Disque** : `df -h /tmp`; cible Cargo sur un volume assez grand.
- **Version d'API** : toujours consulter la source de la version installée
  (souvent dans le cache Cargo) au lieu de supposer une ancienne API.
- **Runner GitHub** : `ubuntu-latest` peut changer d'Ubuntu/glibc ; épingler une
  version de runner pour les binaires Linux destinés au public.
- **Permissions release** : un `403 Resource not accessible by integration`
  signifie que le token Actions n'a pas `contents: write`.
- **AppImage** : tester FUSE 2 et un lancement réel de l'asset.
- **Opérations lourdes** : ne pas bloquer le thread UI ; mesurer mémoire/temps.
- **Tests flottants** : utiliser une tolérance pour les conversions d'écran.
- **Contexte** : documenter les décisions dans `docs/decisions.md` au fil de
  l'eau, pas à la fin.

## Reprise d'une fonctionnalité

1. Reformuler le comportement attendu et les cas d'erreur.
2. Localiser le crate le plus profond possible.
3. Écrire le test d'abord (cœur, I/O, puis UI).
4. Coder, formater, clippy ciblé, tests ciblés.
5. Vérifier visuellement si UI.
6. Mettre à jour README/CHANGELOG/captures.
7. Commit atomique.
8. CI → tag → release seulement pour une version complète.

## Stop conditions

Arrêter et demander avant :

- d'ajouter une dépendance lourde (GPU, FFmpeg, codec, framework UI) ;
- de changer le format de fichier ou le schéma de façon incompatible ;
- de supprimer/migrer des données utilisateur ;
- de publier une release ou de déplacer un tag ;
- d'inclure un asset dont la licence n'est pas clarifiée.
