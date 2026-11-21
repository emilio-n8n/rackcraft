//! Built-in demonstration rack used by `--sample` and the test-suite.
//!
//! The document is entirely self-contained: each item carries normalized vector
//! artwork, connection points and shape data, so the sample opens even when no
//! `.vssx` file is available.

use crate::connector::{Connector, ConnectorEnd, LineStyle, Routing};
use crate::document::{
    Canvas, ConnectionPoint, Document, DrawPath, Item, MountSpec, PathCommand, ShapeField,
    StencilRef,
};
use crate::geometry::{Color, Point, Size};
use crate::units::{RACK_U_MM, RACK_WIDTH_19_IN_MM};

const LIBRARY: &str = "RackCraft sample";
const RACK_X: f64 = 80.0;
const RACK_Y: f64 = 80.0;
const RACK_W: f64 = 600.0;
const RACK_H: f64 = 1181.25;
const MOUNT_LEFT: f64 = 58.7;
const MOUNT_TOP: f64 = 35.0;
const MOUNT_RIGHT: f64 = 58.7;
const MOUNT_BOTTOM: f64 = 35.0;
const UNITS: u32 = 25;

const INK: Color = Color::rgb(12, 15, 20);
const PANEL: Color = Color::rgb(43, 49, 64);
const EDGE: Color = Color::rgb(94, 106, 128);
const MUTED: Color = Color::rgb(48, 55, 68);
const ACCENT: Color = Color::rgb(232, 87, 63);
const GREEN: Color = Color::rgb(84, 209, 132);
const GOLD: Color = Color::rgb(212, 175, 95);
const SFP: Color = Color::rgb(32, 37, 47);

struct Norm {
    size: Size,
}

impl Norm {
    fn new(size: Size) -> Self {
        Self { size }
    }

    fn p(&self, x: f64, y: f64) -> Point {
        Point::new(x / self.size.width, y / self.size.height)
    }

    fn rect_cmds(&self, x: f64, y: f64, w: f64, h: f64) -> Vec<PathCommand> {
        vec![
            PathCommand::MoveTo { to: self.p(x, y) },
            PathCommand::LineTo {
                to: self.p(x + w, y),
            },
            PathCommand::LineTo {
                to: self.p(x + w, y + h),
            },
            PathCommand::LineTo {
                to: self.p(x, y + h),
            },
            PathCommand::Close,
        ]
    }

    fn rect(&self, x: f64, y: f64, w: f64, h: f64, fill: Color, stroke: Option<Color>) -> DrawPath {
        let mut path = DrawPath::filled(self.rect_cmds(x, y, w, h), fill);
        if let Some(stroke) = stroke {
            path = path.with_stroke(stroke, 0.3);
        }
        path
    }

    fn line(&self, x1: f64, y1: f64, x2: f64, y2: f64, stroke: Color, width: f64) -> DrawPath {
        DrawPath {
            commands: vec![
                PathCommand::MoveTo { to: self.p(x1, y1) },
                PathCommand::LineTo { to: self.p(x2, y2) },
            ],
            fill: None,
            stroke: Some(stroke),
            stroke_width_mm: width,
        }
    }
}

fn rack_graphics() -> Vec<DrawPath> {
    let size = Size::new(RACK_W, RACK_H);
    let n = Norm::new(size);
    let mut paths = vec![
        n.rect(0.0, 0.0, RACK_W, RACK_H, Color::rgb(30, 34, 43), Some(EDGE)),
        n.rect(
            MOUNT_LEFT,
            MOUNT_TOP,
            RACK_WIDTH_19_IN_MM,
            f64::from(UNITS) * RACK_U_MM,
            Color::rgb(16, 19, 25),
            Some(MUTED),
        ),
        n.rect(6.0, 4.0, 210.0, 16.0, ACCENT, None),
        n.rect(RACK_W - 86.0, 6.0, 80.0, 12.0, MUTED, None),
    ];
    // Vertical rails with mounting holes.
    for side in 0..2 {
        let x = if side == 0 { 29.0 } else { RACK_W - 39.0 };
        paths.push(n.rect(
            x,
            MOUNT_TOP,
            10.0,
            f64::from(UNITS) * RACK_U_MM,
            PANEL,
            Some(MUTED),
        ));
        for unit in 0..UNITS {
            let y = MOUNT_TOP + f64::from(unit) * RACK_U_MM + 8.0;
            paths.push(n.rect(x + 3.0, y, 4.0, 4.0, INK, None));
        }
    }
    // Every rack-unit boundary.
    for unit in 0..=UNITS {
        let y = MOUNT_TOP + f64::from(unit) * RACK_U_MM;
        paths.push(n.line(
            MOUNT_LEFT,
            y,
            MOUNT_LEFT + RACK_WIDTH_19_IN_MM,
            y,
            Color::rgb(56, 64, 79),
            0.18,
        ));
    }
    // Feet.
    paths.push(n.rect(40.0, RACK_H - 16.0, 70.0, 16.0, PANEL, Some(EDGE)));
    paths.push(n.rect(RACK_W - 110.0, RACK_H - 16.0, 70.0, 16.0, PANEL, Some(EDGE)));
    paths
}

