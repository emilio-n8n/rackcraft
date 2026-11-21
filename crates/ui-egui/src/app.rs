//! Main RackCraft application shell and interaction logic.

use std::path::{Path, PathBuf};

use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, Key, Modifiers, PointerButton, Pos2, Rect, RichText,
    Sense, Stroke, pos2, vec2,
};
use rackcraft_engine::{
    Document, History, ItemId, Point, ProjectFields, Rect as DocRect, SnapKind, SnapResult,
    SnapSettings, snap_item,
};
use rackcraft_io::StencilLibrary;

use crate::browser::{self, BrowserState, StencilKey};
use crate::{canvas, inspector, inventory, theme};

/// Startup options.
#[derive(Debug, Clone, Copy, Default)]
pub struct RunOptions {
    /// Open the built-in sample rack immediately.
    pub sample: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tone {
    Info,
    Success,
    Error,
}

struct Status {
    text: String,
    tone: Tone,
}

impl Status {
    fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: Tone::Info,
        }
    }

    fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: Tone::Success,
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: Tone::Error,
        }
    }
}

struct ItemDrag {
    id: ItemId,
    start_pointer: Pos2,
    start_position: Point,
    current: Point,
    snap: Option<SnapResult>,
    moved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingAction {
    New,
    Open,
    Sample,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiAction {
    New,
    Open,
    Save,
    SaveAs,
    Import,
    Sample,
    Undo,
    Redo,
    Inventory,
    ExportExcel,
    Fit,
    Help,
    DeleteSelected,
    DuplicateSelected,
    Escape,
}

/// The eframe application.
pub struct RackCraftApp {
    history: History,
    libraries: Vec<StencilLibrary>,
    browser: BrowserState,
    view: canvas::View,
    selection: Option<ItemId>,
    hovered: Option<ItemId>,
    item_drag: Option<ItemDrag>,
    form: Option<inspector::InspectorForm>,
    project_path: Option<PathBuf>,
    status: Status,
    show_inventory: bool,
    show_help: bool,
    error_dialog: Option<String>,
    confirm: Option<PendingAction>,
    canvas_rect: Option<Rect>,
    drop_queue: Option<(StencilKey, Pos2)>,
    last_title: String,
}

impl RackCraftApp {
    /// Build the app from an eframe creation context.
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>, options: RunOptions) -> Self {
        theme::apply(&cc.egui_ctx);
        let bundled = rackcraft_io::load_bundled_stencils();
        let document = if options.sample {
            rackcraft_engine::sample::document()
        } else {
            Document::new("Untitled rack")
        };
        let master_count: usize = bundled
            .libraries
            .iter()
            .map(|library| library.masters.len())
            .sum();
        let status = if options.sample {
            Status::success(format!(
                "Sample loaded · {} items · {} stencils ready",
                document.items.len(),
                master_count
            ))
        } else {
            Status::info(format!(
                "Ready · {master_count} stencils in {} bundled libraries",
                bundled.libraries.len()
            ))
        };
        let error_dialog = bundled.errors.first().cloned();
        Self {
            history: History::new(document),
            libraries: bundled.libraries,
            browser: BrowserState::default(),
            view: canvas::View::default(),
            selection: None,
            hovered: None,
            item_drag: None,
            form: None,
            project_path: None,
            status,
            show_inventory: false,
            show_help: false,
            error_dialog,
            confirm: None,
            canvas_rect: None,
            drop_queue: None,
            last_title: String::new(),
        }
    }

    fn select(&mut self, id: Option<ItemId>) {
        if self.selection != id {
            self.form = None;
        }
        self.selection = id;
    }

    fn reset_session(&mut self) {
        self.selection = None;
        self.hovered = None;
        self.form = None;
        self.item_drag = None;
        self.browser.armed = None;
        self.browser.dragging = None;
        self.drop_queue = None;
        self.show_inventory = false;
        self.view.initialized = false;
    }

    fn perform(&mut self, action: PendingAction) {
        match action {
            PendingAction::New => {
                self.history.replace(Document::new("Untitled rack"));
                self.project_path = None;
                self.reset_session();
                self.status = Status::info("New document");
            }
            PendingAction::Open => self.open_project(),
            PendingAction::Sample => {
                self.history.replace(rackcraft_engine::sample::document());
                self.project_path = None;
                self.reset_session();
                self.status = Status::success(format!(
                    "Sample loaded · {} items · ready to edit",
                    self.history.document().items.len()
                ));
            }
        }
    }

    fn request(&mut self, action: PendingAction) {
        if self.history.is_dirty() {
            self.confirm = Some(action);
        } else {
            self.perform(action);
        }
    }

    fn handle_ui_action(&mut self, action: UiAction) {
        match action {
            UiAction::New => self.request(PendingAction::New),
            UiAction::Open => self.request(PendingAction::Open),
            UiAction::Save => {
                self.save_project(false);
            }
            UiAction::SaveAs => {
                self.save_project(true);
            }
            UiAction::Import => self.import_stencil(),
            UiAction::Sample => self.request(PendingAction::Sample),
            UiAction::Undo => {
                if self.history.undo() {
                    self.after_history_change();
                    self.status = Status::info("Undo");
                }
            }
            UiAction::Redo => {
                if self.history.redo() {
                    self.after_history_change();
                    self.status = Status::info("Redo");
                }
            }
            UiAction::Inventory => {
                self.show_inventory = !self.show_inventory;
            }
            UiAction::ExportExcel => self.export_inventory(false),
            UiAction::Fit => {
                self.view.initialized = false;
                self.status = Status::info("Canvas fitted to window");
            }
            UiAction::Help => {
                self.show_help = !self.show_help;
            }
            UiAction::DeleteSelected => self.delete_selected(),
            UiAction::DuplicateSelected => self.duplicate_selected(),
            UiAction::Escape => {
                if self.browser.armed.is_some() || self.browser.dragging.is_some() {
                    self.browser.armed = None;
                    self.browser.dragging = None;
                    self.status = Status::info("Placement cancelled");
                } else if self.item_drag.take().is_some() {
                    self.status = Status::info("Move cancelled");
                } else if self.drop_queue.take().is_some() {
                    self.status = Status::info("Drop cancelled");
                } else if self.selection.is_some() {
                    self.select(None);
                } else if self.show_inventory {
                    self.show_inventory = false;
                } else if self.show_help {
                    self.show_help = false;
                }
            }
        }
    }

    fn after_history_change(&mut self) {
        self.form = None;
        self.item_drag = None;
        if self
            .selection
            .is_some_and(|id| self.history.document().item(id).is_none())
        {
            self.selection = None;
        }
    }

    fn save_project(&mut self, force_dialog: bool) -> bool {
        let document = self.history.document().clone();
        let path = if !force_dialog {
            self.project_path.clone()
        } else {
            None
        };
        let path = path.or_else(|| {
            let default_name = format!("{}.json", sanitize_file_stem(&document.name));
            rfd::FileDialog::new()
                .add_filter("RackCraft project", &["json"])
                .set_file_name(default_name)
                .save_file()
                .map(|path| with_extension(path, "json"))
        });
        let Some(path) = path else {
            return false;
        };
        let json = match document.to_json_pretty() {
            Ok(json) => json,
            Err(err) => {
                self.status = Status::error(format!("Could not serialize project: {err}"));
                return false;
            }
        };
        match std::fs::write(&path, json) {
            Ok(()) => {
                self.project_path = Some(path.clone());
                self.history.mark_saved();
                self.status = Status::success(format!("Saved {}", display_path(&path)));
                true
            }
            Err(err) => {
                self.status = Status::error(format!("Could not save project: {err}"));
                false
            }
        }
    }

    fn open_project(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("RackCraft project", &["json"])
            .pick_file()
        else {
            return;
        };
        match std::fs::read_to_string(&path) {
            Ok(json) => match Document::from_json(&json) {
                Ok(document) => {
                    let item_count = document.items.len();
                    self.history.replace(document);
                    self.project_path = Some(path.clone());
                    self.reset_session();
                    self.status = Status::success(format!(
                        "Opened {} · {item_count} items",
                        display_path(&path)
                    ));
                }
                Err(err) => {
                    self.status = Status::error("Invalid project file");
                    self.error_dialog = Some(format!(
                        "The file {} is not a valid RackCraft project.\n\n{err}",
                        path.display()
                    ));
                }
            },
            Err(err) => {
                self.status = Status::error("Could not open project");
                self.error_dialog = Some(format!("Could not read {}:\n\n{err}", path.display()));
            }
        }
    }

    fn import_stencil(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Visio stencil", &["vssx", "vss"])
            .pick_file()
        else {
            return;
        };
        match rackcraft_io::load_stencil_file(&path) {
            Ok(mut library) => {
                let base_name = library.name.clone();
                let mut unique = base_name.clone();
                let mut suffix = 2;
                while self
                    .libraries
                    .iter()
                    .any(|existing| existing.name.eq_ignore_ascii_case(&unique))
                {
                    unique = format!("{base_name} ({suffix})");
                    suffix += 1;
                }
                library.name = unique.clone();
                let master_count = library.masters.len();
                let warning_count = library.warnings.len();
                self.libraries.push(library);
                self.browser.clear_textures();
                self.browser.armed = None;
                self.browser.search.clear();
                self.status = if warning_count == 0 {
                    Status::success(format!("Imported {unique} · {master_count} masters"))
                } else {
                    Status::info(format!(
                        "Imported {unique} · {master_count} masters ({warning_count} warnings)"
                    ))
                };
            }
            Err(err) => {
                self.status = Status::error("Stencil import failed");
                self.error_dialog = Some(format!("Could not import {}.\n\n{err}", path.display()));
            }
        }
    }

    fn export_inventory(&mut self, csv: bool) {
        let document = self.history.document();
        let bom = document.bom();
        let title = format!("{} — inventory", document.name);
        let extension = if csv { "csv" } else { "xlsx" };
        let filter_name = if csv {
            "CSV inventory"
        } else {
            "Excel inventory"
        };
        let default_name = format!(
            "{}-inventory.{extension}",
            sanitize_file_stem(&document.name)
        );
        let Some(path) = rfd::FileDialog::new()
            .add_filter(filter_name, &[extension])
            .set_file_name(default_name)
            .save_file()
            .map(|path| with_extension(path, extension))
        else {
            return;
        };
        let result = if csv {
            rackcraft_io::export_bom_csv(&bom, &path)
        } else {
            rackcraft_io::export_bom_xlsx(&bom, &path, &title)
        };
        match result {
            Ok(()) => {
                self.status = Status::success(format!(
                    "Exported {} · {} items",
                    display_path(&path),
                    bom.total_quantity
                ));
            }
            Err(err) => {
                self.status = Status::error("Inventory export failed");
                self.error_dialog = Some(format!("Could not write {}:\n\n{err}", path.display()));
            }
        }
    }

    fn delete_selected(&mut self) {
        let Some(id) = self.selection else {
            return;
        };
        let name = self
            .history
            .document()
            .item(id)
            .map(|item| item.name.clone())
            .unwrap_or_else(|| "Item".into());
        let mut removed = false;
        self.history.apply(|doc| {
            removed = doc.remove_item(id).is_some();
        });
        if removed {
            self.select(None);
            self.status = Status::success(format!("Removed {name}"));
        }
    }

    fn duplicate_selected(&mut self) {
        let Some(id) = self.selection else {
            return;
        };
        let mut new_id = None;
        self.history.apply(|doc| {
            new_id = doc.duplicate_item(id, 20.0);
        });
        if let Some(new_id) = new_id {
            self.select(Some(new_id));
            self.status = Status::success("Item duplicated");
        }
    }

    fn apply_form(&mut self, form: inspector::InspectorForm) {
        let id = form.item_id;
        self.history.apply(|doc| {
            if let Some(item) = doc.item_mut(id) {
                item.set_shape_value("Manufacturer", form.manufacturer.trim());
                item.set_shape_value("Part Number", form.part_number.trim());
                item.set_shape_value("Product Description", form.description.trim());
                item.project = ProjectFields {
                    installed_date: form.installed_date.trim().to_owned(),
                    warranty_exp: form.warranty_exp.trim().to_owned(),
                    location: form.location.trim().to_owned(),
                    building: form.building.trim().to_owned(),
                    notes: form.notes.clone(),
                };
            }
        });
        self.form = None;
        self.status = Status::success("Shape data applied");
    }

    fn place_stencil_at(&mut self, key: &StencilKey, screen: Pos2) {
        let Some(rect) = self.canvas_rect else {
            return;
        };
        let Some(library) = self.libraries.get(key.library) else {
            return;
        };
        let Some(master) = library.master(&key.master).cloned() else {
            return;
        };
        let library_name = library.name.clone();
        let cursor = self.view.screen_to_doc(rect.min, screen);
        let mut item = master.to_item(&library_name, Point::default());
        let canvas_rect = DocRect::new(
            0.0,
            0.0,
            self.history.document().canvas.width_mm,
            self.history.document().canvas.height_mm,
        );
        let desired = Point::new(
            cursor.x - item.size.width * 0.5,
            cursor.y - item.size.height * 0.5,
        );
        let desired = canvas_rect.clamp_position(item.size, desired);
        let settings = self.snap_settings();
        let snap = snap_item(self.history.document(), &item, desired, &settings);
        item.position = snap.position;
        let item_name = item.name.clone();
        let mut new_id = None;
        self.history.apply(|doc| {
            new_id = Some(doc.add_item(item));
        });
        if let Some(id) = new_id {
            self.select(Some(id));
            self.browser.armed = None;
            self.browser.dragging = None;
            self.drop_queue = None;
            let snap_text = match snap.kind {
                Some(SnapKind::RackUnit) => " · snapped to rack U",
                Some(SnapKind::ConnectionPoint) => " · snapped to port",
                _ => "",
            };
            self.status = Status::success(format!("Placed {item_name}{snap_text}"));
        }
    }

    fn snap_settings(&self) -> SnapSettings {
        SnapSettings {
            threshold_mm: f64::from(14.0 / self.view.zoom.max(0.08)),
            rack_units: true,
            connection_points: true,
            grid_mm: None,
        }
    }

    fn handle_shortcuts(&mut self, ui: &egui::Ui) {
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        let mut action = None;
        ui.input_mut(|input| {
            if input.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)
                || input.consume_key(Modifiers::COMMAND, Key::Y)
            {
                action = Some(UiAction::Redo);
            } else if input.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::S) {
                action = Some(UiAction::SaveAs);
            } else if input.consume_key(Modifiers::COMMAND, Key::Z) {
                action = Some(UiAction::Undo);
            } else if input.consume_key(Modifiers::COMMAND, Key::S) {
                action = Some(UiAction::Save);
            } else if input.consume_key(Modifiers::COMMAND, Key::O) {
                action = Some(UiAction::Open);
            } else if input.consume_key(Modifiers::COMMAND, Key::N) {
                action = Some(UiAction::New);
            } else if input.consume_key(Modifiers::COMMAND, Key::D) {
                action = Some(UiAction::DuplicateSelected);
            } else if input.consume_key(Modifiers::COMMAND, Key::E) {
                action = Some(UiAction::ExportExcel);
            } else if input.consume_key(Modifiers::COMMAND, Key::I) {
                action = Some(UiAction::Import);
            } else if input.consume_key(Modifiers::NONE, Key::Delete)
                || input.consume_key(Modifiers::NONE, Key::Backspace)
            {
                action = Some(UiAction::DeleteSelected);
            } else if input.consume_key(Modifiers::NONE, Key::F1) {
                action = Some(UiAction::Help);
            } else if input.consume_key(Modifiers::NONE, Key::Escape) {
                action = Some(UiAction::Escape);
            }
        });
        if let Some(action) = action {
            self.handle_ui_action(action);
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) -> Option<UiAction> {
        let mut action = None;
        let can_undo = self.history.can_undo();
        let can_redo = self.history.can_redo();
        let dirty = self.history.is_dirty();
        let document = self.history.document();
        let doc_name = document.name.clone();
        let item_count = document.items.len();
        let part_count = document.bom().distinct_parts();
        let path_label = self
            .project_path
            .as_ref()
            .map(|path| display_path(path))
            .unwrap_or_else(|| "Not saved yet".to_owned());

        egui::Panel::top("top-bar")
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(17, 19, 24))
                    .inner_margin(egui::Margin::symmetric(14, 8))
                    .stroke(Stroke::new(1.0, theme::BORDER)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    paint_logo(ui);
                    ui.add_space(2.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("RACKCRAFT")
                                    .size(16.0)
                                    .strong()
                                    .color(theme::TEXT),
                            );
                            theme::badge(ui, env!("CARGO_PKG_VERSION"), theme::ACCENT);
                        });
                        ui.label(
                            RichText::new("VISIO STENCIL RACK DESIGNER")
                                .size(8.0)
                                .color(theme::FAINT),
                        );
                    });
                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if dirty {
                                ui.label(RichText::new("●").size(9.0).color(theme::ACCENT));
                            }
                            ui.label(
                                RichText::new(truncate(&doc_name, 48))
                                    .size(13.0)
                                    .strong()
                                    .color(theme::TEXT),
                            );
                        });
                        ui.label(
                            RichText::new(format!(
                                "{path_label} · {item_count} items · {part_count} part numbers"
                            ))
                            .size(9.0)
                            .color(theme::FAINT),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(RichText::new("?").size(13.0))
                                    .fill(theme::PANEL)
                                    .corner_radius(CornerRadius::same(5))
                                    .min_size(vec2(26.0, 26.0)),
                            )
                            .on_hover_text("Help and keyboard shortcuts (F1)")
                            .clicked()
                        {
                            action = Some(UiAction::Help);
                        }
                        if ui
                            .add(
                                egui::Button::new(RichText::new("Fit").size(10.5))
                                    .fill(theme::PANEL)
                                    .corner_radius(CornerRadius::same(5)),
                            )
                            .on_hover_text("Fit the document canvas in the window")
                            .clicked()
                        {
                            action = Some(UiAction::Fit);
                        }
                        ui.separator();
                        if ui
                            .add_enabled(
                                can_redo,
                                egui::Button::new(RichText::new("Redo").size(10.5))
                                    .fill(theme::PANEL)
                                    .corner_radius(CornerRadius::same(5)),
                            )
                            .on_hover_text("Redo (Ctrl+Shift+Z)")
                            .clicked()
                        {
                            action = Some(UiAction::Redo);
                        }
                        if ui
                            .add_enabled(
                                can_undo,
                                egui::Button::new(RichText::new("Undo").size(10.5))
                                    .fill(theme::PANEL)
                                    .corner_radius(CornerRadius::same(5)),
                            )
                            .on_hover_text("Undo (Ctrl+Z)")
                            .clicked()
                        {
                            action = Some(UiAction::Undo);
                        }
                    });
                });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    if toolbar_button(ui, "New", "New document (Ctrl+N)").clicked() {
                        action = Some(UiAction::New);
                    }
                    if toolbar_button(ui, "Open", "Open a RackCraft project (Ctrl+O)").clicked() {
                        action = Some(UiAction::Open);
                    }
                    if toolbar_button(ui, "Save", "Save the project (Ctrl+S)").clicked() {
                        action = Some(UiAction::Save);
                    }
                    if toolbar_button(ui, "Import .vssx", "Import a Visio stencil (Ctrl+I)")
                        .clicked()
                    {
                        action = Some(UiAction::Import);
                    }
                    if toolbar_button(ui, "Load sample", "Open the offline sample rack").clicked() {
                        action = Some(UiAction::Sample);
                    }
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
                    if toolbar_button(ui, "Fit", "Fit canvas").clicked() {
                        action = Some(UiAction::Fit);
                    }
                    let inventory_label = format!("Inventory  {item_count}");
                    if toolbar_button(ui, &inventory_label, "Preview the bill of materials")
                        .clicked()
                    {
                        action = Some(UiAction::Inventory);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Export Excel")
                                        .size(11.5)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(theme::ACCENT)
                                .corner_radius(CornerRadius::same(5))
                                .min_size(vec2(118.0, 27.0)),
                            )
                            .on_hover_text("Export the inventory to .xlsx (Ctrl+E)")
                            .clicked()
                        {
                            action = Some(UiAction::ExportExcel);
                        }
                        ui.label(
                            RichText::new(format!("{part_count} parts · {item_count} items"))
                                .size(9.5)
                                .color(theme::FAINT),
                        );
                    });
                });
            });
        action
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let path = self
            .project_path
            .as_ref()
            .map(|path| display_path(path))
            .unwrap_or_else(|| "Untitled".to_owned());
        let dirty = self.history.is_dirty();
        let tone_colour = match self.status.tone {
            Tone::Info => theme::MUTED,
            Tone::Success => theme::SNAP,
            Tone::Error => theme::ERROR,
        };
        egui::Panel::bottom("status-bar")
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(16, 18, 23))
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .stroke(Stroke::new(1.0, theme::BORDER)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (dot, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                    ui.painter().circle_filled(dot.center(), 3.0, tone_colour);
                    ui.label(
                        RichText::new(truncate(&self.status.text, 100))
                            .size(10.0)
                            .color(tone_colour),
                    );
                    ui.add_space(6.0);
                    ui.label(RichText::new("·").size(9.0).color(theme::FAINT));
                    ui.label(
                        RichText::new(format!("{path}{}", if dirty { "  (unsaved)" } else { "" }))
                            .size(9.5)
                            .color(if dirty {
                                theme::ACCENT_LIGHT
                            } else {
                                theme::FAINT
                            }),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("MIT OR Apache-2.0 · community project")
                                .size(8.5)
                                .color(theme::FAINT),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("Zoom {}%", self.view.zoom_percent()))
                                .size(9.0)
                                .color(theme::FAINT),
                        );
                    });
                });
            });
    }

    fn inspector_panel(&mut self, ui: &mut egui::Ui) -> Option<inspector::InspectorAction> {
        let mut action = None;
        egui::Panel::right("inspector-panel")
            .resizable(true)
            .default_size(342.0)
            .min_size(290.0)
            .max_size(470.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(11, 8))
                    .stroke(Stroke::new(1.0, theme::BORDER)),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        action = Some(inspector::show(
                            ui,
                            self.history.document(),
                            self.selection,
                            &mut self.form,
                        ));
                    });
            });
        action
    }

    fn canvas_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(0))
            .show(ui, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
                self.canvas_rect = Some(rect);
                {
                    let document = self.history.document();
                    self.view.ensure_fit(rect, &document.canvas);
                }

                let space_down = !ui.ctx().egui_wants_keyboard_input()
                    && ui.input(|input| input.key_down(Key::Space));
                let pointer_over = response
                    .hover_pos()
                    .or_else(|| ui.ctx().pointer_latest_pos())
                    .filter(|pointer| rect.contains(*pointer));

                // Wheel / pinch zoom under the pointer.
                if let Some(pointer) = pointer_over {
                    let (zoom_delta, scroll) =
                        ui.input(|input| (input.zoom_delta(), input.smooth_scroll_delta.y));
                    if (zoom_delta - 1.0).abs() > 0.001 {
                        self.view.zoom_around(rect, pointer, zoom_delta);
                    } else if scroll.abs() > 0.05 {
                        let factor = (scroll * 0.0025).exp();
                        self.view.zoom_around(rect, pointer, factor);
                    }
                }

                // Middle-drag or space+left-drag pans.
                let (middle_down, primary_down, pointer_delta) = ui.input(|input| {
                    (
                        input.pointer.button_down(PointerButton::Middle),
                        input.pointer.button_down(PointerButton::Primary),
                        input.pointer.delta(),
                    )
                });
                let panning =
                    pointer_over.is_some() && (middle_down || (space_down && primary_down));
                if panning {
                    self.view.pan += pointer_delta;
                    ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                }

                // Begin moving an item.
                if response.drag_started_by(PointerButton::Primary)
                    && !panning
                    && self.browser.armed.is_none()
                    && let Some(pointer) = response.interact_pointer_pos()
                {
                    let doc_point = self.view.screen_to_doc(rect.min, pointer);
                    let hit = self
                        .history
                        .document()
                        .hit_test(doc_point, true)
                        .map(|item| (item.id, item.position));
                    if let Some((id, start_position)) = hit {
                        self.select(Some(id));
                        self.item_drag = Some(ItemDrag {
                            id,
                            start_pointer: pointer,
                            start_position,
                            current: start_position,
                            snap: None,
                            moved: false,
                        });
                    }
                }

                // Update drag position and snapping.
                let drag_info = self
                    .item_drag
                    .as_ref()
                    .map(|drag| (drag.id, drag.start_pointer, drag.start_position));
                if let Some((id, start_pointer, start_position)) = drag_info
                    && let Some(pointer) = response
                        .interact_pointer_pos()
                        .or_else(|| ui.ctx().pointer_latest_pos())
                {
                    let updated = {
                        let document = self.history.document();
                        document.item(id).map(|item| {
                            let delta_x = f64::from((pointer.x - start_pointer.x) / self.view.zoom);
                            let delta_y = f64::from((pointer.y - start_pointer.y) / self.view.zoom);
                            let desired =
                                Point::new(start_position.x + delta_x, start_position.y + delta_y);
                            let canvas = DocRect::new(
                                0.0,
                                0.0,
                                document.canvas.width_mm,
                                document.canvas.height_mm,
                            );
                            let desired = canvas.clamp_position(item.size, desired);
                            let snap = snap_item(document, item, desired, &self.snap_settings());
                            (snap, (pointer - start_pointer).length() > 3.0)
                        })
                    };
                    if let Some((snap, moved)) = updated
                        && let Some(drag) = &mut self.item_drag
                    {
                        drag.current = snap.position;
                        drag.snap = snap.is_snapped().then_some(snap);
                        drag.moved |= moved;
                    }
                }

                // Selection on press/click (drag selection is handled above).
                if response.clicked_by(PointerButton::Primary)
                    && !panning
                    && self.item_drag.is_none()
                    && self.browser.armed.is_none()
                    && let Some(pointer) = response.interact_pointer_pos()
                {
                    let doc_point = self.view.screen_to_doc(rect.min, pointer);
                    let hit = self
                        .history
                        .document()
                        .hit_test(doc_point, true)
                        .map(|item| item.id);
                    self.select(hit);
                }

                // Hover.
                if self.item_drag.is_none() {
                    self.hovered = pointer_over.and_then(|pointer| {
                        let doc_point = self.view.screen_to_doc(rect.min, pointer);
                        self.history
                            .document()
                            .hit_test(doc_point, true)
                            .map(|item| item.id)
                    });
                }

                // Build placement ghost from the browser's armed/dragging stencil.
                let active_key = self
                    .browser
                    .dragging
                    .clone()
                    .or_else(|| self.browser.armed.clone());
                let ghost = pointer_over.and_then(|pointer| {
                    let key = active_key.as_ref()?;
                    let library = self.libraries.get(key.library)?;
                    let master = library.master(&key.master)?;
                    let document = self.history.document();
                    let mut item = master.to_item(&library.name, Point::default());
                    let cursor = self.view.screen_to_doc(rect.min, pointer);
                    let desired = Point::new(
                        cursor.x - item.size.width * 0.5,
                        cursor.y - item.size.height * 0.5,
                    );
                    let canvas_rect = DocRect::new(
                        0.0,
                        0.0,
                        document.canvas.width_mm,
                        document.canvas.height_mm,
                    );
                    let desired = canvas_rect.clamp_position(item.size, desired);
                    let snap = snap_item(document, &item, desired, &self.snap_settings());
                    item.position = snap.position;
                    Some((item, snap.position, snap))
                });

                // Paint.
                let document = self.history.document();
                let painter = ui.painter_at(rect);
                canvas::paint_background(&painter, rect, &self.view, &document.canvas);
                let drag_preview = self.item_drag.as_ref().map(|drag| (drag.id, drag.current));
                canvas::paint_document(
                    &painter,
                    rect,
                    &self.view,
                    document,
                    self.selection,
                    self.hovered,
                    drag_preview,
                );
                if let Some((item, position, snap)) = &ghost {
                    canvas::paint_ghost(&painter, rect, &self.view, item, *position);
                    if snap.is_snapped() {
                        canvas::paint_snap(&painter, rect, &self.view, snap);
                    }
                }
                if let Some(drag) = &self.item_drag
                    && let Some(snap) = &drag.snap
                {
                    canvas::paint_snap(&painter, rect, &self.view, snap);
                }
                canvas::paint_hud(&painter, rect, &self.view, document);

                if self.browser.armed.is_some() && pointer_over.is_some() && !panning {
                    ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                }
                if self.item_drag.is_some() {
                    ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                }

                // Commit a finished item drag.
                if response.drag_stopped_by(PointerButton::Primary)
                    && let Some(drag) = self.item_drag.take()
                    && drag.moved
                {
                    let id = drag.id;
                    let position = drag.current;
                    self.history.apply(|doc| {
                        if let Some(item) = doc.item_mut(id) {
                            item.position = position;
                        }
                    });
                    self.status = Status::success("Item moved");
                }

                // Place an armed stencil on click.
                let placement_click = response.clicked_by(PointerButton::Primary)
                    && !panning
                    && self.browser.armed.is_some();
                if placement_click
                    && let (Some(pointer), Some(key)) =
                        (response.interact_pointer_pos(), self.browser.armed.clone())
                {
                    self.place_stencil_at(&key, pointer);
                }

                // A browser drag released over the canvas.
                if let Some((key, screen)) = self.drop_queue.take() {
                    if rect.contains(screen) {
                        self.place_stencil_at(&key, screen);
                    } else {
                        self.status = Status::info("Drop a stencil on the canvas to place it");
                    }
                }
            });
    }

    fn windows(&mut self, ctx: &egui::Context) {
        if self.show_inventory {
            let document = self.history.document();
            let bom = document.bom();
            let action = inventory::show(ctx, &mut self.show_inventory, &bom, &document.name);
            match action {
                inventory::InventoryAction::None | inventory::InventoryAction::Close => {}
                inventory::InventoryAction::Excel => self.export_inventory(false),
                inventory::InventoryAction::Csv => self.export_inventory(true),
            }
        }

        if self.show_help {
            let mut open = true;
            egui::Window::new("RackCraft help")
                .open(&mut open)
                .default_width(480.0)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(
                        RichText::new("Rack & cabling, without the Visio licensing.")
                            .size(14.0)
                            .strong()
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new(
                            "RackCraft is an independent community project compatible with ArtCraft conventions. It is not an official Storytold / ArtCraft product.",
                        )
                        .size(10.0)
                        .color(theme::MUTED),
                    );
                    ui.add_space(8.0);
                    theme::section_label(ui, "Keyboard");
                    egui::Grid::new("shortcuts")
                        .num_columns(2)
                        .spacing([24.0, 5.0])
                        .show(ui, |ui| {
                            for (keys, label) in [
                                ("Ctrl+N / Ctrl+O", "New / open project"),
                                ("Ctrl+S / Ctrl+Shift+S", "Save / save as"),
                                ("Ctrl+I", "Import a .vssx stencil"),
                                ("Ctrl+E", "Export inventory to Excel"),
                                ("Ctrl+Z / Ctrl+Shift+Z", "Undo / redo"),
                                ("Ctrl+D / Del", "Duplicate / delete"),
                                ("Wheel", "Zoom under the cursor"),
                                ("Middle-drag / Space+drag", "Pan the canvas"),
                                ("F1", "This help"),
                            ] {
                                ui.label(RichText::new(keys).monospace().size(9.5).color(theme::ACCENT_LIGHT));
                                ui.label(RichText::new(label).size(10.5).color(theme::MUTED));
                                ui.end_row();
                            }
                        });
                    ui.add_space(8.0);
                    theme::section_label(ui, "Stencils");
                    ui.label(
                        RichText::new(
                            "RackCraft reads Visio 2013+ .vssx stencils, including vector geometry, connection points, shape data and embedded previews. Legacy .vss files are detected and can be converted with LibreOffice.",
                        )
                        .size(10.5)
                        .color(theme::MUTED),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("MIT OR Apache-2.0 · RackCraft contributors")
                            .size(9.0)
                            .color(theme::FAINT),
                    );
                });
            if !open {
                self.show_help = false;
            }
        }

        if let Some(message) = self.error_dialog.clone() {
            let mut open = true;
            egui::Window::new("RackCraft notice")
                .open(&mut open)
                .default_width(540.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label(RichText::new(message).size(11.0).color(theme::TEXT));
                });
            if !open {
                self.error_dialog = None;
            }
        }

        if let Some(pending) = self.confirm {
            let mut decision = 0_u8;
            let mut open = true;
            egui::Window::new("Unsaved changes")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(420.0)
                .show(ctx, |ui| {
                    ui.label(
                        RichText::new("The current document has unsaved changes.")
                            .size(12.0)
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new("Save before continuing, discard the changes, or cancel.")
                            .size(10.5)
                            .color(theme::MUTED),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(RichText::new("Save").color(Color32::WHITE))
                                    .fill(theme::ACCENT),
                            )
                            .clicked()
                        {
                            decision = 1;
                        }
                        if ui.button("Discard").clicked() {
                            decision = 2;
                        }
                        if ui.button("Cancel").clicked() {
                            decision = 3;
                        }
                    });
                });
            match decision {
                1 => {
                    if self.save_project(false) {
                        self.confirm = None;
                        self.perform(pending);
                    }
                }
                2 => {
                    self.confirm = None;
                    self.perform(pending);
                }
                3 => self.confirm = None,
                _ => {
                    if !open {
                        self.confirm = None;
                    }
                }
            }
        }
    }

    fn update_title(&mut self, ctx: &egui::Context) {
        let dirty = self.history.is_dirty();
        let title = format!(
            "{}{} — RackCraft",
            if dirty { "* " } else { "" },
            self.history.document().name
        );
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }
}

