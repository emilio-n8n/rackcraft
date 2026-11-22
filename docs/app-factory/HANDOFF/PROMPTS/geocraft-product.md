# GeoCraft — productisation v0.2

Objectif : une carte vectorielle utilisable, pas un clone d'ArcGIS.

## Priorités

1. Outils de dessin : point, ligne, polygone, suppression, édition simple des
   géométries.
2. Table attributaire éditable (ajout/modification de propriétés) et
   filtre/recherche.
3. Import GeoJSON + CSV de points, export GeoJSON/CSV et carte PNG.
4. Styles de couche (couleur, épaisseur, opacité) et zoom-to-layer.
5. Tests : dessin, édition, filtre, import/export, projection, round-trip, undo.

Pas de GeoTIFF/raster ni de base CRS complète. Garder `geocraft`, schéma 1,
manifeste launcher. fmt, clippy -D warnings, tests, push main, CI verte.
Pas de tag/release/sous-agent, `craftkit` intact.
