//! Integrated bill-of-materials window.

use std::collections::BTreeSet;

use eframe::egui::{self, Align, Color32, CornerRadius, FontId, Layout, RichText, Stroke};
use rackcraft_engine::Bom;

use crate::theme;

/// Requested export from the inventory window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryAction {
    /// No action.
    None,
    /// Export `.xlsx`.
    Excel,
    /// Export `.csv`.
    Csv,
    /// Close the window.
    Close,
}

/// Display the BOM. `open` is updated by the window close button.
pub fn show(ctx: &egui::Context, open: &mut bool, bom: &Bom, title: &str) -> InventoryAction {
    let mut action = InventoryAction::None;
    let mut is_open = *open;
    let locations: BTreeSet<&str> = bom
        .lines
        .iter()
        .flat_map(|line| line.locations.iter().map(String::as_str))
        .collect();

    egui::Window::new("Inventory")
        .open(&mut is_open)
        .default_width(980.0)
        .min_width(720.0)
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.set_min_width(920.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(18.0).strong().color(theme::TEXT));
                    ui.label(
                        RichText::new("Grouped by manufacturer, part number and description")
                            .size(10.5)
                            .color(theme::MUTED),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    theme::badge(ui, "LIVE BOM", theme::SNAP);
                });
            });
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                metric(
                    ui,
                    "Placed items",
                    &bom.total_quantity.to_string(),
                    theme::ACCENT,
                );
                metric(
                    ui,
                    "Distinct parts",
                    &bom.distinct_parts().to_string(),
                    theme::SNAP,
                );
                metric(
                    ui,
                    "Locations",
                    &locations.len().to_string(),
                    theme::WARNING,
                );
                let largest = bom
                    .lines
                    .iter()
                    .map(|line| line.quantity)
                    .max()
                    .unwrap_or(0);
                metric(ui, "Largest group", &largest.to_string(), theme::MUTED);
            });
            ui.add_space(10.0);

            if bom.lines.is_empty() {
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .stroke(Stroke::new(1.0, theme::BORDER))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::same(20))
                    .show(ui, |ui| {
                        ui.label(RichText::new("No equipment placed yet").strong());
                        ui.label(
                            RichText::new(
                                "Drop stencils on the canvas to build the inventory. \
                                 Empty stencils still appear in the BOM with their shape data.",
                            )
                            .size(11.0)
                            .color(theme::MUTED),
                        );
                    });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(380.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Grid::new("bom-grid")
                            .num_columns(5)
                            .striped(true)
                            .spacing([16.0, 8.0])
                            .min_col_width(70.0)
                            .show(ui, |ui| {
                                header(ui, "Manufacturer");
                                header(ui, "Part Number");
                                header(ui, "Product Description");
                                header(ui, "Qty");
                                header(ui, "Locations");
                                ui.end_row();

                                for line in &bom.lines {
                                    ui.label(
                                        RichText::new(&line.manufacturer)
                                            .size(11.0)
                                            .color(theme::TEXT),
                                    );
                                    ui.label(
                                        RichText::new(if line.part_number.is_empty() {
                                            "—"
                                        } else {
                                            &line.part_number
                                        })
                                        .monospace()
                                        .size(10.5)
                                        .color(theme::ACCENT_LIGHT),
                                    );
                                    ui.label(
                                        RichText::new(&line.description)
                                            .size(11.0)
                                            .color(theme::MUTED),
                                    );
                                    ui.label(
                                        RichText::new(line.quantity.to_string())
                                            .size(12.0)
                                            .strong()
                                            .color(theme::TEXT),
                                    );
                                    let location = if line.locations.is_empty() {
                                        "—".to_owned()
                                    } else {
                                        line.locations.join("; ")
                                    };
                                    ui.label(
                                        RichText::new(location).size(10.0).color(theme::FAINT),
                                    );
                                    ui.end_row();
                                }
                            });
                    });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("Total quantity: {}", bom.total_quantity))
                            .size(11.5)
                            .strong()
                            .color(theme::TEXT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Excel export includes a formatted inventory sheet")
                                .size(9.5)
                                .color(theme::FAINT),
                        );
                    });
                });
            }

            ui.add_space(10.0);
            theme::divider(ui);
            ui.horizontal(|ui| {
                let excel = ui.add(
                    egui::Button::new(
                        RichText::new("Export Excel (.xlsx)")
                            .size(12.0)
                            .color(Color32::WHITE)
                            .strong(),
                    )
                    .fill(theme::ACCENT)
                    .corner_radius(CornerRadius::same(6))
                    .min_size(egui::vec2(170.0, 30.0)),
                );
                if excel.clicked() {
                    action = InventoryAction::Excel;
                }
                let csv = ui.add(
                    egui::Button::new(RichText::new("Export CSV").size(12.0))
                        .fill(theme::RAISED)
                        .corner_radius(CornerRadius::same(6))
                        .min_size(egui::vec2(110.0, 30.0)),
                );
                if csv.clicked() {
                    action = InventoryAction::Csv;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        action = InventoryAction::Close;
                    }
                });
            });
        });

    if !is_open {
        action = InventoryAction::Close;
    }
    *open = is_open && action != InventoryAction::Close;
    action
}

fn header(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(9.0)
            .strong()
            .color(theme::FAINT),
    );
}

fn metric(ui: &mut egui::Ui, label: &str, value: &str, colour: Color32) {
    let available = (ui.available_width() - 36.0) / 4.0;
    egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(11, 7))
        .show(ui, |ui| {
            ui.set_width(available.max(90.0));
            ui.label(
                RichText::new(value)
                    .font(FontId::monospace(20.0))
                    .color(colour)
                    .strong(),
            );
            ui.label(
                RichText::new(label.to_uppercase())
                    .size(8.5)
                    .color(theme::FAINT),
            );
        });
}

/// Small helper for future inventory tests / summaries.
#[cfg(test)]
#[must_use]
pub fn location_count(bom: &Bom) -> usize {
    let mut set = BTreeSet::new();
    for line in &bom.lines {
        for location in &line.locations {
            set.insert(location.as_str());
        }
    }
    set.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rackcraft_engine::{Document, Item, Point, Size, StencilRef};

    #[test]
    fn location_count_deduplicates() {
        let mut doc = Document::new("Count");
        for location in ["A", "B", "A"] {
            let mut item = Item::new(
                StencilRef::default(),
                "Panel",
                Point::default(),
                Size::new(10.0, 10.0),
            );
            item.project.location = location.into();
            doc.add_item(item);
        }
        assert_eq!(location_count(&doc.bom()), 2);
    }
}
