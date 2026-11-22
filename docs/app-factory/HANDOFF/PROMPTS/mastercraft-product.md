# MasterCraft — productisation v0.2

Tu reprends MasterCraft (`/home/emilio_developpeur/mastercraft`) après la
tranche 1. Objectif : un utilitaire de restauration audio offline crédible, pas
un clone d'iZotope RX.

## Lecture

- `README.md`, `CHANGELOG.md`, `AGENTS.md`, `docs/decisions.md`
- `src/document.rs`, `src/audio.rs`, `src/app.rs`
- Les limites connues du rapport de tranche 1.

## Priorités

1. **Sélection** : sélectionner une plage sur la waveform, traitement limité à
   la sélection, une étape d'undo.
2. **Édition** : trim/crop, fade in/out, silence, normalisation et gain sur la
   sélection.
3. **Export** : choix WAV PCM 16 bits ou float 32 bits, nom de fichier et
   métadonnées simples.
4. **Traitement par lot** : appliquer un preset (gain/normalize/fade) à
   plusieurs WAV, avec job et progression.
5. **Confort** : zoom waveform, marqueurs de début/fin, durée de sélection,
   messages d'erreur, états vides.

Pas de lecture temps réel ni de dénoise ML dans cette tranche.

## Qualité

- Préserver le format `mastercraft`, le schéma 1 et le manifeste launcher.
- Tests : sélection, trim, fade, PCM16, batch, round-trip, undo.
- README/CHANGELOG/ADR à jour.
- Pas de tag/release/sous-agent ; `craftkit` intact.
- fmt, clippy `-D warnings`, tests, push `main`, CI verte.

## Rapport

Hash, CI, fonctions, limites.
