//! Left-hand stencil browser: libraries, categories, search and drag payloads.

use std::collections::HashMap;

use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, FontId, Pos2, Rect, Sense, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Vec2, pos2, vec2,
};
use rackcraft_io::{Master, StencilLibrary};

use crate::canvas;
use crate::theme;

/// Identifies one master across libraries.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StencilKey {
    /// Index in the app's library vector.
    pub library: usize,
    /// Package master id.
    pub master: String,
}

/// Action produced by the browser UI.
#[derive(Debug, Clone)]
pub enum BrowserAction {
    /// Nothing happened.
    None,
    /// The user asked to import a `.vssx` / `.vss`.
    Import,
    /// A stencil was dropped on the canvas at a screen position.
    Dropped {
        /// Stencil payload.
        key: StencilKey,
        /// Screen position of the drop.
        screen: Pos2,
    },
}

/// Persistent browser state owned by the app.
#[derive(Default)]
pub struct BrowserState {
    /// Search query.
    pub search: String,
    /// Decoded icon textures, keyed by `library:master`.
    pub textures: HashMap<String, TextureHandle>,
    /// Stencil armed by a click (click-to-place mode).
    pub armed: Option<StencilKey>,
    /// Stencil currently being dragged.
    pub dragging: Option<StencilKey>,
}

impl BrowserState {
    /// Forget a library's textures after re-imports.
    pub fn clear_textures(&mut self) {
        self.textures.clear();
    }
}

/// Draw the browser. `canvas_rect` is the canvas area from the previous frame,
/// used to decide whether a drag release happened on the canvas.
pub fn show(
    ui: &mut egui::Ui,
    libraries: &[StencilLibrary],
    state: &mut BrowserState,
    canvas_rect: Option<Rect>,
    dirty: bool,
) -> BrowserAction {
    let mut action = BrowserAction::None;

    ui.horizontal(|ui| {
        theme::section_label(ui, "Stencils");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let import = ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("+ Import .vssx")
                            .size(11.5)
                            .color(theme::TEXT),
                    )
                    .fill(theme::RAISED)
                    .corner_radius(CornerRadius::same(5)),
                )
                .on_hover_text("Import a Visio stencil library (.vssx or .vss)");
            if import.clicked() {
                action = BrowserAction::Import;
            }
        });
    });
    ui.add_space(2.0);
    ui.add(
        egui::TextEdit::singleline(&mut state.search)
            .hint_text("Search by name, part number…")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(4.0);

    if libraries.is_empty() {
        egui::Frame::new()
            .fill(theme::PANEL)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("No stencil libraries")
                        .strong()
                        .color(theme::TEXT),
                );
                ui.label(
                    egui::RichText::new(
                        "Import a .vssx file, or restart with --sample to load the bundled RackCraft libraries.",
                    )
                    .size(11.0)
                    .color(theme::MUTED),
                );
            });
        return action;
    }

    let query = state.search.trim().to_lowercase();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if !query.is_empty() {
                let mut count = 0;
                for (library_index, library) in libraries.iter().enumerate() {
                    for master in library
                        .masters
                        .iter()
                        .filter(|master| master.matches(&query))
                    {
                        count += 1;
                        if let Some(result) =
                            card(ui, library_index, library, master, state, canvas_rect)
                        {
                            action = result;
                        }
                    }
                }
                if count == 0 {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("No matching stencil")
                            .color(theme::MUTED)
                            .size(12.0),
                    );
                }
            } else {
                for (library_index, library) in libraries.iter().enumerate() {
                    ui.add_space(3.0);
                    egui::CollapsingHeader::new(
                        egui::RichText::new(&library.name)
                            .size(12.0)
                            .strong()
                            .color(theme::TEXT),
                    )
                    .id_salt(("library", library_index))
                    .default_open(true)
                    .show(ui, |ui| {
                        for (category, masters) in library.by_category() {
                            egui::CollapsingHeader::new(
                                egui::RichText::new(format!("{category}  ({})", masters.len()))
                                    .size(10.5)
                                    .color(theme::MUTED),
                            )
                            .id_salt(("category", library_index, category))
                            .default_open(true)
                            .show(ui, |ui| {
                                for master in masters {
                                    if let Some(result) =
                                        card(ui, library_index, library, master, state, canvas_rect)
                                    {
                                        action = result;
                                    }
                                }
                            });
                        }
                    });
                }
            }
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(if dirty {
                    "Drag a stencil onto the rack. Changes are unsaved."
                } else {
                    "Drag a stencil onto the rack."
                })
                .size(10.0)
                .color(theme::FAINT),
            );
        });

    action
}

