//! Magnetic snapping: rack-unit alignment and connection-point glue.

use serde::{Deserialize, Serialize};

use crate::document::{Document, Item, MountSpec};
use crate::geometry::Point;

/// What produced a snap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapKind {
    /// The item aligns to a rack-unit boundary.
    RackUnit,
    /// Two connection points are glued together.
    ConnectionPoint,
    /// The free grid handled the movement.
    Grid,
}

/// Tunable snap behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SnapSettings {
    /// Maximum glue distance in millimetres (convert from screen pixels in the UI).
    pub threshold_mm: f64,
    /// Enable rack-U alignment.
    pub rack_units: bool,
    /// Enable connection-point glue.
    pub connection_points: bool,
    /// Optional free grid spacing; `None` disables grid fallback.
    pub grid_mm: Option<f64>,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            threshold_mm: 8.0,
            rack_units: true,
            connection_points: true,
            grid_mm: None,
        }
    }
}

/// Result of a snap query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapResult {
    /// Adjusted top-left position for the moving item.
    pub position: Point,
    /// Point where the green snap indicator should be painted.
    pub indicator: Option<Point>,
    /// Which rule matched.
    pub kind: Option<SnapKind>,
}

impl SnapResult {
    /// No snap: the desired position is returned untouched.
    #[must_use]
    pub fn none(position: Point) -> Self {
        Self {
            position,
            indicator: None,
            kind: None,
        }
    }

    /// Whether a snap occurred.
    #[must_use]
    pub fn is_snapped(&self) -> bool {
        self.kind.is_some()
    }
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    distance: f64,
    position: Point,
    indicator: Option<Point>,
    kind: SnapKind,
}

fn consider(best: &mut Option<Candidate>, candidate: Candidate) {
    if best.is_none_or(|current| candidate.distance < current.distance) {
        *best = Some(candidate);
    }
}

/// Snap a moving item to rack units and connection points in `document`.
///
/// `moving` is a probe with its intended size, connection points and tags; its
/// current `position` is ignored. The returned `position` should be assigned to
/// the real item.
#[must_use]
pub fn snap_item(
    document: &Document,
    moving: &Item,
    desired: Point,
    settings: &SnapSettings,
) -> SnapResult {
    let mut probe = moving.clone();
    probe.position = desired;

    if probe.locked {
        return SnapResult::none(desired);
    }

    let mut best: Option<Candidate> = None;

    if settings.rack_units && !probe.is_rack() {
        for rack in &document.items {
            if rack.id == probe.id {
                continue;
            }
            let Some(mount) = rack.effective_mount() else {
                continue;
            };
            if let Some(candidate) =
                rack_candidate(rack, &probe, &mount, desired, settings.threshold_mm)
            {
                consider(&mut best, candidate);
            }
        }
    }

    if settings.connection_points {
        let moving_anchors = anchors(&probe);
        for target in &document.items {
            if target.id == probe.id {
                continue;
            }
            // Racks use the unit-grid rule above. Their decorative connection
            // points must not pull equipment to the middle of a rack unit.
            if settings.rack_units && target.is_rack() {
                continue;
            }
            for target_anchor in anchors(target) {
                for moving_anchor in &moving_anchors {
                    let distance = moving_anchor.distance(target_anchor);
                    if distance <= settings.threshold_mm {
                        let position = desired.translated(
                            target_anchor.x - moving_anchor.x,
                            target_anchor.y - moving_anchor.y,
                        );
                        consider(
                            &mut best,
                            Candidate {
                                distance,
                                position,
                                indicator: Some(target_anchor),
                                kind: SnapKind::ConnectionPoint,
                            },
                        );
                    }
                }
            }
        }
    }

    if let Some(best) = best {
        return SnapResult {
            position: best.position,
            indicator: best.indicator,
            kind: Some(best.kind),
        };
    }

    if let Some(grid) = settings.grid_mm.filter(|grid| *grid > 0.0) {
        let position = Point::new(
            (desired.x / grid).round() * grid,
            (desired.y / grid).round() * grid,
        );
        return SnapResult {
            position,
            indicator: None,
            kind: Some(SnapKind::Grid),
        };
    }

    SnapResult::none(desired)
}

fn anchors(item: &Item) -> Vec<Point> {
    if item.connection_points.is_empty() {
        vec![item.center_anchor()]
    } else {
        item.absolute_connections()
    }
}

