# MusicCraft — productisation v0.2

Objectif : un petit DAW utilisable pour composer et exporter, pas un clone
d'Ableton.

## Priorités

1. Édition de notes dans le piano roll : déplacement, redimensionnement,
   duplication, quantification, une gesture = un undo.
2. Import/export MIDI (`midly`) et export WAV existant.
3. Plusieurs instruments et effets simples (delay/reverb basiques), mixer.
4. Lecture temps réel avec `cpal` si l'environnement CI le permet (ajouter
   `libasound2-dev` aux workflows Linux) ; sinon lecture offline clairement
   indiquée.
5. Tests : édition de notes, MIDI round-trip, quantification, mixage, undo.

Pas de plugins, pas de VST. Garder `musiccraft`, schéma 1, manifeste launcher.
fmt, clippy -D warnings, tests, push main, CI verte. Pas de tag/release/
sous-agent, `craftkit` intact.