impl eframe::App for RackCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ui);

        let top_action = self.top_bar(ui);
        if let Some(action) = top_action {
            self.handle_ui_action(action);
        }
        self.status_bar(ui);

        let left = egui::Panel::left("stencil-panel")
            .resizable(true)
            .default_size(310.0)
            .min_size(250.0)
            .max_size(430.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .stroke(Stroke::new(1.0, theme::BORDER)),
            )
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                browser::show(
                    ui,
                    &self.libraries,
                    &mut self.browser,
                    self.canvas_rect,
                    self.history.is_dirty(),
                )
            });
        match left.inner {
            browser::BrowserAction::None => {}
            browser::BrowserAction::Import => self.import_stencil(),
            browser::BrowserAction::Dropped { key, screen } => {
                self.drop_queue = Some((key, screen));
            }
        }

        if let Some(action) = self.inspector_panel(ui) {
            match action {
                inspector::InspectorAction::None => {}
                inspector::InspectorAction::Apply(form) => self.apply_form(form),
                inspector::InspectorAction::Delete => self.delete_selected(),
                inspector::InspectorAction::Duplicate => self.duplicate_selected(),
            }
        }

        self.canvas_panel(ui);
        self.windows(ui.ctx());
        self.update_title(ui.ctx());
    }
}

