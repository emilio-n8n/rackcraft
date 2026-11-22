# Prompt de reprise pour une nouvelle session

Copier-coller ce texte dans la nouvelle conversation :

```text
Tu reprends le portefeuille CraftKit (10 applications desktop Rust natives
construites sur la base CraftKit) après une compaction de contexte.

Lis dans cet ordre, dans /home/emilio_developpeur/rackcraft :
1. docs/app-factory/HANDOFF/README.md
2. docs/app-factory/HANDOFF/STATUS.md
3. docs/app-factory/HANDOFF/APPS.md
4. docs/app-factory/HANDOFF/WORKFLOW.md
5. docs/app-factory/HANDOFF/NEXT.md
6. docs/app-factory/HANDOFF/state.json

Règles absolues :
- Ne touche pas à RackCraft : c'est le projet d'un ami, trop spécifique pour
  ArtCraft ; il reste à 372f81a et hors périmètre.
- Maximum 5 sous-agents simultanés, modèle opencode-go/hy4-flash-free.
- Ne pas modifier /home/emilio_developpeur/craftkit depuis une app.
- Pas de tag ni de release v0.2.0 avant vérification complète.
- Pas de code/assets propriétaires copiés.
- Les releases v0.1.0 sont des snapshots ; les commits de productisation
  iront vers v0.2.0.

État au moment de la compaction :
- 10 apps, 10 releases v0.1.0 publiées et validées par craft-release-check
  (0 warning).
- Vague A de productisation terminée : DiagramCraft 19ac4ba, ProjectCraft
  847ec5d, FormCraft 59f0850, MathCraft efc1f7d, MasterCraft bb09a91.
- Vague B : CircuitCraft (`cd71ed6`), SimCraft (`07e56a1`) et GeoCraft
  (`a169b10`) sont livrés, CI verte ; MusicCraft et StreamCraft sont encore
  en cours.
- Sessions B dans WORKFLOW.md ; attendre leurs rapports, vérifier CI et
  launcher_check, puis mettre à jour le dossier.

Première action : lire le dossier, vérifier l'état réel des dépôts et des
sessions, puis dire ce que tu as compris et la prochaine action atomique.
Ne relance pas les agents déjà actifs.
```

## Après compaction

- Vérifier les rapports de vague B.
- Mettre à jour `HANDOFF/PRODUCTIZATION.md`, `STATUS.md`, `APPS.md`,
  `state.json`.
- Vérifier les CI et `python3 tools/launcher_check.py .` des 10 apps.
- Décider des releases `v0.2.0` et de la soumission launcher.
