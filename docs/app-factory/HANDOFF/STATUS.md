# État synthétique

## Base

| Élément | Valeur |
| --- | --- |
| CraftKit `main` | `8b88798` |
| CraftKit tag bibliothèque | `v0.1.0` |
| Tests CraftKit | 35 |
| RackCraft | `372f81a`, **exclu, ne pas toucher** |
| Agents simultanés max | **5** |
| Modèle agents | `opencode-go/hy4-flash-free` |

## Les 10 apps

| App | `main` | `v0.1.0` | Release | Productisation |
| --- | --- | --- | --- | --- |
| DiagramCraft | `015c0d8` | `f6e8929` | ✅ publiée | Vague A en cours |
| ProjectCraft | `847ec5d` | `00bdd07` | ✅ publiée | Vague A terminée |
| FormCraft | `aae8d4d` | `79fe16c` | ✅ publiée | Vague A en cours |
| MathCraft | `6d9839b` | `1099943` | ✅ publiée | Vague A en cours |
| MasterCraft | `8d76c81` | `bfddbde` | ✅ publiée | Vague A en cours |
| CircuitCraft | `d86ad8a` | `99ef5ae` | ✅ publiée | Vague B à lancer |
| GeoCraft | `da38592` | `dd771be` | ✅ publiée | Vague B à lancer |
| MusicCraft | `c971588` | `f4f03ce` | ✅ publiée | Vague B à lancer |
| StreamCraft | `76147a7` | `7ad60b7` | ✅ publiée | Vague B à lancer |
| SimCraft | `981b128` | `69464f5` | ✅ publiée | Vague B à lancer |

Les commits `main` ci-dessus sont ceux **avant** la productisation. Les agents
de vague A vont ajouter de nouveaux commits.

## Releases

- Les **10 releases `v0.1.0`** sont publiées, avec **7 assets** chacune :
  archives Linux/Windows/macOS ARM/Intel, `craft-app.json`, icône,
  `SHA256SUMS.txt`.
- Les 10 passent `craft-release-check github` avec **0 warning**.
- Les 10 `SHA256SUMS.txt` ont été vérifiés après téléchargement.
- Aucun MSI, DMG, AppImage, `.deb` ou `.rpm` signé pour l'instant.

## Productisation

| App | Commit | CI | État |
| --- | --- | --- | --- |
| ProjectCraft | `847ec5d` | ✅ | Livré (v0.2 fonctionnelle) |
| DiagramCraft | en cours | — | Vague A |
| FormCraft | en cours | — | Vague A |
| MathCraft | en cours | — | Vague A |
| MasterCraft | en cours | — | Vague A |
| CircuitCraft, GeoCraft, MusicCraft, StreamCraft, SimCraft | — | — | Vague B à lancer |

## Launcher

- **Craft Unofficial Launcher** : lit `craft-app.json`, chemin validé.
- **ArtCraft Launcher officiel** : ne lit pas encore `craft-app.json` (#989) ;
  les apps communautaires approuvées passent par revue + catalogue + paquets
  signés.
- Pas de certificats Apple/Windows disponibles.

## Rate limit

- Aucune limite de concurrence publiée.
- Quota journalier de requêtes non exposé.
- 5 agents simultanés = marge de sécurité.
- ~775 steps Hy4 constatés lors du dernier contrôle, 15 sous-agents, aucun 429.
- Ne pas lancer de 6e agent sans test contrôlé.

## Disque

Au dernier contrôle : ~3,3 Go libres sur `/tmp`, `debug/` ~5 Go.
Les builds complets peuvent remplir le disque et provoquer des `SIGBUS` du
linker. Nettoyer les `debug/build/<app>` des apps terminées avant une vague.
