# Checklist — fonctionnalité

## Avant de coder

- [ ] Comportement attendu en une phrase.
- [ ] Cas d'erreur listés.
- [ ] Crate cible identifié (le plus profond possible).
- [ ] Impact sur le format de fichier / schéma évalué.
- [ ] Test prévu (cœur, I/O ou UI).

## Implémentation

- [ ] logique dans `core` si elle est pure ;
- [ ] I/O dans `io`, avec limites et erreurs actionnables ;
- [ ] UI dans `ui`, sans parsing métier ;
- [ ] aucun `unwrap` sur une entrée externe ;
- [ ] mutation passée par l'historique si persistée ;
- [ ] opération longue hors du thread UI.

## Vérification

- [ ] test unitaire ou d'intégration ajouté ;
- [ ] `cargo test -p <crate>` vert ;
- [ ] `cargo fmt --all -- --check` vert ;
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` vert ;
- [ ] test manuel ou capture si UI ;
- [ ] test avec un fichier réaliste si import/export ;
- [ ] README/CHANGELOG mis à jour si visible.

## Commit

- [ ] message conventionnel (`feat:`, `fix:`, `docs:`, `refactor:`).
- [ ] diff relu, pas de fichier parasite ;
- [ ] pas de secret, asset propriétaire ou binaire commité.
