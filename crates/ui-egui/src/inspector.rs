//! Right-hand shape-data inspector.

use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke};
use rackcraft_engine::{
    Arrow, Color, Connector, ConnectorEnd, ConnectorId, Document, Item, ItemId, LineStyle, Routing,
    ShapeField,
};

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

/// Editable copy of connector properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectorForm {
    /// Connector being edited.
    pub connector_id: ConnectorId,
    /// Cable label.
    pub label: String,
    /// Stroke colour.
    pub color: Color,
    /// Straight or orthogonal routing.
    pub routing: Routing,
    /// Solid or dashed.
    pub line_style: LineStyle,
    /// Arrowheads.
    pub arrow: Arrow,
}

impl ConnectorForm {
    /// Build a form from a connector.
    #[must_use]
    pub fn from_connector(connector: &Connector) -> Self {
        Self {
            connector_id: connector.id,
            label: connector.label.clone(),
            color: connector.color,
            routing: connector.routing,
            line_style: connector.line_style,
            arrow: connector.arrow,
        }
    }
}

/// Action requested by the connector inspector.
#[derive(Debug, Clone)]
pub enum ConnectorAction {
    /// No action.
    None,
    /// Apply the edited connector properties.
    Apply(ConnectorForm),
    /// Delete the connector.
    Delete,
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

/// Draw the connector editor in the right-hand panel.
pub fn show_connector(
    ui: &mut egui::Ui,
    document: &Document,
    id: ConnectorId,
    form: &mut Option<ConnectorForm>,
) -> ConnectorAction {
    let mut action = ConnectorAction::None;
    ui.horizontal(|ui| {
        theme::section_label(ui, "Connector");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            theme::badge(ui, &format!("#{id}"), theme::FAINT);
        });
    });
    ui.add_space(4.0);
    let Some(connector) = document.connector(id) else {
        ui.label(RichText::new("Connector no longer exists").color(theme::MUTED));
        return action;
    };
    if form.as_ref().map(|form| form.connector_id) != Some(id) {
        *form = Some(ConnectorForm::from_connector(connector));
    }
    let Some(form) = form.as_mut() else {
        return action;
    };
    let dirty = *form != ConnectorForm::from_connector(connector);

    egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(CornerRadius::same(7))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(if connector.label.is_empty() {
                    "Untitled link"
                } else {
                    &connector.label
                })
                .size(14.0)
                .strong()
                .color(theme::TEXT),
            );
            ui.label(
                RichText::new(format!(
                    "{} -> {}",
                    describe_end(document, &connector.from),
                    describe_end(document, &connector.to)
                ))
                .size(9.5)
                .color(theme::MUTED),
            );
            ui.add_space(3.0);
            ui.label(
                RichText::new(
                    "Attached ends follow equipment moves; deleted equipment leaves a free anchor.",
                )
                .size(9.0)
                .color(theme::FAINT),
            );
        });

    ui.add_space(8.0);
    theme::section_label(ui, "Link");
    ui.label(RichText::new("Label").size(10.0).color(theme::MUTED));
    ui.add(
        egui::TextEdit::singleline(&mut form.label)
            .hint_text("Cat6A, 10G SFP+, WAN…")
            .desired_width(f32::INFINITY),
    );

    ui.add_space(5.0);
    ui.label(RichText::new("Color").size(10.0).color(theme::MUTED));
    ui.horizontal(|ui| {
        let mut color =
            Color32::from_rgba_unmultiplied(form.color.r, form.color.g, form.color.b, form.color.a);
        if ui.color_edit_button_srgba(&mut color).changed() {
            form.color = Color::rgba(color.r(), color.g(), color.b(), color.a());
        }
        color_swatch(ui, Color::rgb(84, 209, 132), &mut form.color);
        color_swatch(ui, Color::rgb(232, 87, 63), &mut form.color);
        color_swatch(ui, Color::rgb(90, 167, 232), &mut form.color);
        color_swatch(ui, Color::rgb(240, 180, 90), &mut form.color);
        color_swatch(ui, Color::rgb(235, 239, 246), &mut form.color);
    });

    ui.add_space(5.0);
    ui.label(RichText::new("Routing").size(10.0).color(theme::MUTED));
    ui.horizontal(|ui| {
        ui.radio_value(&mut form.routing, Routing::Straight, "Straight");
        ui.radio_value(&mut form.routing, Routing::Orthogonal, "Orthogonal");
    });
    ui.label(RichText::new("Line style").size(10.0).color(theme::MUTED));
    ui.horizontal(|ui| {
        ui.radio_value(&mut form.line_style, LineStyle::Solid, "Solid");
        ui.radio_value(&mut form.line_style, LineStyle::Dashed, "Dashed");
    });
    ui.label(RichText::new("Arrows").size(10.0).color(theme::MUTED));
    ui.horizontal(|ui| {
        ui.radio_value(&mut form.arrow, Arrow::None, "None");
        ui.radio_value(&mut form.arrow, Arrow::End, "End");
        ui.radio_value(&mut form.arrow, Arrow::Double, "Both");
    });

    ui.add_space(6.0);
    if dirty {
        ui.label(
            RichText::new("● Unsaved connector edits")
                .size(9.5)
                .color(theme::WARNING),
        );
    }
    ui.horizontal(|ui| {
        let apply = ui.add(
            egui::Button::new(
                RichText::new("Apply link")
                    .size(11.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(if dirty { theme::ACCENT } else { theme::RAISED })
            .corner_radius(CornerRadius::same(5))
            .min_size(egui::vec2(110.0, 28.0)),
        );
        if apply.clicked() {
            action = ConnectorAction::Apply(form.clone());
        }
        if ui
            .add(
                egui::Button::new(RichText::new("Delete").size(10.5).color(theme::ERROR))
                    .fill(theme::RAISED)
                    .corner_radius(CornerRadius::same(5)),
            )
            .on_hover_text("Delete this connector (Del)")
            .clicked()
        {
            action = ConnectorAction::Delete;
        }
    });
    action
}

fn color_swatch(ui: &mut egui::Ui, colour: Color, current: &mut Color) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::click());
    ui.painter().rect(
        rect,
        CornerRadius::same(4),
        Color32::from_rgba_unmultiplied(colour.r, colour.g, colour.b, colour.a),
        if *current == colour {
            Stroke::new(2.0, Color32::WHITE)
        } else {
            Stroke::new(1.0, theme::BORDER)
        },
        egui::StrokeKind::Inside,
    );
    if response.clicked() {
        *current = colour;
    }
    response.on_hover_text("Use this color");
}

fn describe_end(document: &Document, end: &ConnectorEnd) -> String {
    match end {
        ConnectorEnd::Free { point } => format!("Free {:.0}, {:.0} mm", point.x, point.y),
        ConnectorEnd::ConnectionPoint {
            item_id,
            connection_point,
        } => document.item(*item_id).map_or_else(
            || "Missing equipment".to_owned(),
            |item| {
                let name = item
                    .connection_points
                    .get(*connection_point)
                    .and_then(|connection| connection.name.as_deref())
                    .unwrap_or("Connection point");
                format!("{} · {name}", item.name)
            },
        ),
    }
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
        "V selects; C draws cables between connection points.",
        "Click a cable in Select mode to edit its label and style.",
        "Green rings show rack-U and port snapping.",
        "Ctrl+E exports a vector PDF; Ctrl+Z undoes.",
    ] {
        ui.label(
            RichText::new(format!("· {tip}"))
                .size(10.0)
                .color(theme::MUTED),
        );
    }
}
