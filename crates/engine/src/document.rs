//! The persisted document model.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION;
use crate::connector::{Connector, ConnectorId};
use crate::error::{EngineError, Result};
use crate::geometry::{Color, Point, Rect, Size};

/// Stable identifier of a placed item inside one document.
pub type ItemId = u64;

/// Reference back to the stencil master an item was placed from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StencilRef {
    /// Stencil library display name (usually the `.vssx` file name).
    pub library: String,
    /// Master ID inside the Visio package.
    pub master_id: String,
    /// Master name, e.g. `24-Port CAT6 Patch Panel`.
    pub name: String,
}

/// A shape-data row read from the stencil (Visio `Property` / `ShapeData`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ShapeField {
    /// Human-readable label, e.g. `Manufacturer`.
    pub label: String,
    /// Current value.
    pub value: String,
    /// Optional Visio prompt / help text.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub prompt: String,
}

/// Editable project metadata attached to one placed item.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFields {
    /// Installation date (free-form ISO date recommended).
    #[serde(default)]
    pub installed_date: String,
    /// Warranty expiry (free-form ISO date recommended).
    #[serde(default)]
    pub warranty_exp: String,
    /// Room / row / position.
    #[serde(default)]
    pub location: String,
    /// Building or site.
    #[serde(default)]
    pub building: String,
    /// Free-form notes.
    #[serde(default)]
    pub notes: String,
}

/// Normalized mounting area inside a rack stencil.
///
/// Values are fractions of the rack item's width/height. They are optional so
/// arbitrary imported stencils still work; RackCraft infers sensible defaults
/// when a stencil is tagged as a rack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MountSpec {
    /// Number of rack units in the mounting area.
    pub units: u32,
    /// Left inset as a fraction of the rack width.
    #[serde(default)]
    pub left: f64,
    /// Right inset as a fraction of the rack width.
    #[serde(default)]
    pub right: f64,
    /// Top inset as a fraction of the rack height.
    #[serde(default)]
    pub top: f64,
    /// Bottom inset as a fraction of the rack height.
    #[serde(default)]
    pub bottom: f64,
}

impl MountSpec {
    /// Create a mounting specification with standard insets.
    #[must_use]
    pub fn new(units: u32) -> Self {
        Self {
            units,
            left: 0.10,
            right: 0.10,
            top: 0.03,
            bottom: 0.03,
        }
    }

    /// Mounting rectangle inside `rack_bounds`.
    #[must_use]
    pub fn interior(&self, rack_bounds: Rect) -> Rect {
        let left = self.left.clamp(0.0, 0.45);
        let right = self.right.clamp(0.0, 0.45);
        let top = self.top.clamp(0.0, 0.45);
        let bottom = self.bottom.clamp(0.0, 0.45);
        Rect::new(
            rack_bounds.x + rack_bounds.width * left,
            rack_bounds.y + rack_bounds.height * top,
            rack_bounds.width * (1.0 - left - right).max(0.1),
            rack_bounds.height * (1.0 - top - bottom).max(0.1),
        )
    }

    /// Height of one rack unit inside `rack_bounds`.
    #[must_use]
    pub fn unit_height(&self, rack_bounds: Rect) -> f64 {
        if self.units == 0 {
            0.0
        } else {
            self.interior(rack_bounds).height / f64::from(self.units)
        }
    }
}

/// One drawing command in normalized item space (`0..=1` on both axes).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum PathCommand {
    /// Start a new sub-path.
    MoveTo {
        /// End point.
        to: Point,
    },
    /// Straight segment.
    LineTo {
        /// End point.
        to: Point,
    },
    /// Quadratic Bézier.
    QuadTo {
        /// Control point.
        ctrl: Point,
        /// End point.
        to: Point,
    },
    /// Cubic Bézier.
    CubicTo {
        /// First control point.
        c1: Point,
        /// Second control point.
        c2: Point,
        /// End point.
        to: Point,
    },
    /// Close the current sub-path.
    Close,
}

