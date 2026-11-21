//! Magnetic cables / network connectors between equipment ports or free points.

use serde::{Deserialize, Serialize};

use crate::document::{Document, ItemId};
use crate::geometry::{Color, Point};

/// Stable identifier of a connector inside one document.
pub type ConnectorId = u64;

/// One end of a connector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConnectorEnd {
    /// Glue to a connection point on a placed item.
    ConnectionPoint {
        /// Target item.
        item_id: ItemId,
        /// Index in [`crate::Item::connection_points`].
        connection_point: usize,
    },
    /// A free document-space anchor in millimetres.
    Free {
        /// Anchor position.
        point: Point,
    },
}

impl ConnectorEnd {
    /// Whether this end is currently attached to a document item.
    #[must_use]
    pub fn is_attached(&self) -> bool {
        matches!(self, Self::ConnectionPoint { .. })
    }

    /// The target item id, if attached.
    #[must_use]
    pub fn item_id(&self) -> Option<ItemId> {
        match self {
            Self::ConnectionPoint { item_id, .. } => Some(*item_id),
            Self::Free { .. } => None,
        }
    }
}

/// How the connector is routed on the canvas.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Routing {
    /// One straight segment.
    #[default]
    Straight,
    /// Right-angled segments.
    Orthogonal,
}

/// Connector stroke style.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineStyle {
    /// Continuous line.
    #[default]
    Solid,
    /// Dashed line.
    Dashed,
}

/// Arrowhead decoration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arrow {
    /// No arrowhead.
    #[default]
    None,
    /// Arrow at the destination.
    End,
    /// Arrowheads at both ends.
    Double,
}

/// A cable/link between two document points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connector {
    /// Unique id in the document.
    pub id: ConnectorId,
    /// Start end.
    pub from: ConnectorEnd,
    /// Destination end.
    pub to: ConnectorEnd,
    /// User label (`10G SFP+`, `Cat6A`, `WAN`…).
    #[serde(default)]
    pub label: String,
    /// Stroke colour.
    #[serde(default = "default_connector_color")]
    pub color: Color,
    /// Straight or orthogonal routing.
    #[serde(default)]
    pub routing: Routing,
    /// Solid or dashed stroke.
    #[serde(default)]
    pub line_style: LineStyle,
    /// Arrowheads.
    #[serde(default)]
    pub arrow: Arrow,
}

const fn default_connector_color() -> Color {
    Color::rgb(84, 209, 132)
}

impl Connector {
    /// Create an unregistered connector; [`Document::add_connector`] assigns its id.
    #[must_use]
    pub fn new(from: ConnectorEnd, to: ConnectorEnd) -> Self {
        Self {
            id: 0,
            from,
            to,
            label: String::new(),
            color: default_connector_color(),
            routing: Routing::Straight,
            line_style: LineStyle::Solid,
            arrow: Arrow::None,
        }
    }

    /// Convenience builder for a label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Convenience builder for routing.
    #[must_use]
    pub fn with_routing(mut self, routing: Routing) -> Self {
        self.routing = routing;
        self
    }
}

impl Document {
    /// Add a connector and assign it a fresh id.
    pub fn add_connector(&mut self, mut connector: Connector) -> ConnectorId {
        let id = self.next_connector_id.max(1);
        self.next_connector_id = id + 1;
        connector.id = id;
        self.connectors.push(connector);
        id
    }

    /// Borrow a connector by id.
    #[must_use]
    pub fn connector(&self, id: ConnectorId) -> Option<&Connector> {
        self.connectors.iter().find(|connector| connector.id == id)
    }

    /// Mutably borrow a connector by id.
    pub fn connector_mut(&mut self, id: ConnectorId) -> Option<&mut Connector> {
        self.connectors
            .iter_mut()
            .find(|connector| connector.id == id)
    }

    /// Remove a connector.
    pub fn remove_connector(&mut self, id: ConnectorId) -> Option<Connector> {
        let index = self
            .connectors
            .iter()
            .position(|connector| connector.id == id)?;
        Some(self.connectors.remove(index))
    }

    /// Resolve an endpoint against the current item positions.
    #[must_use]
    pub fn resolve_connector_end(&self, end: &ConnectorEnd) -> Option<Point> {
        match end {
            ConnectorEnd::Free { point } => Some(*point),
            ConnectorEnd::ConnectionPoint {
                item_id,
                connection_point,
            } => self.item(*item_id).map(|item| {
                item.connection_points.get(*connection_point).map_or_else(
                    || item.bounds().center(),
                    |connection| {
                        Point::new(
                            item.position.x + connection.x * item.size.width,
                            item.position.y + connection.y * item.size.height,
                        )
                    },
                )
            }),
        }
    }

