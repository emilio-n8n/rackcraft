# Audit de bout en bout — 22 novembre 2026

## Ce qui a été testé

Les binaires **publiés** `v0.2.0` (Linux x86_64) ont été téléchargés depuis
GitHub, extraits et lancés sous Xvfb 1400×900 :

- `--version` et `--help` ;
- démarrage avec `--sample`, vérification de la fenêtre et capture d'écran ;
- interactions réelles à la souris/clavier sur les actions principales.

Aucun binaire local n'a été utilisé pour ce premier passage : ce sont les
archives de release qui ont été testées.

## Résultats

| App | Démarrage | Interaction testée | Résultat |
| --- | --- | --- | --- |
| DiagramCraft | ✅ | ajout de forme, connexion | ❌ deux bugs corrigés |
| ProjectCraft | ✅ | ajout de tâche | ✅ |
| FormCraft | ✅ | ajout de ligne | ✅ |
| MathCraft | ✅ | Evaluate | ✅ |
| MasterCraft | ✅ | chargement du sample uniquement | ⚠️ WAV non testé |
| CircuitCraft | ✅ | sélection + rotation | ✅ |
| GeoCraft | ✅ | outil Point + clic carte | ✅ |
| StreamCraft | ✅ | ajout de scène | ✅ |
| SimCraft | ✅ | Solve + carte | ❌ panneau non scrollable, corrigé |
| MusicCraft | ✅ | lecture / playhead | ✅ |

## Bugs trouvés et corrigés

### DiagramCraft

1. Le panneau supérieur affichait **deux boutons Settings** et **trois boutons
   Job** (copier-coller).
2. Les outils Rectangle / Ellipse / Note / Diamond n'ajoutaient une forme que
   si le clic tombait **sur une forme existante**. Sur un canevas vide, rien ne
   se passait, alors que la barre d'état disait « click the canvas ».
3. Corrigé en `1fdab24`, release `v0.2.1` (`c3722fe`).

### SimCraft

1. Le panneau de paramètres n'était pas scrollable. Sur une fenêtre de 900 px,
   le bouton **Solve** et les exports étaient sous le bas de la fenêtre et
   inaccessibles.
2. Corrigé en `976db22` (ScrollArea verticale), release `v0.2.1` (`a76e96d`).

## Ce qui reste non testé de bout en bout

- **Sauvegarde/ouverture via la boîte de dialogue native** : `rfd` n'a pas de
  portail XDG/GTK dans l'environnement Xvfb ; `Ctrl+S` ne produit rien ici. La
  logique `DocumentHost` save/load et le round-trip JSON sont testés
  unitairement, mais pas le dialogue.
- **Exports via l'UI** (CSV, PNG, SVG, PDF, WAV, MIDI) : les fonctions sont
  testées avec des contenus réels, mais le chemin bouton → dialogue → fichier
  n'a pas été automatisé.
- **MasterCraft** : chargement d'un vrai WAV et lecture audio non testés (pas
  de périphérique audio ; le sample intégré s'affiche).
- **StreamCraft** : capture X11 testée sous Xvfb ; pas d'écran physique, pas de
  Wayland, pas de webcam/micro, pas d'encodage ffmpeg (binaire absent).
- **MusicCraft** : MIDI/WAV testés en logique ; pas de périphérique audio réel.
- **Windows et macOS** : binaires construits par GitHub Actions mais non
  exécutés dans cet environnement Linux.
- **Solve de SimCraft** : la carte et le message de fin ont été vérifiés avec
  une limite de 50 balayages ; la convergence complète à 5000 balayages n'a
  pas été chronométrée en release.

## Prochaine campagne

1. Tester les dialogues de fichier sur un vrai bureau Linux (X11 et Wayland).
2. Automatiser un scénario sauvegarde → fermeture → réouverture par app.
3. Tester les exports depuis l'UI et vérifier les fichiers produits.
4. Tester MasterCraft avec un WAV réel et un périphérique audio.
5. Tester StreamCraft sur un écran physique et avec ffmpeg installé.
6. Exécuter les binaires Windows/macOS sur les OS cibles.

## Vérification des correctifs `v0.2.1` (binaires publiés)

Les archives Linux `v0.2.1` ont été retéléchargées et relancées sous Xvfb :

- **DiagramCraft** : `0.2.1`, un seul bouton Settings et un seul Job ; outil
  Rectangle puis clic sur canevas vide → 4 formes, statut « Added a Rectangle
  shape ». Correctif confirmé dans le binaire publié.
- **SimCraft** : `0.2.1`, panneau scrollable ; Solve visible, résolution de
  l'exemple en 449 balayages (ω = 1,940, correction 9,63e-9) et carte de
  température affichée avec sa légende. Correctif confirmé dans le binaire
  publié.