fn toolbar_button(ui: &mut egui::Ui, label: &str, hover: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(11.0))
            .fill(theme::PANEL)
            .corner_radius(CornerRadius::same(5))
            .min_size(vec2(0.0, 27.0)),
    )
    .on_hover_text(hover)
}

fn paint_logo(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(9), theme::ACCENT);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(9),
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)),
        egui::StrokeKind::Inside,
    );
    let inner = rect.shrink(8.0);
    for index in 0..3 {
        let y = inner.top() + index as f32 * inner.height() * 0.34;
        ui.painter().line_segment(
            [pos2(inner.left(), y), pos2(inner.right(), y)],
            Stroke::new(1.5, Color32::WHITE),
        );
    }
    ui.painter().line_segment(
        [
            pos2(inner.left(), inner.top()),
            pos2(inner.left(), inner.bottom()),
        ],
        Stroke::new(1.5, Color32::WHITE),
    );
    ui.painter().line_segment(
        [
            pos2(inner.right(), inner.top()),
            pos2(inner.right(), inner.bottom()),
        ],
        Stroke::new(1.5, Color32::WHITE),
    );
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

fn display_path(path: &Path) -> String {
    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

fn with_extension(path: PathBuf, extension: &str) -> PathBuf {
    if path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|existing| existing.eq_ignore_ascii_case(extension))
    {
        path
    } else {
        path.with_extension(extension)
    }
}

fn sanitize_file_stem(name: &str) -> String {
    let mut stem: String = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    while stem.contains("--") {
        stem = stem.replace("--", "-");
    }
    let stem = stem.trim_matches('-').to_owned();
    if stem.is_empty() {
        "rackcraft".to_owned()
    } else {
        stem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_stem_is_safe() {
        assert_eq!(
            sanitize_file_stem("Demo — Server Room A"),
            "demo-server-room-a"
        );
        assert_eq!(sanitize_file_stem("///"), "rackcraft");
        assert_eq!(sanitize_file_stem("A  B"), "a-b");
    }

    #[test]
    fn extension_is_only_added_once() {
        let path = PathBuf::from("inventory.xlsx");
        assert_eq!(
            with_extension(path, "xlsx"),
            PathBuf::from("inventory.xlsx")
        );
        assert_eq!(
            with_extension(PathBuf::from("inventory"), "xlsx"),
            PathBuf::from("inventory.xlsx")
        );
    }
}