fn card(
    ui: &mut egui::Ui,
    library_index: usize,
    library: &StencilLibrary,
    master: &Master,
    state: &mut BrowserState,
    canvas_rect: Option<Rect>,
) -> Option<BrowserAction> {
    let width = ui.available_width().max(180.0);
    let height = 76.0;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());

    let key = StencilKey {
        library: library_index,
        master: master.id.clone(),
    };
    let is_dragging = state.dragging.as_ref() == Some(&key);
    let is_armed = state.armed.as_ref() == Some(&key);

    let background = if is_dragging || is_armed {
        Color32::from_rgb(52, 37, 35)
    } else if response.hovered() {
        theme::RAISED
    } else {
        theme::PANEL
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(6),
        background,
        Stroke::new(
            1.0,
            if is_dragging || is_armed {
                theme::ACCENT
            } else {
                theme::BORDER
            },
        ),
        StrokeKind::Inside,
    );

    let icon_rect = Rect::from_min_size(rect.min + vec2(8.0, 8.0), vec2(60.0, 60.0));
    if !draw_icon(ui, library_index, master, icon_rect, &mut state.textures) {
        canvas::paint_master_thumbnail(ui.painter(), icon_rect, master);
    }

    let text_left = icon_rect.right() + 10.0;
    let name = if master.name.chars().count() > 28 {
        format!("{}…", master.name.chars().take(27).collect::<String>())
    } else {
        master.name.clone()
    };
    ui.painter().text(
        pos2(text_left, rect.top() + 10.0),
        egui::Align2::LEFT_TOP,
        name,
        FontId::proportional(11.5),
        theme::TEXT,
    );
    let part =
        shape_value(&master.shape_data, &["Part Number", "PartNumber", "SKU"]).unwrap_or("—");
    ui.painter().text(
        pos2(text_left, rect.top() + 29.0),
        egui::Align2::LEFT_TOP,
        part,
        FontId::monospace(9.5),
        theme::ACCENT_LIGHT,
    );
    let meta = format!(
        "{} · {:.0}×{:.0} mm",
        master.category, master.width_mm, master.height_mm
    );
    ui.painter().text(
        pos2(text_left, rect.top() + 45.0),
        egui::Align2::LEFT_TOP,
        truncate(&meta, 34),
        FontId::proportional(9.0),
        theme::MUTED,
    );
    if !master.connection_points.is_empty() {
        ui.painter().text(
            pos2(text_left, rect.top() + 58.0),
            egui::Align2::LEFT_TOP,
            format!("{} connection points", master.connection_points.len()),
            FontId::proportional(8.5),
            theme::FAINT,
        );
    }

    response.clone().on_hover_ui(|ui| {
        ui.set_max_width(300.0);
        ui.label(egui::RichText::new(&master.name).strong());
        ui.label(
            egui::RichText::new(format!("{} · {}", library.name, master.category))
                .size(10.5)
                .color(theme::MUTED),
        );
        ui.label(
            egui::RichText::new(format!(
                "{} × {} mm · {} connection points",
                master.width_mm.round(),
                master.height_mm.round(),
                master.connection_points.len()
            ))
            .size(10.0)
            .color(theme::FAINT),
        );
        ui.label(
            egui::RichText::new("Click to arm, drag to place")
                .size(10.0)
                .color(theme::ACCENT_LIGHT),
        );
    });

    if response.drag_started() {
        state.dragging = Some(key.clone());
        state.armed = None;
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    }
    if response.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    }
    if response.drag_stopped() {
        state.dragging = None;
        if let Some(pointer) = ui.ctx().pointer_latest_pos()
            && canvas_rect.is_some_and(|canvas| canvas.contains(pointer))
        {
            return Some(BrowserAction::Dropped {
                key,
                screen: pointer,
            });
        }
    }
    if response.clicked() {
        if state.armed.as_ref() == Some(&key) {
            state.armed = None;
        } else {
            state.armed = Some(key.clone());
            state.dragging = None;
        }
    }

    None
}

fn draw_icon(
    ui: &egui::Ui,
    library_index: usize,
    master: &Master,
    rect: Rect,
    textures: &mut HashMap<String, TextureHandle>,
) -> bool {
    let Some(icon) = &master.icon else {
        return false;
    };
    if icon.rgba.len() != icon.width as usize * icon.height as usize * 4 || icon.width == 0 {
        return false;
    }
    let cache_key = format!("{library_index}:{}", master.id);
    let texture = textures.entry(cache_key.clone()).or_insert_with(|| {
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [icon.width as usize, icon.height as usize],
            &icon.rgba,
        );
        ui.ctx()
            .load_texture(cache_key, image, TextureOptions::LINEAR)
    });
    let ratio = icon.width as f32 / icon.height as f32;
    let mut width = rect.width() - 8.0;
    let mut height = width / ratio.max(0.01);
    if height > rect.height() - 8.0 {
        height = rect.height() - 8.0;
        width = height * ratio;
    }
    let image_rect = Rect::from_center_size(rect.center(), vec2(width, height));
    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), theme::SUNKEN);
    ui.painter().image(
        texture.id(),
        image_rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    true
}

fn shape_value<'a>(fields: &'a [rackcraft_engine::ShapeField], labels: &[&str]) -> Option<&'a str> {
    fields.iter().find_map(|field| {
        labels
            .iter()
            .any(|label| field.label.trim().eq_ignore_ascii_case(label))
            .then_some(field.value.as_str())
            .filter(|value| !value.trim().is_empty())
    })
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() > max {
        format!(
            "{}…",
            text.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    } else {
        text.to_owned()
    }
}
