//! Right-hand shape-data inspector.

use eframe::egui::{self, CornerRadius, RichText, Stroke};
use rackcraft_engine::{Document, Item, ItemId, ShapeField};

use crate::theme;

/// Editable copy of the fields shown in the inspector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorForm {
    /// Item this form belongs to.
    pub item_id: ItemId,
    /// Stencil manufacturer field.
    pub manufacturer: String,
    /// Stencil part-number field.
    pub part_number: String,
    /// Stencil product-description field.
    pub description: String,
    /// Project installation date.
    pub installed_date: String,
    /// Project warranty expiry.
    pub warranty_exp: String,
    /// Project location.
    pub location: String,
    /// Project building.
    pub building: String,
    /// Project notes.
    pub notes: String,
}

impl InspectorForm {
    /// Build a form from the current item values.
    #[must_use]
    pub fn from_item(item: &Item) -> Self {
        Self {
            item_id: item.id,
            manufacturer: item
                .shape_value_any(&["Manufacturer", "Vendor", "Brand"])
                .unwrap_or("")
                .to_owned(),
            part_number: item
                .shape_value_any(&[
                    "Part Number",
                    "PartNumber",
                    "Part No.",
                    "Part No",
                    "Product Number",
                    "SKU",
                ])
                .unwrap_or("")
                .to_owned(),
            description: item
                .shape_value_any(&["Product Description", "Description", "Product", "Model"])
                .unwrap_or("")
                .to_owned(),
            installed_date: item.project.installed_date.clone(),
            warranty_exp: item.project.warranty_exp.clone(),
            location: item.project.location.clone(),
            building: item.project.building.clone(),
            notes: item.project.notes.clone(),
        }
    }

    /// Whether the form differs from the item on the canvas.
    #[must_use]
    pub fn is_dirty(&self, item: &Item) -> bool {
        &Self::from_item(item) != self
    }
}

/// Action requested by the inspector.
#[derive(Debug, Clone)]
pub enum InspectorAction {
    /// No action.
    None,
    /// Apply the edited fields.
    Apply(InspectorForm),
    /// Delete the selected item.
    Delete,
    /// Duplicate the selected item.
    Duplicate,
}