/// A styled path stored in normalized coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawPath {
    /// Drawing commands.
    pub commands: Vec<PathCommand>,
    /// Fill colour (`None` = unfilled).
    #[serde(default)]
    pub fill: Option<Color>,
    /// Stroke colour (`None` = no stroke).
    #[serde(default)]
    pub stroke: Option<Color>,
    /// Stroke width in millimetres at the item's native size.
    #[serde(default = "default_stroke_width")]
    pub stroke_width_mm: f64,
}

const fn default_stroke_width() -> f64 {
    0.25
}

impl DrawPath {
    /// Create a path with a fill and no stroke.
    #[must_use]
    pub fn filled(commands: Vec<PathCommand>, fill: Color) -> Self {
        Self {
            commands,
            fill: Some(fill),
            stroke: None,
            stroke_width_mm: default_stroke_width(),
        }
    }

    /// Add a stroke.
    #[must_use]
    pub fn with_stroke(mut self, stroke: Color, width_mm: f64) -> Self {
        self.stroke = Some(stroke);
        self.stroke_width_mm = width_mm;
        self
    }

    /// Whether the path can be drawn at all.
    #[must_use]
    pub fn is_drawable(&self) -> bool {
        self.commands.iter().any(|c| {
            matches!(
                c,
                PathCommand::LineTo { .. }
                    | PathCommand::QuadTo { .. }
                    | PathCommand::CubicTo { .. }
            )
        })
    }
}

/// One connection point (Visio "glue" point), normalized to the item box.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionPoint {
    /// Stable id from the stencil (`Row_1`, `Port 1`, …).
    #[serde(default)]
    pub id: String,
    /// X coordinate, `0..=1`.
    pub x: f64,
    /// Y coordinate, `0..=1`.
    pub y: f64,
    /// Optional user-facing name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ConnectionPoint {
    /// Create a normalized connection point.
    #[must_use]
    pub fn new(id: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            x,
            y,
            name: None,
        }
    }

    /// Give the point a display name.
    #[must_use]
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// A placed equipment or stencil instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// Unique id within the document.
    pub id: ItemId,
    /// Source master.
    pub master: StencilRef,
    /// Display name shown on the canvas and in the BOM fallback.
    pub name: String,
    /// Stencil category metadata.
    #[serde(default)]
    pub category: String,
    /// Top-left position in millimetres.
    pub position: Point,
    /// Size in millimetres.
    pub size: Size,
    /// Self-contained normalized vector artwork.
    #[serde(default)]
    pub graphics: Vec<DrawPath>,
    /// Snap anchors, normalized.
    #[serde(default)]
    pub connection_points: Vec<ConnectionPoint>,
    /// Manufacturer shape data, in display order.
    #[serde(default)]
    pub shape_data: Vec<ShapeField>,
    /// Editable project fields.
    #[serde(default)]
    pub project: ProjectFields,
    /// Searchable/inference tags (`rack`, `patch-panel`, `keystone`, …).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Rack mounting information, when this item is a rack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mount: Option<MountSpec>,
    /// Painter's order (higher draws on top).
    #[serde(default)]
    pub z_index: i32,
    /// Locked items cannot be moved or selected for editing.
    #[serde(default)]
    pub locked: bool,
}

impl Item {
    /// Create an empty placed item. [`Document::add_item`] assigns the id.
    #[must_use]
    pub fn new(master: StencilRef, name: impl Into<String>, position: Point, size: Size) -> Self {
        Self {
            id: 0,
            master,
            name: name.into(),
            category: String::new(),
            position,
            size,
            graphics: Vec::new(),
            connection_points: Vec::new(),
            shape_data: Vec::new(),
            project: ProjectFields::default(),
            tags: Vec::new(),
            mount: None,
            z_index: 0,
            locked: false,
        }
    }

