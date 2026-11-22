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
| DiagramCraft | `19ac4ba` | `f6e8929` | ✅ publiée | Vague A terminée |
| ProjectCraft | `847ec5d` | `00bdd07` | ✅ publiée | Vague A terminée |
| FormCraft | `59f0850` | `79fe16c` | ✅ publiée | Vague A terminée |
| MathCraft | `efc1f7d` | `1099943` | ✅ publiée | Vague A terminée |
| MasterCraft | `bb09a91` | `bfddbde` | ✅ publiée | Vague A terminée |
| CircuitCraft | `cd71ed6` | `99ef5ae` | ✅ publiée | Vague B terminée |
| GeoCraft | `a169b10` | `dd771be` | ✅ publiée | Vague B terminée |
| MusicCraft | `575052b` | `f4f03ce` | ✅ publiée | Vague B terminée |
| StreamCraft | `a0aca05` | `7ad60b7` | ✅ publiée | Vague B terminée |
| SimCraft | `07e56a1` | `69464f5` | ✅ publiée | Vague B terminée |

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
| DiagramCraft | `19ac4ba` | ✅ | Livré |
| ProjectCraft | `847ec5d` | ✅ | Livré |
| FormCraft | `59f0850` | ✅ | Livré |
| MathCraft | `efc1f7d` | ✅ | Livré |
| MasterCraft | `bb09a91` | ✅ | Livré |
| CircuitCraft | `cd71ed6` | ✅ | Livré (v0.2 fonctionnelle) |
| SimCraft | `07e56a1` | ✅ | Livré (v0.2 fonctionnelle) |
| GeoCraft | `a169b10` | ✅ | Livré (v0.2 fonctionnelle) |
| StreamCraft | `a0aca05` | ✅ | Livré (v0.2 fonctionnelle) |
| MusicCraft | `575052b` | ✅ | Livré (v0.2 fonctionnelle) |

## Releases v0.2.0

Les 10 apps ont reçu le tag `v0.2.0` le 22 novembre 2026 et les workflows de
release GitHub Actions ont été lancés :

| App | Commit | Tag | Release |
| --- | --- | --- | --- |
| DiagramCraft | `8f7d284` | `v0.2.0` | en cours |
| ProjectCraft | `1a4f808` | `v0.2.0` | en cours |
| FormCraft | `c6b927e` | `v0.2.0` | en cours |
| MathCraft | `99061bc` | `v0.2.0` | en cours |
| MasterCraft | `dd43c55` | `v0.2.0` | en cours |
| CircuitCraft | `02e8a88` | `v0.2.0` | en cours |
| GeoCraft | `ab09978` | `v0.2.0` | en cours |
| StreamCraft | `08de7dd` | `v0.2.0` | en cours |
| SimCraft | `07e56a1` | `v0.2.0` | en cours |
| MusicCraft | `575052b` | `v0.2.0` | en cours |

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

Au dernier contrôle : ~2,3 Go libres sur `/tmp`, `debug/` ~6 Go.
Les builds complets peuvent remplir le disque et provoquer des `SIGBUS` du
linker. Nettoyer les `debug/build/<app>` des apps terminées avant une vague.
