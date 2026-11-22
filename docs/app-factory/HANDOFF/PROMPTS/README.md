# Prompts de productisation

Chaque fichier est prêt à être donné à un agent `hy4-flash-free`, une app par
agent. Ils supposent que le dépôt est déjà cloné dans
`/home/emilio_developpeur/<slug>` et que la release `v0.1.0` est un snapshot.

## Vague A — déjà lancée

- `diagramcraft-product.md`
- `projectcraft-product.md`
- `formcraft-product.md`
- `mathcraft-product.md`
- `mastercraft-product.md`

## Vague B — à lancer après A

- `circuitcraft-product.md`
- `geocraft-product.md`
- `musiccraft-product.md`
- `streamcraft-product.md`
- `simcraft-product.md`

## Rappels à ajouter au prompt

- ne pas modifier `/home/emilio_developpeur/craftkit` ;
- ne pas créer de tag ni de release ;
- ne pas lancer d’autre sous-agent ;
- `CARGO_TARGET_DIR=/tmp/cargo-target` ;
- fmt, clippy `-D warnings`, tests, `launcher_check.py` ;
- push `main`, CI verte, rapport final.