fn rack_connections() -> Vec<ConnectionPoint> {
    (0..UNITS)
        .map(|unit| {
            let y = (MOUNT_TOP + (f64::from(unit) + 0.5) * RACK_U_MM) / RACK_H;
            ConnectionPoint::new(format!("u-{}", UNITS - unit), 0.5, y)
                .named(format!("U{}", UNITS - unit))
        })
        .collect()
}

fn patch_panel_graphics() -> Vec<DrawPath> {
    let size = Size::new(RACK_WIDTH_19_IN_MM, RACK_U_MM);
    let n = Norm::new(size);
    let mut paths = vec![
        n.rect(0.0, 0.0, size.width, size.height, PANEL, Some(EDGE)),
        n.rect(
            2.0,
            3.0,
            58.0,
            size.height - 6.0,
            Color::rgb(24, 28, 37),
            Some(MUTED),
        ),
        n.rect(8.0, 9.0, 20.0, 4.0, ACCENT, None),
    ];
    for row in 0..2 {
        for column in 0..12 {
            let x = 72.0 + f64::from(column) * 32.0;
            let y = 6.0 + f64::from(row) * 22.0;
            paths.push(n.rect(x, y, 17.0, 14.0, INK, Some(Color::rgb(116, 127, 146))));
            paths.push(n.rect(x + 3.0, y + 3.0, 11.0, 5.0, Color::rgb(28, 33, 42), None));
            paths.push(n.rect(x + 14.5, y + 2.0, 1.5, 2.0, GREEN, None));
        }
    }
    for x in [8.0, size.width - 18.0] {
        paths.push(n.rect(x, 18.0, 10.0, 8.0, MUTED, Some(EDGE)));
    }
    paths
}

fn patch_panel_connections() -> Vec<ConnectionPoint> {
    let mut points = Vec::with_capacity(24);
    for row in 0..2 {
        for column in 0..12 {
            let x = (72.0 + f64::from(column) * 32.0 + 8.5) / RACK_WIDTH_19_IN_MM;
            let y = (6.0 + f64::from(row) * 22.0 + 7.0) / RACK_U_MM;
            let number = row * 12 + column + 1;
            points.push(
                ConnectionPoint::new(format!("port-{number}"), x, y)
                    .named(format!("Port {number}")),
            );
        }
    }
    points
}

fn switch_graphics() -> Vec<DrawPath> {
    let size = Size::new(RACK_WIDTH_19_IN_MM, RACK_U_MM);
    let n = Norm::new(size);
    let mut paths = vec![
        n.rect(
            0.0,
            0.0,
            size.width,
            size.height,
            Color::rgb(31, 37, 48),
            Some(EDGE),
        ),
        n.rect(10.0, 8.0, 26.0, 4.0, ACCENT, None),
    ];
    for column in 0..24 {
        let x = 61.0 + f64::from(column) * 16.6;
        let y = 23.0;
        paths.push(n.rect(x, y, 12.0, 13.0, INK, Some(Color::rgb(105, 117, 136))));
        paths.push(n.rect(x + 3.0, y + 3.0, 6.0, 4.0, Color::rgb(26, 32, 41), None));
        if column % 4 == 0 {
            paths.push(n.rect(x + 1.0, 17.0, 2.5, 2.0, GREEN, None));
        }
    }
    for sfp in 0..2 {
        let x = 429.0 + f64::from(sfp) * 23.0;
        paths.push(n.rect(x, 6.0, 18.0, 12.0, SFP, Some(EDGE)));
    }
    paths
}

fn switch_connections() -> Vec<ConnectionPoint> {
    (0..24)
        .map(|column| {
            let x = (61.0 + f64::from(column) * 16.6 + 6.0) / RACK_WIDTH_19_IN_MM;
            ConnectionPoint::new(format!("eth-{}", column + 1), x, 29.5 / RACK_U_MM)
                .named(format!("Ethernet {}", column + 1))
        })
        .collect()
}

