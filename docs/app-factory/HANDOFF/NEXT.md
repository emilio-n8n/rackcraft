# Prochaines actions

## Immédiat

1. Les 10 releases `v0.1.0` sont publiées, avec 7 assets, checksums vérifiés
   et `craft-release-check github` à 0 warning.

2. Valider la release `v0.2.1` de StreamCraft et tester l’enregistrement sur le
   binaire publié. Restent : matériel audio/webcam, Windows/macOS à
   l'exécution.

3. Suivre la PR #9 de l’index communautaire :
   https://github.com/YalKouassi/craft-unofficial-launcher/pull/9
   Répondre aux demandes du mainteneur ; ne pas merger soi-même.

## Ensuite

3. Après validation :
   - vérifier les binaires `--version`/`--help` ;
   - soumettre DiagramCraft au Craft Unofficial Launcher ;
   - préparer la revue ArtCraft officielle (certificats à décider).

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
