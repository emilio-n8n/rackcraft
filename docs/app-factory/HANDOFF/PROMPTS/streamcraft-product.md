# StreamCraft — productisation v0.2

Objectif : un compositeur de scènes avec capture réelle, pas un clone d'OBS.

## Priorités

1. Capture d'écran ou de fenêtre en preview (commencer par Linux/X11, avec une
   couche abstraite et un fallback clair sur les autres OS).
2. Micro et niveau audio (cpal), source webcam si disponible.
3. Enregistrement local simple (MKV/MP4 via ffmpeg si présent ; sinon
   enregistrement PNG séquence documenté).
4. Transitions de scène, raccourcis, mode plein écran, export PNG.
5. Tests : graphe de scènes, sources, transitions, round-trip ; capture testée
   manuellement si aucun device.

Garder `streamcraft`, schéma 1, manifeste launcher. Ne pas prétendre que la
capture fonctionne si elle n'est pas testée. fmt, clippy -D warnings, tests,
push main, CI verte. Pas de tag/release/sous-agent, `craftkit` intact.
