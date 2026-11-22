# CircuitCraft — productisation v0.2

Objectif : un éditeur de schéma utilisable, pas un clone de KiCad.

## Priorités

1. Rotation des composants (R), labels et valeurs éditables, suppression propre.
2. Fils orthogonaux avec jonctions, sélection et suppression d'un fil.
3. Netlist nommée et export BOM CSV ; export SVG/PDF du schéma.
4. Grille/snapping, zoom-to-fit, meilleure palette de symboles (ajouter
   transistor, ampli op, connecteur si le modèle le permet).
5. Tests : rotation, jonctions, netlist, BOM, export, round-trip, undo.

Pas de PCB, pas de SPICE. Garder le format `circuitcraft`, le schéma 1 et le
manifeste launcher. fmt, clippy -D warnings, tests, push main, CI verte.
Pas de tag/release/sous-agent, `craftkit` intact.