fn cable_manager_graphics() -> Vec<DrawPath> {
    let size = Size::new(RACK_WIDTH_19_IN_MM, RACK_U_MM);
    let n = Norm::new(size);
    let mut paths = vec![n.rect(
        0.0,
        0.0,
        size.width,
        size.height,
        Color::rgb(35, 40, 51),
        Some(EDGE),
    )];
    for finger in 0..12 {
        let x = 26.0 + f64::from(finger) * 37.0;
        paths.push(n.rect(x, 12.0, 24.0, 20.0, INK, Some(MUTED)));
        paths.push(n.rect(x + 5.0, 16.0, 14.0, 2.0, MUTED, None));
    }
    paths
}

fn modular_panel_graphics() -> Vec<DrawPath> {
    let size = Size::new(RACK_WIDTH_19_IN_MM, RACK_U_MM);
    let n = Norm::new(size);
    let aperture = size.width / 24.0;
    let mut paths = vec![
        n.rect(0.0, 0.0, size.width, size.height, PANEL, Some(EDGE)),
        n.rect(
            4.0,
            5.0,
            42.0,
            size.height - 10.0,
            Color::rgb(24, 28, 37),
            Some(MUTED),
        ),
    ];
    for column in 0..24 {
        let x = f64::from(column) * aperture + 2.4;
        paths.push(n.rect(
            x,
            8.0,
            aperture - 4.8,
            28.0,
            INK,
            Some(Color::rgb(104, 116, 135)),
        ));
        paths.push(n.rect(x + 3.0, 32.0, aperture - 10.8, 2.0, ACCENT, None));
    }
    paths
}

fn modular_panel_connections() -> Vec<ConnectionPoint> {
    let aperture = RACK_WIDTH_19_IN_MM / 24.0;
    (0..24)
        .map(|column| {
            let x = (f64::from(column) + 0.5) * aperture / RACK_WIDTH_19_IN_MM;
            ConnectionPoint::new(format!("aperture-{}", column + 1), x, 0.5)
                .named(format!("Aperture {}", column + 1))
        })
        .collect()
}

fn keystone_graphics() -> Vec<DrawPath> {
    let size = Size::new(14.7, 17.8);
    let n = Norm::new(size);
    vec![
        n.rect(
            0.0,
            0.0,
            size.width,
            size.height,
            Color::rgb(225, 230, 238),
            Some(EDGE),
        ),
        n.rect(
            2.6,
            3.0,
            size.width - 5.2,
            10.5,
            INK,
            Some(Color::rgb(72, 80, 94)),
        ),
        n.rect(4.4, 6.0, 5.9, 4.2, Color::rgb(26, 31, 40), None),
        n.line(4.0, 13.0, 10.7, 13.0, GOLD, 0.35),
        n.rect(3.0, 15.0, 8.7, 1.4, GREEN, None),
    ]
}

fn keystone_connections() -> Vec<ConnectionPoint> {
    vec![ConnectionPoint::new("rj45", 0.5, 0.5).named("RJ45")]
}

