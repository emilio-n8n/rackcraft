//! Stencil library and master data model shared by the UI and exporters.

use std::collections::BTreeMap;

use rackcraft_engine::{
    Color, ConnectionPoint, DrawPath, Item, MountSpec, PathCommand, Point, ProjectFields,
    ShapeField, Size, StencilRef,
};
use serde::{Deserialize, Serialize};

/// A decoded master thumbnail (32-bit RGBA, row-major).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IconImage {
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// RGBA bytes, `width * height * 4`.
    pub rgba: Vec<u8>,
}

/// A parsed Visio master.
#[derive(Debug, Clone, PartialEq)]
pub struct Master {
    /// Master ID inside the package.
    pub id: String,
    /// Display name (`Name`/`NameU`).
    pub name: String,
    /// Category used by the stencil browser.
    pub category: String,
    /// Native width in millimetres.
    pub width_mm: f64,
    /// Native height in millimetres.
    pub height_mm: f64,
    /// Self-contained normalized artwork.
    pub graphics: Vec<DrawPath>,
    /// Normalized connection points.
    pub connection_points: Vec<ConnectionPoint>,
    /// Shape data template rows.
    pub shape_data: Vec<ShapeField>,
    /// Inferred search tags.
    pub tags: Vec<String>,
    /// Rack mounting specification, when the master is a rack.
    pub mount: Option<MountSpec>,
    /// Lowercase searchable text (name + category + shape data).
    pub keywords: String,
    /// Optional thumbnail decoded from the embedded ICO.
    pub icon: Option<IconImage>,
}

impl Master {
    /// Native size as a `Size`.
    #[must_use]
    pub fn size(&self) -> Size {
        Size::new(self.width_mm, self.height_mm)
    }

    /// Convert the normalized vector artwork to `kurbo` Bézier paths at native
    /// size (millimetres). This is the interchange form used by future SVG/PDF
    /// exporters and by geometry tests.
    #[must_use]
    pub fn outline_paths(&self) -> Vec<kurbo::BezPath> {
        self.graphics
            .iter()
            .map(|path| {
                let mut bez = kurbo::BezPath::new();
                for command in &path.commands {
                    match *command {
                        PathCommand::MoveTo { to } => {
                            bez.move_to(kurbo::Point::new(
                                to.x * self.width_mm,
                                to.y * self.height_mm,
                            ));
                        }
                        PathCommand::LineTo { to } => {
                            bez.line_to(kurbo::Point::new(
                                to.x * self.width_mm,
                                to.y * self.height_mm,
                            ));
                        }
                        PathCommand::QuadTo { ctrl, to } => {
                            bez.quad_to(
                                kurbo::Point::new(ctrl.x * self.width_mm, ctrl.y * self.height_mm),
                                kurbo::Point::new(to.x * self.width_mm, to.y * self.height_mm),
                            );
                        }
                        PathCommand::CubicTo { c1, c2, to } => {
                            bez.curve_to(
                                kurbo::Point::new(c1.x * self.width_mm, c1.y * self.height_mm),
                                kurbo::Point::new(c2.x * self.width_mm, c2.y * self.height_mm),
                                kurbo::Point::new(to.x * self.width_mm, to.y * self.height_mm),
                            );
                        }
                        PathCommand::Close => bez.close_path(),
                    }
                }
                bez
            })
            .collect()
    }

    /// Whether a search query matches this master.
    #[must_use]
    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return true;
        }
        query
            .split_whitespace()
            .all(|term| self.keywords.contains(term))
    }

    /// Instantiate this master at a document position.
    ///
    /// Vector artwork, connection points and shape data are copied so the
    /// document remains self-contained even if the `.vssx` file disappears.
    #[must_use]
    pub fn to_item(&self, library: &str, position: Point) -> Item {
        let mut item = Item::new(
            StencilRef {
                library: library.to_owned(),
                master_id: self.id.clone(),
                name: self.name.clone(),
            },
            self.name.clone(),
            position,
            self.size(),
        );
        item.category = self.category.clone();
        item.graphics = self.graphics.clone();
        item.connection_points = self.connection_points.clone();
        item.shape_data = self.shape_data.clone();
        item.tags = self.tags.clone();
        item.mount = self.mount.clone();

        let mut project = ProjectFields::default();
        let mut remaining = Vec::with_capacity(item.shape_data.len());
        for field in item.shape_data.drain(..) {
            let label = field.label.trim();
            if label.eq_ignore_ascii_case("Installed Date") {
                project.installed_date = field.value;
            } else if label.eq_ignore_ascii_case("Warranty Exp")
                || label.eq_ignore_ascii_case("Warranty Expiry")
                || label.eq_ignore_ascii_case("Warranty")
            {
                project.warranty_exp = field.value;
            } else if label.eq_ignore_ascii_case("Location") {
                project.location = field.value;
            } else if label.eq_ignore_ascii_case("Building") {
                project.building = field.value;
            } else if label.eq_ignore_ascii_case("Notes") || label.eq_ignore_ascii_case("Comments")
            {
                project.notes = field.value;
            } else {
                remaining.push(field);
            }
        }
        item.shape_data = remaining;
        item.project = project;
        item
    }
}