/// Draw the inspector panel.
pub fn show(
    ui: &mut egui::Ui,
    document: &Document,
    selection: Option<ItemId>,
    form: &mut Option<InspectorForm>,
) -> InspectorAction {
    let mut action = InspectorAction::None;

    ui.horizontal(|ui| {
        theme::section_label(ui, "Shape data");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(item) = selection.and_then(|id| document.item(id)) {
                theme::badge(ui, &format!("#{}", item.id), theme::FAINT);
            }
        });
    });
    ui.add_space(4.0);

    let Some(id) = selection else {
        empty_state(ui, document);
        return action;
    };
    let Some(item) = document.item(id) else {
        empty_state(ui, document);
        return action;
    };

    if form.as_ref().map(|form| form.item_id) != Some(id) {
        *form = Some(InspectorForm::from_item(item));
    }
    let Some(form) = form.as_mut() else {
        return action;
    };
    let dirty = form.is_dirty(item);

    egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(CornerRadius::same(7))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(&item.name)
                    .size(14.0)
                    .strong()
                    .color(theme::TEXT),
            );
            ui.label(
                RichText::new(format!("{} · {}", item.category, item.master.library))
                    .size(10.0)
                    .color(theme::MUTED),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{:.1} × {:.1} mm",
                        item.size.width, item.size.height
                    ))
                    .monospace()
                    .size(9.5)
                    .color(theme::FAINT),
                );
                if item.is_rack() {
                    theme::badge(
                        ui,
                        &format!(
                            "{}U RACK",
                            item.effective_mount().map_or(0, |mount| mount.units)
                        ),
                        theme::ACCENT,
                    );
                }
                if !item.connection_points.is_empty() {
                    theme::badge(
                        ui,
                        &format!("{} PORTS", item.connection_points.len()),
                        theme::SNAP,
                    );
                }
            });
        });

    ui.add_space(8.0);
    theme::section_label(ui, "Manufacturer");
    field(ui, "Manufacturer", "e.g. RackCraft", &mut form.manufacturer);
    field(ui, "Part Number", "e.g. RC-PP-24C6", &mut form.part_number);
    field(
        ui,
        "Product Description",
        "Product name and key specs",
        &mut form.description,
    );

    ui.add_space(8.0);
    theme::section_label(ui, "Project");
    field(ui, "Installed Date", "YYYY-MM-DD", &mut form.installed_date);
    field(ui, "Warranty Exp", "YYYY-MM-DD", &mut form.warranty_exp);
    field(ui, "Location", "Room, row, rack unit…", &mut form.location);
    field(ui, "Building", "Site / building", &mut form.building);
    ui.label(RichText::new("Notes").size(10.0).color(theme::MUTED));
    ui.add(
        egui::TextEdit::multiline(&mut form.notes)
            .hint_text("Cabling notes, ticket reference…")
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );

    ui.add_space(6.0);
    if dirty {
        ui.label(
            RichText::new("● Unsaved field edits")
                .size(9.5)
                .color(theme::WARNING),
        );
    } else {
        ui.label(
            RichText::new("Fields match the placed item")
                .size(9.5)
                .color(theme::FAINT),
        );
    }
    ui.add_space(5.0);
    ui.horizontal(|ui| {
        let apply = ui.add(
            egui::Button::new(
                RichText::new(if dirty { "Apply changes" } else { "Apply" })
                    .size(11.5)
                    .strong()
                    .color(egui::Color32::WHITE),
            )
            .fill(if dirty { theme::ACCENT } else { theme::RAISED })
            .corner_radius(CornerRadius::same(5))
            .min_size(egui::vec2(120.0, 28.0)),
        );
        if apply.clicked() {
            action = InspectorAction::Apply(form.clone());
        }
        if ui
            .add(
                egui::Button::new(RichText::new("Revert").size(11.0))
                    .fill(theme::RAISED)
                    .corner_radius(CornerRadius::same(5)),
            )
            .clicked()
        {
            *form = InspectorForm::from_item(item);
        }
    });

    ui.add_space(9.0);
    theme::divider(ui);
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(RichText::new("Duplicate").size(10.5))
                    .fill(theme::RAISED)
                    .corner_radius(CornerRadius::same(5)),
            )
            .on_hover_text("Duplicate this item (Ctrl+D)")
            .clicked()
        {
            action = InspectorAction::Duplicate;
        }
        if ui
            .add(
                egui::Button::new(RichText::new("Delete").size(10.5).color(theme::ERROR))
                    .fill(theme::RAISED)
                    .corner_radius(CornerRadius::same(5)),
            )
            .on_hover_text("Remove this item (Del)")
            .clicked()
        {
            action = InspectorAction::Delete;
        }
    });

    if item.shape_data.len() > 3 {
        ui.add_space(6.0);
        egui::CollapsingHeader::new(
            RichText::new(format!("All stencil fields ({})", item.shape_data.len()))
                .size(10.0)
                .color(theme::MUTED),
        )
        .id_salt(("shape-data", item.id))
        .default_open(false)
        .show(ui, |ui| {
            for ShapeField { label, value, .. } in &item.shape_data {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!("{label}:"))
                            .size(9.5)
                            .color(theme::FAINT),
                    );
                    ui.label(RichText::new(value).size(9.5).color(theme::MUTED));
                });
            }
        });
    }

    action
}

fn field(ui: &mut egui::Ui, label: &str, hint: &str, value: &mut String) {
    ui.label(RichText::new(label).size(10.0).color(theme::MUTED));
    ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(hint)
            .desired_width(f32::INFINITY),
    );
    ui.add_space(2.0);
}

fn empty_state(ui: &mut egui::Ui, document: &Document) {
    egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(CornerRadius::same(7))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.label(RichText::new("No selection").strong().color(theme::TEXT));
            ui.label(
                RichText::new(
                    "Select an equipment item on the canvas to edit its manufacturer data and project metadata.",
                )
                .size(11.0)
                .color(theme::MUTED),
            );
        });
    ui.add_space(10.0);
    theme::section_label(ui, "Document");
    ui.label(RichText::new(&document.name).size(12.0).color(theme::TEXT));
    let bom = document.bom();
    ui.label(
        RichText::new(format!(
            "{} items · {} distinct parts · {} U rack height",
            document.items.len(),
            bom.distinct_parts(),
            document
                .items
                .iter()
                .filter_map(|item| item.effective_mount())
                .map(|mount| mount.units)
                .sum::<u32>()
        ))
        .size(10.0)
        .color(theme::FAINT),
    );
    ui.add_space(10.0);
    theme::section_label(ui, "Tips");
    for tip in [
        "Wheel zooms under the cursor.",
        "Middle-drag or space-drag pans.",
        "Green rings show rack-U and port snapping.",
        "Ctrl+Z undoes, Ctrl+D duplicates, Del removes.",
    ] {
        ui.label(
            RichText::new(format!("· {tip}"))
                .size(10.0)
                .color(theme::MUTED),
        );
    }
}