    /// Axis-aligned item box in millimetres.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.size.width,
            self.size.height,
        )
    }

    /// Whether a document-space point hits this item.
    #[must_use]
    pub fn hit_test(&self, point: Point) -> bool {
        self.bounds().contains(point)
    }

    /// Absolute connection points for snapping and debugging overlays.
    #[must_use]
    pub fn absolute_connections(&self) -> Vec<Point> {
        self.connection_points
            .iter()
            .map(|cp| {
                Point::new(
                    self.position.x + cp.x * self.size.width,
                    self.position.y + cp.y * self.size.height,
                )
            })
            .collect()
    }

    /// Centre anchor, used for stencils without connection points.
    #[must_use]
    pub fn center_anchor(&self) -> Point {
        self.bounds().center()
    }

    /// Case-insensitive shape-data lookup.
    #[must_use]
    pub fn shape_value(&self, label: &str) -> Option<&str> {
        self.shape_data
            .iter()
            .find(|field| field.label.trim().eq_ignore_ascii_case(label))
            .map(|field| field.value.as_str())
    }

    /// Case-insensitive lookup across several aliases, returning the first hit.
    #[must_use]
    pub fn shape_value_any(&self, labels: &[&str]) -> Option<&str> {
        labels.iter().find_map(|label| self.shape_value(label))
    }

    /// Insert or update a shape-data row.
    pub fn set_shape_value(&mut self, label: &str, value: impl Into<String>) {
        let value = value.into();
        if let Some(field) = self
            .shape_data
            .iter_mut()
            .find(|field| field.label.trim().eq_ignore_ascii_case(label))
        {
            field.value = value;
        } else {
            self.shape_data.push(ShapeField {
                label: label.to_owned(),
                value,
                prompt: String::new(),
            });
        }
    }

    /// Manufacturer, or a neutral fallback for the BOM.
    #[must_use]
    pub fn manufacturer(&self) -> &str {
        self.shape_value_any(&["Manufacturer", "Vendor", "Brand"])
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Unknown")
    }

    /// Part number, or an empty string when the stencil supplies none.
    #[must_use]
    pub fn part_number(&self) -> &str {
        self.shape_value_any(&[
            "Part Number",
            "PartNumber",
            "Part No.",
            "Part No",
            "Product Number",
            "SKU",
        ])
        .map(str::trim)
        .unwrap_or("")
    }

    /// Product description, falling back to the stencil name.
    #[must_use]
    pub fn description(&self) -> &str {
        self.shape_value_any(&[
            "Product Description",
            "Description",
            "Product",
            "Model",
            "Name",
        ])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&self.name)
    }

    /// Whether a tag is present (case-insensitive).
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }

    /// Whether this item is a rack-like container.
    #[must_use]
    pub fn is_rack(&self) -> bool {
        self.mount.is_some()
            || self.has_tag("rack")
            || self.name.to_ascii_lowercase().contains("rack")
    }

    /// Read the declared number of rack units from shape data or the name.
    #[must_use]
    pub fn rack_units(&self) -> Option<u32> {
        if let Some(value) = self.shape_value_any(&["Rack Units", "Units", "U Height", "RU"]) {
            if let Ok(units) = value.trim().parse::<u32>()
                && units > 0
            {
                return Some(units);
            }
            let digits: String = value.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(units) = digits.parse::<u32>()
                && units > 0
            {
                return Some(units);
            }
        }
        let lower = self.name.to_ascii_lowercase();
        for token in lower.split(|c: char| !c.is_ascii_alphanumeric()) {
            if let Some(prefix) = token.strip_suffix('u')
                && let Ok(units) = prefix.parse::<u32>()
                && (4..=60).contains(&units)
            {
                return Some(units);
            }
        }
        None
    }

    /// Mounting geometry for rack-like items, with a sensible fallback.
    #[must_use]
    pub fn effective_mount(&self) -> Option<MountSpec> {
        if !self.is_rack() {
            return None;
        }
        self.mount
            .clone()
            .or_else(|| Some(MountSpec::new(self.rack_units().unwrap_or(24))))
    }

    /// Location summary for the BOM.
    #[must_use]
    pub fn location_summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.project.building.trim().is_empty() {
            parts.push(self.project.building.trim());
        }
        if !self.project.location.trim().is_empty() {
            parts.push(self.project.location.trim());
        }
        parts.join(" — ")
    }
}

