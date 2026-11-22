# Architecture générique

## Séparer le métier de la présentation

Structure recommandée, adaptable au domaine :

```
apps/<slug>          binaire natif + CLI
  └── crates/ui       présentation, interactions, rendu
        ├── crates/io formats, disque, réseau, exports
        │     └── crates/core  modèle métier pur
        └── framework UI
```

Dépendance à sens unique :

```
core <- io <- ui <- app
```

Avantages :

- tests du métier rapides et sans fenêtre ;
- changement d'UI possible (egui ↔ autre) sans réécrire le domaine ;
- ajout d'un CLI/MCP/WebAssembly plus tard ;
- builds ciblés moins coûteux en environnement contraint.

### Rôle de `core`

- types du domaine, invariants, calculs ;
- ids stables, versions de schéma ;
- undo/redo ;
- sérialisation pure (`serde`) sans accès disque ;
- aucune dépendance egui, winit, rfd, zip, HTTP.

### Rôle de `io`

- parseurs/encodeurs de formats ;
- fichiers et dialogues techniques ;
- conversions vers/depuis le modèle ;
- limites de taille et erreurs actionnables ;
- accès réseau éventuel ;
- exports (PDF, image, audio, etc.).

### Rôle de `ui`

- état de session (document, historique, sélection, outils, caches) ;
- rendu et interactions ;
- traduction des actions en mutations du cœur ;
- feedback utilisateur ;
- aucune logique de format métier.

### Rôle de `app`

- parse des arguments ;
- options `--sample`, `--open`, `--version`, `--help` ;
- appel du framework UI ;
- arrêt propre.

## Concevoir le modèle avant l'UI

Questions à trancher :

- quelles entités ? quels ids ? quelles relations ?
- quelles unités et précisions ? (mm, secondes, samples, pixels, frames)
- quelles données dérivées vs persistées ?
- quelles opérations sont annulables ?
- quels invariants doivent toujours tenir ?

Conseils :

- préférer des structures plates et composables à une hiérarchie profonde ;
- utiliser des ids plutôt que des références cycliques ;
- garder les valeurs calculables hors du stockage ;
- normaliser les compteurs d'ids au chargement ;
- documenter le schéma et sa version dès la v0.1.

## Persistance

- format JSON pour un MVP, si les volumes restent raisonnables ;
- `serde` + champs `#[serde(default)]` pour les évolutions ;
- **`serde_json` avec `float_roundtrip`** si on teste l'égalité bit-à-bit des
  documents contenant des f64 ;
- pour de gros volumes : binaire versionné, SQLite, projet en dossier ;
- toujours tester le round-trip et la migration depuis une ancienne version ;
- ne jamais stocker un chemin absolu critique dans les données.

## Undo/redo

Deux stratégies :

| Stratégie | Avantages | Inconvénients |
| --- | --- | --- |
| Snapshots | simple, robuste, parfait pour un MVP | mémoire si documents énormes |
| Commandes inverses | mémoire maîtrisée | bugs d'inverse, plus de code |

Pour un MVP de bureau, les snapshots bornés (ex. 100 étapes) sont souvent le
meilleur compromis. Ne jamais exposer un `document_mut()` public sans passer par
l'historique.

## Jobs lourds et temps réel

Règle : **le thread UI ne bloque jamais**.

- calcul/image/vidéo/audio offline : thread pool, worker, channel de progression ;
- audio temps réel : thread dédié, pas d'allocation, pas de lock bloquant,
  communication lock-free/canal ;
- affichage : mise à jour progressive, annulation, feedback ;
- GPU : limiter les allocations par frame, recycler les buffers.

Tester les performances sur des fichiers réels, pas seulement des fixtures.

## Design de l'UI

- un état de session central, pas des états éparpillés dans les widgets ;
- séparer « intention » (clic, drag) et « mutation » (undo step) ;
- afficher les actions disponibles, désactiver ce qui ne l'est pas ;
- status bar + notifications pour les erreurs courtes ;
- fenêtre détaillée pour les erreurs longues ;
- raccourcis clavier et focus clavier ;
- confirmation si fermeture avec modifications non sauvegardées ;
- thème et tailles de police centralisés.

## Tests

Pyramide :

1. tests unitaires du cœur : invariants, calculs, undo, sérialisation ;
2. tests d'I/O : parse/encode, fichiers corrompus, limites ;
3. tests d'intégration : flux bout en bout sans UI ;
4. tests UI purs : transforms, formulaires, sélections ;
5. smoke test visuel Xvfb/capture ;
6. test réel d'un asset ou fichier utilisateur représentatif.

Ne pas compenser l'absence de tests du cœur par des tests UI fragiles.

## Dépendances

- privilégier les crates Rust pures (pas de C/C++ système) si possible ;
- vérifier la licence, la MSRV et l'activité ;
- éviter d'empiler deux frameworks UI ou deux moteurs de rendu ;
- épingler les versions majeures qui changent souvent (egui, wgpu, frameworks) ;
- pour les codecs/protocoles, préférer une abstraction interne pour changer de
  backend sans réécrire le cœur.

## Performance

- mesurer avant d'optimiser : `cargo flamegraph`, profiling système, compteurs ;
- attention aux clones de gros buffers et aux allocations par frame ;
- caches bornés (mémoire) et invalidation explicite ;
- chargement paresseux des assets lourds ;
- opérations annulables et progression pour les traitements longs.

## Sécurité

- aucun `unsafe` sans justification ;
- parseurs défensifs : limites de taille, pas de panic, pas de path traversal ;
- secrets via variables d'environnement ou gestionnaire système, jamais dans Git ;
- URLs externes ouvertes via navigateur, pas via shell ;
- mise à jour des dépendances pour les CVE.

## Évolutivité

Commencer simple :

- pas de système de plugins au MVP ;
- pas de synchronisation réseau si le produit est offline ;
- pas de microservices pour une app desktop ;
- pas d'abstraction prématurée de chaque fonction.

Documenter les ADR dans `docs/decisions.md` pour conserver les raisons après
compaction du contexte.