fn rack_candidate(
    rack: &Item,
    moving: &Item,
    mount: &MountSpec,
    desired: Point,
    threshold: f64,
) -> Option<Candidate> {
    if mount.units == 0 {
        return None;
    }
    let interior = mount.interior(rack.bounds());
    let unit_height = mount.unit_height(rack.bounds());
    if unit_height <= f64::EPSILON {
        return None;
    }

    // Ignore racks the pointer is nowhere near horizontally.
    let horizontal_margin = threshold * 4.0;
    if desired.x + moving.size.width < interior.left() - horizontal_margin
        || desired.x > interior.right() + horizontal_margin
    {
        return None;
    }

    let centered_x = interior.center().x - moving.size.width * 0.5;
    let max_top = (interior.bottom() - moving.size.height).max(interior.top());

    let mut best_top = interior.top();
    let mut best_distance = f64::INFINITY;

    for index in 0..=mount.units {
        let line = interior.top() + unit_height * f64::from(index);
        for candidate_y in [line, line - moving.size.height] {
            let candidate_y = candidate_y.clamp(interior.top(), max_top);
            let distance = (candidate_y - desired.y).abs();
            if distance < best_distance {
                best_distance = distance;
                best_top = candidate_y;
            }
        }
    }

    if best_distance > threshold {
        return None;
    }

    let position = Point::new(centered_x, best_top);
    // Horizontal offset is part of the magnetic feel, but rack alignment is
    // vertical-first: a panel dropped inside the rack is centred automatically.
    let dx = (desired.x - centered_x).abs();
    let distance = best_distance.hypot(dx * 0.15);
    Some(Candidate {
        distance,
        position,
        indicator: Some(Point::new(interior.center().x, best_top)),
        kind: SnapKind::RackUnit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{ConnectionPoint, Item, MountSpec, StencilRef};
    use crate::geometry::Size;

    fn rack() -> Item {
        let mut rack = Item::new(
            StencilRef::default(),
            "25U Rack",
            Point::new(0.0, 0.0),
            Size::new(600.0, 1181.25),
        );
        rack.tags = vec!["rack".into()];
        rack.mount = Some(MountSpec {
            units: 25,
            left: 58.7 / 600.0,
            right: 58.7 / 600.0,
            top: 35.0 / 1181.25,
            bottom: 35.0 / 1181.25,
        });
        rack
    }

    fn one_u(name: &str) -> Item {
        Item::new(
            StencilRef::default(),
            name,
            Point::new(0.0, 0.0),
            Size::new(482.6, 44.45),
        )
    }

    #[test]
    fn one_u_panel_snaps_to_rack_and_is_centred() {
        let mut doc = Document::new("Rack");
        doc.add_item(rack());
        let panel = one_u("Patch panel");
        let desired = Point::new(70.0, 35.0 + 2.0 * 44.45 + 5.0);
        let result = snap_item(&doc, &panel, desired, &SnapSettings::default());
        assert_eq!(result.kind, Some(SnapKind::RackUnit));
        assert!((result.position.x - 58.7).abs() < 1e-9);
        assert!((result.position.y - (35.0 + 2.0 * 44.45)).abs() < 1e-9);
        assert_eq!(result.indicator, Some(Point::new(300.0, result.position.y)));
    }

    #[test]
    fn rack_units_align_two_u_slots() {
        let mut doc = Document::new("Rack");
        doc.add_item(rack());
        let mut two_u = one_u("Double panel");
        two_u.size.height = 2.0 * 44.45;
        let desired = Point::new(58.7, 35.0 + 2.0 * 44.45 - 3.0);
        let result = snap_item(&doc, &two_u, desired, &SnapSettings::default());
        assert_eq!(result.kind, Some(SnapKind::RackUnit));
        assert!((result.position.y - (35.0 + 2.0 * 44.45)).abs() < 1e-9);
    }

    #[test]
    fn connection_points_glue_keystone_to_panel() {
        let mut doc = Document::new("Panel");
        let mut panel = Item::new(
            StencilRef::default(),
            "Modular panel",
            Point::new(0.0, 0.0),
            Size::new(100.0, 100.0),
        );
        panel.connection_points = vec![ConnectionPoint::new("slot-1", 0.5, 0.9)];
        doc.add_item(panel);

        let mut keystone = Item::new(
            StencilRef::default(),
            "Keystone",
            Point::new(0.0, 0.0),
            Size::new(10.0, 10.0),
        );
        keystone.connection_points = vec![ConnectionPoint::new("rj45", 0.5, 0.5)];
        let desired = Point::new(45.0, 88.0);
        let result = snap_item(&doc, &keystone, desired, &SnapSettings::default());
        assert_eq!(result.kind, Some(SnapKind::ConnectionPoint));
        assert_eq!(result.position, Point::new(45.0, 85.0));
        assert_eq!(result.indicator, Some(Point::new(50.0, 90.0)));
    }

    #[test]
    fn far_away_item_does_not_snap() {
        let mut doc = Document::new("Rack");
        doc.add_item(rack());
        let panel = one_u("Patch panel");
        let desired = Point::new(58.7, 500.0);
        let result = snap_item(&doc, &panel, desired, &SnapSettings::default());
        assert!(!result.is_snapped());
        assert_eq!(result.position, desired);
    }

    #[test]
    fn disconnected_threshold_is_respected() {
        let mut doc = Document::new("Panel");
        let mut panel = Item::new(
            StencilRef::default(),
            "Modular panel",
            Point::new(0.0, 0.0),
            Size::new(100.0, 100.0),
        );
        panel.connection_points = vec![ConnectionPoint::new("slot", 0.5, 0.5)];
        doc.add_item(panel);
        let keystone = Item::new(
            StencilRef::default(),
            "Keystone",
            Point::new(0.0, 0.0),
            Size::new(10.0, 10.0),
        );
        let settings = SnapSettings {
            threshold_mm: 2.0,
            ..SnapSettings::default()
        };
        let result = snap_item(&doc, &keystone, Point::new(60.0, 45.0), &settings);
        assert!(!result.is_snapped());
    }

    #[test]
    fn grid_fallback_only_when_nothing_else_matches() {
        let doc = Document::new("Empty");
        let item = one_u("Panel");
        let settings = SnapSettings {
            grid_mm: Some(10.0),
            ..SnapSettings::default()
        };
        let result = snap_item(&doc, &item, Point::new(12.0, 28.0), &settings);
        assert_eq!(result.kind, Some(SnapKind::Grid));
        assert_eq!(result.position, Point::new(10.0, 30.0));
    }

    #[test]
    fn rack_units_infer_from_name() {
        let rack = rack();
        assert_eq!(rack.rack_units(), Some(25));
        assert_eq!(rack.effective_mount().unwrap().units, 25);
    }
}