/// Canvas / paper description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Canvas {
    /// Width in millimetres.
    pub width_mm: f64,
    /// Height in millimetres.
    pub height_mm: f64,
    /// Grid spacing in millimetres (the UI hides the grid below a zoom level).
    pub grid_mm: f64,
    /// Draw the engineering grid.
    pub show_grid: bool,
    /// Paper/canvas background.
    #[serde(default = "default_canvas_color")]
    pub background: Color,
}

const fn default_canvas_color() -> Color {
    Color::rgb(21, 24, 30)
}

/// Extra room kept around content when the canvas grows automatically.
pub const CANVAS_GROW_MARGIN_MM: f64 = 80.0;

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width_mm: 760.0,
            height_mm: 1420.0,
            grid_mm: 10.0,
            show_grid: true,
            background: default_canvas_color(),
        }
    }
}

impl Canvas {
    /// Grow the page so `bounds` fits with `margin` millimetres to spare.
    ///
    /// The canvas only grows; it never shrinks, so saved documents keep their
    /// size and undo does not fight the user.
    pub fn grow_to_include(&mut self, bounds: Rect, margin: f64) {
        if !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
        {
            return;
        }
        let margin = margin.max(0.0);
        self.width_mm = self.width_mm.max(bounds.right() + margin).max(1.0);
        self.height_mm = self.height_mm.max(bounds.bottom() + margin).max(1.0);
    }
}

/// Root persisted document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Schema version for forward-compatible loading.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Project name.
    pub name: String,
    /// Canvas description.
    pub canvas: Canvas,
    /// Placed items, in insertion order (painter sorts by `z_index`).
    #[serde(default)]
    pub items: Vec<Item>,
    /// Cables / network connectors.
    #[serde(default)]
    pub connectors: Vec<Connector>,
    /// Next item id to assign.
    #[serde(default = "default_next_id")]
    pub next_id: ItemId,
    /// Next connector id to assign.
    #[serde(default = "default_next_id")]
    pub next_connector_id: ConnectorId,
    /// Extensible project metadata.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

const fn default_schema_version() -> u32 {
    SCHEMA_VERSION
}

const fn default_next_id() -> ItemId {
    1
}

impl Default for Document {
    fn default() -> Self {
        Self::new("Untitled rack")
    }
}