/// A stencil library (one `.vssx` file).
#[derive(Debug, Clone, PartialEq)]
pub struct StencilLibrary {
    /// Display name (normally the file stem).
    pub name: String,
    /// Original file location, when loaded from disk.
    pub source: Option<String>,
    /// Parsed masters in document order.
    pub masters: Vec<Master>,
    /// Non-fatal diagnostics (skipped malformed masters, unsupported shapes…).
    pub warnings: Vec<String>,
}

impl StencilLibrary {
    /// Find a master by package id.
    #[must_use]
    pub fn master(&self, id: &str) -> Option<&Master> {
        self.masters.iter().find(|master| master.id == id)
    }

    /// Group masters by category, preserving alphabetical category order.
    #[must_use]
    pub fn by_category(&self) -> BTreeMap<&str, Vec<&Master>> {
        let mut grouped: BTreeMap<&str, Vec<&Master>> = BTreeMap::new();
        for master in &self.masters {
            grouped
                .entry(master.category.as_str())
                .or_default()
                .push(master);
        }
        grouped
    }

    /// Search masters by name, category, tags and shape data.
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&Master> {
        self.masters
            .iter()
            .filter(|master| master.matches(query))
            .collect()
    }
}

/// What kind of Visio file was detected from the bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilFormat {
    /// Modern Open XML stencil.
    Vssx,
    /// Legacy OLE/CFB binary stencil.
    Vss,
    /// Something else.
    Unknown,
}

/// Detect a stencil container from its first bytes.
#[must_use]
pub fn detect_format(bytes: &[u8]) -> StencilFormat {
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        StencilFormat::Vssx
    } else if bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
        StencilFormat::Vss
    } else {
        StencilFormat::Unknown
    }
}

/// Category inference for stencils without an explicit category field.
#[must_use]
pub fn infer_category(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("rack") || lower.contains("cabinet") {
        "Racks & cabinets".into()
    } else if lower.contains("patch panel") || lower.contains("patchpanel") {
        "Patch panels".into()
    } else if lower.contains("modular") {
        "Modular panels".into()
    } else if lower.contains("keystone") || lower.contains("jack") {
        "Keystone jacks".into()
    } else if lower.contains("switch") || lower.contains("router") || lower.contains("firewall") {
        "Network".into()
    } else if lower.contains("cable") || lower.contains("manager") || lower.contains("tray") {
        "Cable management".into()
    } else if lower.contains("pdu") || lower.contains("power") || lower.contains("ups") {
        "Power".into()
    } else if lower.contains("server") {
        "Servers".into()
    } else {
        "General".into()
    }
}

/// Tags inferred from a master name and category.
#[must_use]
pub fn infer_tags(name: &str, category: &str) -> Vec<String> {
    let mut text = format!("{name} {category}").to_lowercase();
    let mut tags = Vec::new();
    let mut add = |tag: &str, needles: &[&str]| {
        if needles.iter().any(|needle| text.contains(needle)) {
            tags.push(tag.to_owned());
        }
    };
    add("rack", &["rack", "cabinet"]);
    add("patch-panel", &["patch panel", "patch-panel"]);
    add("modular-panel", &["modular"]);
    add("keystone", &["keystone", "jack"]);
    add("switch", &["switch"]);
    add("router", &["router"]);
    add("server", &["server"]);
    add("cable-management", &["cable manager", "cable management"]);
    add("pdu", &["pdu", "power strip"]);
    tags.sort();
    tags.dedup();
    text.clear();
    tags
}

/// Pick the first non-empty shape-data value among aliases.
#[must_use]
pub fn shape_value<'a>(fields: &'a [ShapeField], labels: &[&str]) -> Option<&'a str> {
    fields.iter().find_map(|field| {
        labels
            .iter()
            .any(|label| field.label.trim().eq_ignore_ascii_case(label))
            .then_some(field.value.as_str())
            .filter(|value| !value.trim().is_empty())
    })
}

/// Parse the first run of digits in a string.
#[must_use]
pub fn first_u32(text: &str) -> Option<u32> {
    let digits: String = text
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// Default fill used when a Visio shape has no usable fill colour.
pub const DEFAULT_FILL: Color = Color::rgb(45, 52, 66);

/// Default stroke used when a Visio shape has no usable line colour.
pub const DEFAULT_STROKE: Color = Color::rgb(174, 185, 202);