    /// Resolve both connector ends.
    #[must_use]
    pub fn resolve_connector(&self, connector: &Connector) -> Option<(Point, Point)> {
        Some((
            self.resolve_connector_end(&connector.from)?,
            self.resolve_connector_end(&connector.to)?,
        ))
    }

    /// Full routed polyline for a connector (two points for straight routing,
    /// up to four for orthogonal routing).
    #[must_use]
    pub fn connector_path(&self, connector: &Connector) -> Option<Vec<Point>> {
        let (start, end) = self.resolve_connector(connector)?;
        Some(match connector.routing {
            Routing::Straight => vec![start, end],
            Routing::Orthogonal => {
                let dx = (end.x - start.x).abs();
                let dy = (end.y - start.y).abs();
                if dx < 0.01 || dy < 0.01 {
                    vec![start, end]
                } else if dx >= dy {
                    let mid_x = (start.x + end.x) * 0.5;
                    vec![
                        start,
                        Point::new(mid_x, start.y),
                        Point::new(mid_x, end.y),
                        end,
                    ]
                } else {
                    let mid_y = (start.y + end.y) * 0.5;
                    vec![
                        start,
                        Point::new(start.x, mid_y),
                        Point::new(end.x, mid_y),
                        end,
                    ]
                }
            }
        })
    }

    /// Find the topmost connector under a point, within `tolerance_mm`.
    #[must_use]
    pub fn hit_test_connector(&self, point: Point, tolerance_mm: f64) -> Option<ConnectorId> {
        self.connectors
            .iter()
            .filter_map(|connector| {
                let path = self.connector_path(connector)?;
                let distance = path
                    .windows(2)
                    .map(|segment| distance_to_segment(point, segment[0], segment[1]))
                    .fold(f64::INFINITY, f64::min);
                (distance <= tolerance_mm).then_some(connector.id)
            })
            .max()
    }

    /// Convert any connector end attached to `item_id` into a free anchor.
    ///
    /// Called before an item is removed so cables keep their last visual
    /// position instead of disappearing or dangling on a missing id.
    pub(crate) fn detach_item(&mut self, item_id: ItemId) {
        let Some(item) = self.item(item_id) else {
            return;
        };
        let center = item.bounds().center();
        let connections = item.absolute_connections();
        for connector in &mut self.connectors {
            detach_end(&mut connector.from, item_id, &connections, center);
            detach_end(&mut connector.to, item_id, &connections, center);
        }
    }
}

fn detach_end(end: &mut ConnectorEnd, item_id: ItemId, connections: &[Point], center: Point) {
    if let ConnectorEnd::ConnectionPoint {
        item_id: attached,
        connection_point,
    } = *end
        && attached == item_id
    {
        let point = connections.get(connection_point).copied().unwrap_or(center);
        *end = ConnectorEnd::Free { point };
    }
}