impl Document {
    /// Create an empty document.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            name: name.into(),
            canvas: Canvas::default(),
            items: Vec::new(),
            connectors: Vec::new(),
            next_id: 1,
            next_connector_id: 1,
            metadata: BTreeMap::new(),
        }
    }

    /// Axis-aligned box that contains every placed item, in millimetres.
    #[must_use]
    pub fn content_bounds(&self) -> Option<Rect> {
        self.items.iter().map(Item::bounds).reduce(Rect::union)
    }

    /// Add an item and assign it a fresh id.
    ///
    /// The canvas grows automatically when the item extends past the page, so
    /// a 42U rack or another large stencil stays usable without a fixed page
    /// size.
    pub fn add_item(&mut self, mut item: Item) -> ItemId {
        let id = self.next_id.max(1);
        self.next_id = id + 1;
        item.id = id;
        if item.z_index == 0 {
            item.z_index = self.items.len() as i32;
        }
        self.canvas
            .grow_to_include(item.bounds(), CANVAS_GROW_MARGIN_MM);
        self.items.push(item);
        id
    }

    /// Borrow an item by id.
    #[must_use]
    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Mutably borrow an item by id.
    pub fn item_mut(&mut self, id: ItemId) -> Option<&mut Item> {
        self.items.iter_mut().find(|item| item.id == id)
    }

    /// Remove an item and return it.
    ///
    /// Connectors attached to this item become free anchors at their last
    /// resolved position, so the diagram never keeps a dangling item id.
    pub fn remove_item(&mut self, id: ItemId) -> Option<Item> {
        let index = self.items.iter().position(|item| item.id == id)?;
        self.detach_item(id);
        Some(self.items.remove(index))
    }

    /// Duplicate an item with a small offset; returns the new id.
    pub fn duplicate_item(&mut self, id: ItemId, offset_mm: f64) -> Option<ItemId> {
        let mut copy = self.item(id)?.clone();
        copy.position = copy.position.translated(offset_mm, offset_mm);
        copy.locked = false;
        Some(self.add_item(copy))
    }

    /// Topmost item under a point (locked items are ignored when unlocked_only).
    #[must_use]
    pub fn hit_test(&self, point: Point, unlocked_only: bool) -> Option<&Item> {
        self.items
            .iter()
            .filter(|item| !unlocked_only || !item.locked)
            .filter(|item| item.hit_test(point))
            .max_by_key(|item| item.z_index)
    }

    /// Bounding rectangle of every item, if any.
    #[must_use]
    pub fn items_bounds(&self) -> Option<Rect> {
        let mut iter = self.items.iter();
        let first = iter.next()?.bounds();
        Some(iter.fold(first, |acc, item| {
            let b = item.bounds();
            let left = acc.left().min(b.left());
            let top = acc.top().min(b.top());
            let right = acc.right().max(b.right());
            let bottom = acc.bottom().max(b.bottom());
            Rect::new(left, top, right - left, bottom - top)
        }))
    }

    /// Serialize as pretty JSON.
    pub fn to_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Load a document from JSON and normalize ids.
    pub fn from_json(json: &str) -> Result<Self> {
        let mut document: Self = serde_json::from_str(json)?;
        if document.schema_version == 0 {
            document.schema_version = 1;
        }
        if document.schema_version > SCHEMA_VERSION {
            return Err(EngineError::UnsupportedSchema {
                found: document.schema_version,
                supported: SCHEMA_VERSION,
            });
        }
        let max_id = document.items.iter().map(|item| item.id).max().unwrap_or(0);
        document.next_id = document.next_id.max(max_id + 1).max(1);
        let max_connector = document
            .connectors
            .iter()
            .map(|connector| connector.id)
            .max()
            .unwrap_or(0);
        document.next_connector_id = document.next_connector_id.max(max_connector + 1).max(1);
        document.schema_version = SCHEMA_VERSION;
        // Repair documents saved before the canvas grew with their content.
        if let Some(bounds) = document.content_bounds() {
            document
                .canvas
                .grow_to_include(bounds, CANVAS_GROW_MARGIN_MM);
        }
        Ok(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item() -> Item {
        let mut item = Item::new(
            StencilRef {
                library: "Test".into(),
                master_id: "1".into(),
                name: "Patch panel".into(),
            },
            "24-port patch panel",
            Point::new(10.0, 20.0),
            Size::new(482.6, 44.45),
        );
        item.set_shape_value("Manufacturer", "RackCraft");
        item.set_shape_value("Part Number", "RC-PP-24");
        item.connection_points = vec![ConnectionPoint::new("Port 1", 0.25, 0.5)];
        item
    }

    #[test]
    fn add_grows_the_canvas_for_a_42u_rack() {
        let mut doc = Document::new("Test");
        let rack = Item::new(
            StencilRef {
                library: "Test".into(),
                master_id: "rack".into(),
                name: "42U rack".into(),
            },
            "42U rack",
            Point::new(0.0, 0.0),
            Size::new(600.0, 42.0 * 44.45),
        );
        doc.add_item(rack);
        assert!(
            doc.canvas.height_mm >= 42.0 * 44.45 + CANVAS_GROW_MARGIN_MM,
            "canvas height {} should include the 42U rack",
            doc.canvas.height_mm
        );
        assert_eq!(
            doc.content_bounds(),
            Some(Rect::new(0.0, 0.0, 600.0, 42.0 * 44.45))
        );
    }

    #[test]
    fn canvas_does_not_grow_for_a_small_item() {
        let mut doc = Document::new("Test");
        let before = doc.canvas.clone();
        doc.add_item(sample_item());
        assert_eq!(doc.canvas, before);
    }

    #[test]
    fn loading_an_old_document_repairs_the_canvas() {
        let mut doc = Document::new("Test");
        doc.add_item(Item::new(
            StencilRef {
                library: "Test".into(),
                master_id: "rack".into(),
                name: "42U rack".into(),
            },
            "42U rack",
            Point::new(0.0, 0.0),
            Size::new(600.0, 42.0 * 44.45),
        ));
        doc.canvas.width_mm = 760.0;
        doc.canvas.height_mm = 1420.0;
        let json = doc.to_json_pretty().expect("serialize");
        let loaded = Document::from_json(&json).expect("load");
        assert!(loaded.canvas.height_mm >= 42.0 * 44.45);
    }

    #[test]
    fn add_assigns_increasing_ids() {
        let mut doc = Document::new("Test");
        let a = doc.add_item(sample_item());
        let b = doc.add_item(sample_item());
        assert_eq!((a, b), (1, 2));
        assert_eq!(doc.items.len(), 2);
        assert_eq!(doc.item(a).unwrap().name, "24-port patch panel");
    }

    #[test]
    fn remove_and_duplicate() {
        let mut doc = Document::new("Test");
        let id = doc.add_item(sample_item());
        let copy = doc.duplicate_item(id, 5.0).unwrap();
        assert_eq!(doc.item(copy).unwrap().position, Point::new(15.0, 25.0));
        let removed = doc.remove_item(id).unwrap();
        assert_eq!(removed.id, id);
        assert!(doc.item(id).is_none());
    }

    #[test]
    fn json_round_trip_normalizes_next_id() {
        let mut doc = Document::new("JSON");
        doc.add_item(sample_item());
        let json = doc.to_json_pretty().unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        value["next_id"] = serde_json::json!(1);
        let patched = serde_json::to_string(&value).unwrap();
        let loaded = Document::from_json(&patched).unwrap();
        assert_eq!(loaded.next_id, 2);
        assert_eq!(loaded.items, doc.items);
        assert_eq!(loaded.name, "JSON");
    }

    #[test]
    fn schema_one_documents_load_without_connectors() {
        let doc = Document::new("Old");
        let mut value = serde_json::to_value(doc).unwrap();
        value["schema_version"] = serde_json::json!(1);
        value.as_object_mut().unwrap().remove("connectors");
        value.as_object_mut().unwrap().remove("next_connector_id");
        let loaded = Document::from_json(&value.to_string()).unwrap();
        assert!(loaded.connectors.is_empty());
        assert_eq!(loaded.next_connector_id, 1);
        assert_eq!(loaded.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn future_schema_is_rejected() {
        let doc = Document::new("Future");
        let mut value = serde_json::to_value(doc).unwrap();
        value["schema_version"] = serde_json::json!(999);
        let err = Document::from_json(&value.to_string()).unwrap_err();
        assert!(matches!(err, EngineError::UnsupportedSchema { .. }));
    }

    #[test]
    fn shape_data_aliases_fall_back() {
        let mut item = sample_item();
        item.set_shape_value("Product Description", "CAT6 panel");
        assert_eq!(item.manufacturer(), "RackCraft");
        assert_eq!(item.part_number(), "RC-PP-24");
        assert_eq!(item.description(), "CAT6 panel");
        assert_eq!(
            item.absolute_connections(),
            vec![Point::new(130.65, 42.225)]
        );
    }

    #[test]
    fn mount_geometry_is_correct() {
        let rack = Rect::new(0.0, 0.0, 600.0, 1181.25);
        let mount = MountSpec {
            units: 25,
            left: 58.7 / 600.0,
            right: 58.7 / 600.0,
            top: 35.0 / 1181.25,
            bottom: 35.0 / 1181.25,
        };
        let interior = mount.interior(rack);
        assert!((interior.width - 482.6).abs() < 1e-9);
        assert!((interior.height - 1111.25).abs() < 1e-9);
        assert!((mount.unit_height(rack) - 44.45).abs() < 1e-9);
    }
}
