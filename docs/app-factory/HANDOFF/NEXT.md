# Prochaines actions

## Immédiat

1. Vérifier les 9 releases `v0.1.0` :
   ```bash
   for r in projectcraft formcraft mathcraft mastercraft circuitcraft geocraft musiccraft streamcraft simcraft; do
     gh run list --repo emilio-n8n/$r --workflow Release --limit 1
     gh release view v0.1.0 --repo emilio-n8n/$r
   done
   ```
   Si une release a échoué : lire le job, corriger le workflow ou le packaging,
   puis relancer via `workflow_dispatch` avec la version `0.1.0` **sans**
   déplacer le tag. Si le tag est en cause, préférer une nouvelle version.

2. Attendre les 5 agents de productisation vague A.
   Vérifier leurs commits, CI, `python3 tools/launcher_check.py .`, puis lancer
   si possible un test visuel `--sample` sous Xvfb.

## Ensuite

3. Lancer la vague B, **5 agents maximum**, avec les prompts de
   `PROMPTS/<slug>-product.md` :
   - CircuitCraft
   - GeoCraft
   - MusicCraft
   - StreamCraft
   - SimCraft

4. Quand A et B sont terminées :
   - vérifier les 10 CI ;
   - vérifier les binaires `--sample` ;
   - décider des releases `v0.2.0` ;
   - tagger et vérifier chaque release avec `craft-release-check github`.

5. Soumission launcher :
   - ajouter DiagramCraft au **Craft Unofficial Launcher** (l’utilisateur peut
     ajouter le dépôt dans les réglages) ;
   - demander aux mainteneurs pour l’index / le catalogue ;
   - pour le launcher officiel ArtCraft, contacter les mainteneurs avec le
     pilote, la release, le manifeste et le statut de signature ;
   - ne pas se présenter comme officiel avant accord.

## Décisions ouvertes

- Certificats Apple Developer ID / Windows Authenticode : aucun pour l’instant.
- Faut-il ajouter AppImage, `.deb`, `.rpm`, `.msi`, `.dmg` ?
- Faut-il migrer RackCraft vers CraftKit ? **Non**, RackCraft reste hors
  périmètre.
- Faut-il tester 6 agents simultanés ? Oui, après la vague B, avec une petite
  tâche et surveillance des `429`.

## Ne pas faire

- Ne pas toucher à RackCraft.
- Ne pas dépasser 5 agents simultanés sans test.
- Ne pas créer de tag `v0.2.0` avant que la productisation soit vérifiée.
- Ne pas copier de code/assets propriétaires.
- Ne pas ajouter d’entrée dans `index/apps.json` du launcher unofficial.