fn field(label: &str, value: &str) -> ShapeField {
    ShapeField {
        label: label.to_owned(),
        value: value.to_owned(),
        prompt: String::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn item(
    master_id: &str,
    name: &str,
    category: &str,
    position: Point,
    size: Size,
    tags: &[&str],
    graphics: Vec<DrawPath>,
    connections: Vec<ConnectionPoint>,
    data: &[(&str, &str)],
) -> Item {
    let mut item = Item::new(
        StencilRef {
            library: LIBRARY.to_owned(),
            master_id: master_id.to_owned(),
            name: name.to_owned(),
        },
        name,
        position,
        size,
    );
    item.category = category.to_owned();
    item.tags = tags.iter().map(|tag| (*tag).to_owned()).collect();
    item.graphics = graphics;
    item.connection_points = connections;
    item.shape_data = data
        .iter()
        .map(|(label, value)| field(label, value))
        .collect();
    item
}

/// Build the sample rack document.
///
/// Layout: a 25U rack with a CAT6 patch panel, a 24-port switch, a cable
/// manager, a modular panel and six keystone jacks.
#[must_use]
pub fn document() -> Document {
    let mut doc = Document::new("Demo — Server Room A");
    doc.canvas = Canvas {
        width_mm: 760.0,
        height_mm: 1420.0,
        grid_mm: 10.0,
        show_grid: true,
        background: Color::rgb(21, 24, 30),
    };
    doc.metadata.insert(
        "description".into(),
        "Offline RackCraft sample: 25U rack, patch panel, switch, modular panel".into(),
    );
    doc.metadata.insert("revision".into(), "A".into());

    let interior_x = RACK_X + MOUNT_LEFT;
    let interior_top = RACK_Y + MOUNT_TOP;

    let mut rack = item(
        "rack-25u",
        "RackCraft 25U Open Frame Rack",
        "Racks",
        Point::new(RACK_X, RACK_Y),
        Size::new(RACK_W, RACK_H),
        &["rack", "infrastructure"],
        rack_graphics(),
        rack_connections(),
        &[
            ("Manufacturer", "RackCraft"),
            ("Part Number", "RC-RK-25U"),
            (
                "Product Description",
                "25U 19-inch open frame rack, 600 x 1181 mm",
            ),
            ("Rack Units", "25"),
        ],
    );
    rack.mount = Some(MountSpec {
        units: UNITS,
        left: MOUNT_LEFT / RACK_W,
        right: MOUNT_RIGHT / RACK_W,
        top: MOUNT_TOP / RACK_H,
        bottom: MOUNT_BOTTOM / RACK_H,
    });
    rack.project.location = "Server Room A".into();
    rack.project.building = "HQ-1".into();
    rack.project.installed_date = "2026-11-20".into();
    rack.project.warranty_exp = "2036-11-20".into();
    doc.add_item(rack);

    let rack_unit_y = |index: i32| interior_top + f64::from(index) * RACK_U_MM;
    let centered_x = interior_x;

    let mut panel = item(
        "patch-panel-24",
        "24-Port CAT6 Patch Panel",
        "Copper patch panels",
        Point::new(centered_x, rack_unit_y(1)),
        Size::new(RACK_WIDTH_19_IN_MM, RACK_U_MM),
        &["patch-panel", "copper", "infrastructure"],
        patch_panel_graphics(),
        patch_panel_connections(),
        &[
            ("Manufacturer", "RackCraft"),
            ("Part Number", "RC-PP-24C6"),
            (
                "Product Description",
                "1U 24-port CAT6 patch panel, 19 inch",
            ),
            ("Port Count", "24"),
        ],
    );
    panel.project.location = "Server Room A · U24".into();
    panel.project.building = "HQ-1".into();
    panel.project.installed_date = "2026-11-20".into();
    panel.project.warranty_exp = "2038-11-20".into();
    let panel_id = doc.add_item(panel);

    let mut switch = item(
        "switch-24",
        "24-Port Gigabit Switch",
        "Network switches",
        Point::new(centered_x, rack_unit_y(3)),
        Size::new(RACK_WIDTH_19_IN_MM, RACK_U_MM),
        &["switch", "network"],
        switch_graphics(),
        switch_connections(),
        &[
            ("Manufacturer", "RackCraft"),
            ("Part Number", "RC-SW-24G"),
            (
                "Product Description",
                "1U 24-port gigabit managed switch, 2 SFP",
            ),
            ("Port Count", "26"),
        ],
    );
    switch.project.location = "Server Room A · U22".into();
    switch.project.building = "HQ-1".into();
    switch.project.installed_date = "2026-11-20".into();
    switch.project.warranty_exp = "2031-11-20".into();
    let switch_id = doc.add_item(switch);

    // Four patched links from the CAT6 panel to the switch.
    for port in 0..4 {
        let mut cable = Connector::new(
            ConnectorEnd::ConnectionPoint {
                item_id: panel_id,
                connection_point: port,
            },
            ConnectorEnd::ConnectionPoint {
                item_id: switch_id,
                connection_point: port,
            },
        )
        .with_label("Cat6A")
        .with_routing(Routing::Orthogonal);
        cable.line_style = LineStyle::Solid;
        doc.add_connector(cable);
    }

    let mut manager = item(
        "cable-manager-1u",
        "1U Cable Manager",
        "Cable management",
        Point::new(centered_x, rack_unit_y(5)),
        Size::new(RACK_WIDTH_19_IN_MM, RACK_U_MM),
        &["cable-management"],
        cable_manager_graphics(),
        Vec::new(),
        &[
            ("Manufacturer", "RackCraft"),
            ("Part Number", "RC-CM-1U"),
            (
                "Product Description",
                "1U horizontal cable manager with 12 fingers",
            ),
        ],
    );
    manager.project.location = "Server Room A · U20".into();
    manager.project.building = "HQ-1".into();
    doc.add_item(manager);

    let modular_x = centered_x;
    let modular_y = rack_unit_y(8);
    let mut modular = item(
        "modular-panel-24",
        "24-Aperture Modular Patch Panel",
        "Modular patch panels",
        Point::new(modular_x, modular_y),
        Size::new(RACK_WIDTH_19_IN_MM, RACK_U_MM),
        &["modular-panel", "patch-panel", "infrastructure"],
        modular_panel_graphics(),
        modular_panel_connections(),
        &[
            ("Manufacturer", "RackCraft"),
            ("Part Number", "RC-MPP-24"),
            (
                "Product Description",
                "1U 24-aperture modular patch panel, empty",
            ),
            ("Aperture Count", "24"),
        ],
    );
    modular.project.location = "Server Room A · U17".into();
    modular.project.building = "HQ-1".into();
    doc.add_item(modular);

    let aperture_width = RACK_WIDTH_19_IN_MM / 24.0;
    for slot in 0..6 {
        let keystone_x =
            modular_x + f64::from(slot) * aperture_width + (aperture_width - 14.7) * 0.5;
        let keystone_y = modular_y + (RACK_U_MM - 17.8) * 0.5;
        let mut keystone = item(
            "keystone-rj45",
            "CAT6 Keystone Jack RJ45",
            "Keystone jacks",
            Point::new(keystone_x, keystone_y),
            Size::new(14.7, 17.8),
            &["keystone", "rj45", "copper"],
            keystone_graphics(),
            keystone_connections(),
            &[
                ("Manufacturer", "RackCraft"),
                ("Part Number", "RC-KJ-RJ45"),
                ("Product Description", "CAT6 keystone jack, RJ45, 180°"),
                ("Colour", "White"),
            ],
        );
        keystone.project.location = format!("Server Room A · U17 · Port {}", slot + 1);
        keystone.project.building = "HQ-1".into();
        keystone.project.installed_date = "2026-11-20".into();
        keystone.project.notes = "Patched to switch port group A".into();
        doc.add_item(keystone);
    }

    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_has_expected_inventory() {
        let doc = document();
        assert_eq!(doc.items.len(), 11);
        let bom = doc.bom();
        assert_eq!(bom.total_quantity, 11);
        assert_eq!(bom.distinct_parts(), 6);
        let keystones = bom
            .lines
            .iter()
            .find(|line| line.part_number == "RC-KJ-RJ45")
            .unwrap();
        assert_eq!(keystones.quantity, 6);
    }

    #[test]
    fn sample_has_patched_cables() {
        let doc = document();
        assert_eq!(doc.connectors.len(), 4);
        let labels: Vec<&str> = doc
            .connectors
            .iter()
            .map(|connector| connector.label.as_str())
            .collect();
        assert!(labels.iter().all(|label| *label == "Cat6A"));
        assert!(
            doc.connectors
                .iter()
                .all(|connector| doc.resolve_connector(connector).is_some())
        );
    }

    #[test]
    fn sample_has_racks_and_ports() {
        let doc = document();
        let rack = doc.items.iter().find(|item| item.is_rack()).unwrap();
        assert_eq!(rack.effective_mount().unwrap().units, 25);
        let panel = doc
            .items
            .iter()
            .find(|item| item.master.master_id == "patch-panel-24")
            .unwrap();
        assert_eq!(panel.connection_points.len(), 24);
        let modular = doc
            .items
            .iter()
            .find(|item| item.master.master_id == "modular-panel-24")
            .unwrap();
        assert_eq!(modular.connection_points.len(), 24);
    }

    #[test]
    fn sample_serializes_and_reloads() {
        let doc = document();
        let json = doc.to_json_pretty().unwrap();
        let loaded = Document::from_json(&json).unwrap();
        assert_eq!(loaded, doc);
    }

    #[test]
    fn sample_items_fit_inside_the_rack() {
        let doc = document();
        let rack = doc.items.iter().find(|item| item.is_rack()).unwrap();
        let interior = rack.effective_mount().unwrap().interior(rack.bounds());
        for item in doc.items.iter().filter(|item| !item.is_rack()) {
            let bounds = item.bounds();
            assert!(
                bounds.left() >= interior.left() - 0.01
                    && bounds.right() <= interior.right() + 0.01
                    && bounds.top() >= interior.top() - 0.01
                    && bounds.bottom() <= interior.bottom() + 0.01,
                "{} is outside the rack: {:?}",
                item.name,
                bounds
            );
        }
    }
}
