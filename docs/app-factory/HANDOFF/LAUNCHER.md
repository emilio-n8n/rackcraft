# Launcher — contrat et chemins

## Deux chemins distincts

1. **Craft Unofficial Launcher** (`YalKouassi/craft-unofficial-launcher`) :
   implémente `craft-app.json` (proposition `storytold/craft-launcher#989`).
   L’utilisateur ajoute le dépôt dans les réglages ; le launcher vérifie le
   manifeste, l’icône, `SHA256SUMS.txt` et les noms d’assets.
2. **ArtCraft Launcher officiel** (`storytold/craft-launcher`) : ne lit pas
   encore `craft-app.json`. Les apps communautaires approuvées (SolidCraft,
   MailCraft) sont revues et épinglées dans le catalogue ; une release du
   launcher est nécessaire. macOS demande une signature Developer ID, Windows un
   MSI avec publisher vérifiable.

## Descripteur

`craft-app.json` à la racine du dépôt, < 16 Ko, schéma 1 :

```json
{
  "schema": 1,
  "id": "diagramcraft",
  "name": "DiagramCraft",
  "tagline": "Diagram editor for shapes, connectors and SVG export",
  "repo": "emilio-n8n/diagramcraft",
  "publisher": "emilio-n8n",
  "upstream": null,
  "bundle_id": "io.github.emilion8n.diagramcraft",
  "msi_publisher": "emilio-n8n",
  "apple_team": null,
  "website": "https://github.com/emilio-n8n/diagramcraft",
  "color": "#e8573f",
  "icon": "diagramcraft-128.png"
}
```

Règles : pas d’`ai.storyteller.*`, pas de `Learning Machines LLC`, pas de team
Apple ArtCraft, `id` = `^[a-z0-9-]+$`, `repo` exact, icône PNG 128×128.

## Assets de release

```text
<id>-<version>-linux-x86_64.tar.gz
<id>-<version>-windows-x64-portable.zip
<id>-<version>-macos-aarch64.tar.gz
<id>-<version>-macos-x86_64.tar.gz
craft-app.json
<id>-128.png
SHA256SUMS.txt
```

Le suffixe Windows doit être **`-portable.zip`** ; un `.zip` sans `-portable`
est ignoré par le launcher. Pour une installation Windows/macOS officielle, il
faudra `.msi` signé et `.dmg` signé/notarisé.

## État

- Les 10 apps ont une release `v0.2.0` (DiagramCraft et SimCraft : `v0.2.1`)
  avec 7 assets.
- `craft-release-check github` sur la dernière release de chaque app :
  **0 warning**.
- Le checker est celui du Craft Unofficial Launcher 0.12.1 ; les noms d’assets,
  `craft-app.json`, l’icône et `SHA256SUMS.txt` sont conformes.
- Aucun certificat Apple/Windows ; `apple_team: null`.
- Aucun MSI/DMG/AppImage/deb/rpm dans les releases actuelles.

## Soumission à l’index communautaire

PR ouverte : <https://github.com/YalKouassi/craft-unofficial-launcher/pull/9>

- branche `emilio-n8n:add-emilio-n8n-apps` sur le fork ;
- ajoute les 10 entrées à `index/apps.json` avec des catégories ;
- tests `cargo test -p craft-app-descriptor index --locked` : 5/5 ;
- **la décision de merger appartient au mainteneur** ; ne pas merger soi-même
  ni présenter l’index comme acquis.
- Un utilisateur peut aussi ajouter `emilio-n8n/<slug>` à la main dans les
  réglages du launcher sans attendre l’index.

## Launcher officiel

- Demander aux mainteneurs d’ArtCraft le chemin exact, avec le pilote
  DiagramCraft, la release, le manifeste et le statut de signature.
- Ne pas se présenter comme officiel avant accord.

## Vérifier

```bash
python3 tools/launcher_check.py .
python3 tools/launcher_check.py . --release-dir dist
```

Voir `WORKFLOW.md` pour `craft-release-check`.
