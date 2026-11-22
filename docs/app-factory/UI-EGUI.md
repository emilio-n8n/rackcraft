# UI native egui / eframe

Cette page s'applique à toute app native Rust utilisant egui/eframe, quel que
soit le domaine. Les versions évoluent vite : toujours vérifier la source de la
version réellement téléchargée.

## Version de référence (2026)

L'écosystème ArtCraft utilise egui/eframe 0.37. Les APIs ci-dessous correspondent
à cette version et ont cassé plusieurs tutoriels plus anciens.

## Cargo

```toml
eframe = { version = "0.37", default-features = false, features = [
  "glow", "default_fonts", "wayland", "x11", "accesskit", "clipboard", "links"
] }
rfd = "0.17"
```

- `glow` suffit pour une UI 2D ; wgpu est plus lourd et à réserver au rendu GPU.
- `default_fonts` est nécessaire pour les accents/emoji de base.
- `x11` et `wayland` sont indispensables sur Linux.
- `rfd` par défaut utilise les portails XDG ; pas de dépendance GTK sur Linux.
- Ajouter les libs systèmes (voir [ENVIRONMENT.md](ENVIRONMENT.md)).

## Le trait App

En 0.37, le point d'entrée est `ui`, pas `update` :

```rust
impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show(ui, |ui| { ... });
        egui::Panel::left("side").show(ui, |ui| { ... });
        egui::CentralPanel::default().show(ui, |ui| { ... });
    }
}
```

`CentralPanel` doit être ajouté en dernier. Les `Window` flottantes prennent
toujours `ctx`.

## Panneaux

`SidePanel`/`TopBottomPanel` sont remplacés par `Panel` :

```rust
egui::Panel::left("browser").resizable(true).default_size(300.0).show(ui, |ui| ...);
egui::Panel::right("inspector").show(ui, |ui| ...);
egui::Panel::top("top").show(ui, |ui| ...);
egui::Panel::bottom("status").show(ui, |ui| ...);
```

## Styles et thème

```rust
ctx.set_theme(egui::ThemePreference::Dark);
let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
style.spacing.item_spacing = egui::vec2(8.0, 6.0);
ctx.set_style_of(egui::Theme::Dark, style);
```

Changements fréquents :

- `CornerRadius` remplace `Rounding` ;
- `window_corner_radius`, `menu_corner_radius` ;
- `Margin` en `i8` ;
- `Frame::new()` remplace `Frame::none()` ;
- `painter.rect(rect, corner_radius, fill, stroke, StrokeKind)` ;
- `painter.circle_filled/stroke`, `Shape::line`, `Shape::closed_line`.

## Rectangles : deux types à ne pas confondre

Un `egui::Rect` n'a pas de `Rect::new(x,y,w,h)` :

```rust
egui::Rect::from_min_size(pos, size)
egui::Rect::from_min_max(min, max)
```

Si le cœur métier a son propre rectangle, l'aliaser dans l'UI :

```rust
use my_core::Rect as DocRect;
```

Ne pas appeler les méthodes du rectangle métier sur un rectangle egui.

## Interactions

- `ui.allocate_exact_size(size, Sense::click_and_drag())` pour une zone custom.
- `response.hover_pos()`, `response.interact_pointer_pos()`, `ctx.pointer_latest_pos()`.
- `ui.input(|i| i.smooth_scroll_delta.y)` pour la molette.
- `ui.input(|i| i.zoom_delta())` pour pinch/ctrl+scroll.
- `ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::Z))` pour les raccourcis.
- Vérifier `ui.ctx().egui_wants_keyboard_input()` avant les raccourcis globaux.
- Pendant un drag, garder une **position visuelle** et n'écrire dans le document
  qu'au relâchement (une seule étape undo).
- Pour un canvas : pan molette/middle-drag, zoom autour du curseur, hit-test en
  coordonnées document, culling des éléments hors écran.

## Drag & drop

- `Sense::click_and_drag` sur la source.
- `response.drag_started()` / `drag_stopped()`.
- Stocker un payload dans l'état de session (id, type), pas dans un thread.
- Vérifier que la cible est sous le pointeur au relâchement.
- Séparer clic (sélection) et drag (déplacement) par un seuil en pixels.

## Dialogues fichiers

```rust
let path = rfd::FileDialog::new()
    .add_filter("Project", &["json"])
    .set_file_name("untitled.json")
    .save_file();
```

- Toujours gérer l'annulation.
- Forcer l'extension.
- Éviter d'appeler un dialogue dans une boucle de repaint.
- Les erreurs longues vont dans une fenêtre de notice, pas dans un toast tronqué.

## Texte et polices

- Les polices par défaut couvrent les accents mais pas tous les symboles.
- Un caractère manquant s'affiche en carré (« tofu »). Dessiner les icônes au
  `Painter` plutôt que d'utiliser des glyphes exotiques.
- Pour une app multilingue, charger une police avec les scripts nécessaires.
- Tester les tailles de fenêtre minimales et le scaling HiDPI.

## Rendu custom

- `Painter`, `Shape`, `Stroke`, `Pos2`, `Vec2` pour le dessin.
- `egui::Shape::convex_polygon` n'est fiable que pour du convexe ; pour du
  concave, approximer ou utiliser un tessellator adapté.
- Lignes pointillées : `Shape::dashed_line`.
- Flatten Bézier quadratique (12 segments) et cubique (16 segments) avant de
  tracer.
- Cache textures (`ctx.load_texture`) et invalider sur changement de thème.

## Opérations lourdes

Ne pas bloquer `ui()` :

- envoyer le travail à un worker ;
- afficher progression/annulation ;
- recevoir le résultat via channel ;
- ne pas partager un `Mutex` de gros document avec l'UI sans besoin.

Pour l'audio temps réel, le thread UI ne doit jamais allouer ni attendre un lock
audio.

## Tests

- tester les transforms écran/document, l'état de sélection, les formulaires ;
- tolérance `1e-3` pour les conversions f32↔f64 ;
- smoke test visuel via Xvfb (voir [ENVIRONMENT.md](ENVIRONMENT.md)) ;
- comparer les captures pour détecter les régressions de layout ;
- ne pas monter une fenêtre dans les tests unitaires CI sans Xvfb.

## Distribution

- Windows : `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
  pour éviter la console en release.
- macOS/Windows : tester les raccourcis `Cmd`/`Ctrl`, les chemins et les écrans
  HiDPI.
- Linux : fournir un `.desktop`, une icône et si possible une AppImage.
- Vérifier le lancement réel du binaire packagé, pas seulement `cargo run`.
