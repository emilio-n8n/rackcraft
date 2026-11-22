# Risques par domaine

Chaque famille d'application a des contraintes qui changent l'architecture et
les tests. Ce tableau aide à choisir les dépendances et à éviter les mauvaises
surprises.

## Image / retouche

| Sujet | Risque | Recommandation |
| --- | --- | --- |
| Mémoire | images 8K/RAW = plusieurs Go | tiles, cache borné, chargement paresseux |
| Couleur | sRGB/Display P3/CMYK | profil explicite, tests de conversion |
| Codecs | licences (HEIC, JPEG XL, WebP) | vérifier licences, backend abstrait |
| Rendu | CPU lent | wgpu si GPU nécessaire, sinon tuiles CPU |
| Undo | copies complètes d'image | undo par commandes/tiles, pas snapshot complet |
| Formats | EXIF/IPTC | métadonnées préservées ou documentées |
| Tests | comparaisons pixel | tolérances, images de référence petites |

Crates typiques : `image`, `png`, `jpeg-decoder`, `resvg`, `tiny-skia`, `wgpu`,
`littlecms`/`lcms2` (attention C), `kamadak-exif`.

## Audio / musique

| Sujet | Risque | Recommandation |
| --- | --- | --- |
| Temps réel | glitches/crackles | thread audio dédié, pas d'allocation/lock |
| Périphériques | ALSA/PipeWire/JACK | tester sur plusieurs backends |
| Latence | buffers trop grands | configurable, mesurer |
| Sample rates | conversions coûteuses | resampler dédié, qualité explicite |
| Plugins | licences VST/AU, sandbox | hors MVP ou processus séparé |
| Fichiers | WAV/FLAC/MP3/AAC | codecs purs si possible, licences vérifiées |
| Tests | audio difficile à comparer | tests offline sur buffers, pas de device requis |
| Export | offline plus rapide que temps réel | render worker, progression |

Crates typiques : `cpal`, `rodio`, `symphonia`, `rubato`, `hound`, `fundsp`.
Ne pas faire d'allocations ou de logs dans le callback audio.

## Vidéo

| Sujet | Risque | Recommandation |
| --- | --- | --- |
| Codecs | brevets/licences | choisir des codecs libres, documenter |
| Mémoire | décodage 4K/8K | décodage streaming, tuiles |
| CPU/GPU | temps de render | jobs, cache, accélération matérielle |
| Sync | audio/vidéo | timeline centralisée |
| Formats conteneurs | MKV/MP4 | bibliothèque robuste, erreurs claires |
| Tests | fichiers lourds | fixtures courtes, pas de vidéo en Git |

Crates/outils : `ffmpeg` (C, licence à vérifier), `gstreamer`, `wgpu`, `symphonia`.

## 3D / CAD

| Sujet | Risque | Recommandation |
| --- | --- | --- |
| Précision | f32 insuffisant | f64 pour le document, f32 au rendu |
| Unités | mm/m/pouces | type d'unité explicite, conversions testées |
| GPU | pilotes | wgpu + fallback/erreur claire |
| Scène | gros maillages | LOD, culling, buffers réutilisés |
| Import/export | glTF/OBJ/STEP | parseur borné, validation |
| Undo | scène entière | commandes ou diff |

## Texte / documents

| Sujet | Risque | Recommandation |
| --- | --- | --- |
| Shaping | scripts complexes, RTL | moteur de shaping éprouvé |
| Polices | licences, fallback | fontdb/font store, polices libres |
| Pagination | PDF/impression | modèle de layout testé |
| i18n | textes variables | catalogue de messages, pluralisation |
| Accessibilité | lecteurs d'écran | focus/sémantique |

## Diagrammes / données

| Sujet | Risque | Recommandation |
| --- | --- | --- |
| Géométrie | hit-test, zoom | transforms testés, f64 document |
| Snap | seuils en px vs unités | convertir avec le zoom |
| Connecteurs | ids orphelins | résolution dynamique, détachement propre |
| Import/export | formats vendeur | parseur défensif, limites |
| Gros documents | rendu | culling, cache, tests de perf |

## Outils développeur / CLI

| Sujet | Risque | Recommandation |
| --- | --- | --- |
| Entrées | parsing ambigu | `clap`, erreurs et codes de sortie |
| Secrets | fuite dans logs | redaction, jamais d'echo |
| Fichiers | chemins, symlinks | canonicalisation prudente |
| Tests | environnement | tempdirs, pas de dépendance réseau |
| Distribution | plateformes | binaires par cible, checksums |

## Commun à tous les domaines

- charger les gros assets paresseusement ;
- ne jamais bloquer le thread UI ;
- borner la mémoire et les formats ;
- écrire des tests de performance sur des données réalistes ;
- documenter les licences des codecs/assets ;
- prévoir une version de fichier dès le départ ;
- conserver un format d'import/export ouvert et documenté.