fn distance_to_segment(point: Point, a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    if dx.abs() < f64::EPSILON && dy.abs() < f64::EPSILON {
        return point.distance(a);
    }
    let t = ((point.x - a.x) * dx + (point.y - a.y) * dy) / (dx * dx + dy * dy);
    let t = t.clamp(0.0, 1.0);
    point.distance(Point::new(a.x + t * dx, a.y + t * dy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{ConnectionPoint, Item, StencilRef};
    use crate::geometry::Size;

    fn item(name: &str, x: f64) -> Item {
        let mut item = Item::new(
            StencilRef::default(),
            name,
            Point::new(x, 10.0),
            Size::new(100.0, 50.0),
        );
        item.connection_points = vec![
            ConnectionPoint::new("left", 0.0, 0.5),
            ConnectionPoint::new("right", 1.0, 0.5),
        ];
        item
    }

    fn attached(item_id: ItemId, point: usize) -> ConnectorEnd {
        ConnectorEnd::ConnectionPoint {
            item_id,
            connection_point: point,
        }
    }

    #[test]
    fn endpoints_follow_moved_items() {
        let mut doc = Document::new("Network");
        let a = doc.add_item(item("A", 10.0));
        let b = doc.add_item(item("B", 200.0));
        let cable = doc.add_connector(Connector::new(attached(a, 1), attached(b, 0)));
        assert_eq!(
            doc.resolve_connector(doc.connector(cable).unwrap()),
            Some((Point::new(110.0, 35.0), Point::new(200.0, 35.0)))
        );

        doc.item_mut(a).unwrap().position = Point::new(20.0, 60.0);
        assert_eq!(
            doc.resolve_connector(doc.connector(cable).unwrap()),
            Some((Point::new(120.0, 85.0), Point::new(200.0, 35.0)))
        );
    }

    #[test]
    fn removing_an_item_detaches_its_ends_at_the_last_anchor() {
        let mut doc = Document::new("Network");
        let a = doc.add_item(item("A", 10.0));
        let b = doc.add_item(item("B", 200.0));
        let cable = doc.add_connector(Connector::new(attached(a, 1), attached(b, 0)));
        let old_anchor = Point::new(110.0, 35.0);
        doc.remove_item(a);
        let connector = doc.connector(cable).unwrap();
        assert!(!connector.from.is_attached());
        assert_eq!(connector.from, ConnectorEnd::Free { point: old_anchor });
        assert_eq!(
            doc.resolve_connector(connector),
            Some((old_anchor, Point::new(200.0, 35.0)))
        );
    }

    #[test]
    fn orthogonal_routing_uses_right_angles() {
        let mut doc = Document::new("Network");
        let a = doc.add_item(item("A", 10.0));
        let b = doc.add_item(item("B", 200.0));
        doc.item_mut(b).unwrap().position.y = 100.0;
        let mut cable = Connector::new(attached(a, 1), attached(b, 0));
        cable.routing = Routing::Orthogonal;
        let cable = doc.add_connector(cable);
        let path = doc.connector_path(doc.connector(cable).unwrap()).unwrap();
        assert_eq!(path.len(), 4);
        assert!(path[1].x == path[2].x);
        assert!(path[0].y == path[1].y);
        assert!(path[2].y == path[3].y);
    }

    #[test]
    fn hit_test_finds_a_nearby_cable() {
        let mut doc = Document::new("Network");
        let a = doc.add_item(item("A", 10.0));
        let b = doc.add_item(item("B", 200.0));
        let cable = doc.add_connector(Connector::new(attached(a, 1), attached(b, 0)));
        assert_eq!(
            doc.hit_test_connector(Point::new(155.0, 36.0), 3.0),
            Some(cable)
        );
        assert_eq!(doc.hit_test_connector(Point::new(155.0, 60.0), 3.0), None);
    }

    #[test]
    fn free_to_free_connectors_resolve() {
        let mut doc = Document::new("Free");
        let connector = doc.add_connector(Connector::new(
            ConnectorEnd::Free {
                point: Point::new(1.0, 2.0),
            },
            ConnectorEnd::Free {
                point: Point::new(30.0, 40.0),
            },
        ));
        assert_eq!(
            doc.resolve_connector(doc.connector(connector).unwrap()),
            Some((Point::new(1.0, 2.0), Point::new(30.0, 40.0)))
        );
    }

    #[test]
    fn deleting_an_item_detaches_connectors_and_undo_restores_them() {
        use crate::history::History;

        let mut document = Document::new("Undo");
        let a = document.add_item(item("A", 10.0));
        let b = document.add_item(item("B", 200.0));
        let cable = document.add_connector(Connector::new(attached(a, 1), attached(b, 0)));
        let mut history = History::new(document);
        history.apply(|doc| {
            doc.remove_item(a);
        });
        assert!(
            !history
                .document()
                .connector(cable)
                .unwrap()
                .from
                .is_attached()
        );
        history.undo();
        assert!(
            history
                .document()
                .connector(cable)
                .unwrap()
                .from
                .is_attached()
        );
    }

    #[test]
    fn connectors_round_trip_through_json() {
        let mut doc = Document::new("JSON");
        let a = doc.add_item(item("A", 10.0));
        let b = doc.add_item(item("B", 200.0));
        let mut connector = Connector::new(attached(a, 1), attached(b, 0))
            .with_label("10G SFP+")
            .with_routing(Routing::Orthogonal);
        connector.line_style = LineStyle::Dashed;
        connector.arrow = Arrow::Double;
        connector.color = Color::rgb(90, 167, 232);
        let id = doc.add_connector(connector);
        let json = doc.to_json_pretty().unwrap();
        let loaded = Document::from_json(&json).unwrap();
        let connector = loaded.connector(id).unwrap();
        assert_eq!(connector.label, "10G SFP+");
        assert_eq!(connector.routing, Routing::Orthogonal);
        assert_eq!(connector.line_style, LineStyle::Dashed);
        assert_eq!(connector.arrow, Arrow::Double);
        assert_eq!(loaded.connectors, doc.connectors);
    }
}
