//! UI-free document engine for RackCraft.
//!
//! The engine owns the document model, millimetre geometry, rack-unit math,
//! magnetic snapping, undo/redo and the bill-of-materials computation. It has
//! no filesystem, UI or Visio-parsing dependencies, which keeps it fast to test
//! and usable from a future CLI.

pub mod bom;
pub mod connector;
pub mod document;
pub mod error;
pub mod geometry;
pub mod history;
pub mod sample;
pub mod snap;
pub mod units;

pub use bom::{Bom, BomLine};
pub use connector::{Arrow, Connector, ConnectorEnd, ConnectorId, LineStyle, Routing};
pub use document::{
    Canvas, ConnectionPoint, Document, DrawPath, Item, ItemId, MountSpec, PathCommand,
    ProjectFields, ShapeField, StencilRef,
};
pub use error::EngineError;
pub use geometry::{Color, Point, Rect, Size};
pub use history::History;
pub use snap::{SnapKind, SnapResult, SnapSettings, snap_item};
pub use units::{MM_PER_INCH, RACK_U_MM, RACK_WIDTH_19_IN_MM, mm_to_px, px_to_mm};

/// Current serialized document schema version.
pub const SCHEMA_VERSION: u32 = 2;
