# SimCraft — productisation v0.2

Objectif : un outil de simulation thermique 2D utilisable, pas un clone de
COMSOL.

## Priorités

1. Matériaux/régions multiples et conditions aux limites variées (température,
   flux, convection).
2. Maillage plus fin configurable et estimation de convergence ; solveur plus
   rapide si possible.
3. Export des résultats CSV et PNG, colormap avec légende et valeurs.
4. Formulaire de scénario, presets, validation des entrées, messages clairs.
5. Tests : matériaux, flux/convection, convergence, export, colormap,
   round-trip.

Pas de multiphysique, pas de transitoire, pas de maillage triangulaire. Garder
`simcraft`, schéma 1, manifeste launcher. fmt, clippy -D warnings, tests, push
main, CI verte. Pas de tag/release/sous-agent, `craftkit` intact.
