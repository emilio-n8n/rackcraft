//! Small, serializable 2D geometry types (millimetres, Y grows downwards).

use serde::{Deserialize, Serialize};

/// A point in document space, in millimetres.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a point.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Euclidean distance to another point.
    #[must_use]
    pub fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }

    /// Return this point translated by `(dx, dy)`.
    #[must_use]
    pub fn translated(self, dx: f64, dy: f64) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }

    /// Linear interpolation.
    #[must_use]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

/// A width/height pair in millimetres.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    /// Create a size.
    #[must_use]
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Whether either dimension is zero or negative.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}

/// An axis-aligned rectangle in millimetres.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    /// Create a rectangle from its top-left corner and size.
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Left edge.
    #[must_use]
    pub const fn left(self) -> f64 {
        self.x
    }

    /// Top edge.
    #[must_use]
    pub const fn top(self) -> f64 {
        self.y
    }

    /// Right edge.
    #[must_use]
    pub const fn right(self) -> f64 {
        self.x + self.width
    }

    /// Bottom edge.
    #[must_use]
    pub const fn bottom(self) -> f64 {
        self.y + self.height
    }

    /// Centre point.
    #[must_use]
    pub const fn center(self) -> Point {
        Point::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    /// Whether a point lies inside (edges included).
    #[must_use]
    pub fn contains(self, p: Point) -> bool {
        p.x >= self.left() && p.x <= self.right() && p.y >= self.top() && p.y <= self.bottom()
    }

    /// Whether this rectangle intersects another one.
    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    /// Smallest rectangle that contains both rectangles.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        let left = self.left().min(other.left());
        let top = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self::new(left, top, right - left, bottom - top)
    }

    /// Clamp a position so a rectangle of `size` stays inside this rectangle
    /// when possible. If `size` is larger, its top-left is clamped to the top-left.
    #[must_use]
    pub fn clamp_position(self, size: Size, pos: Point) -> Point {
        let max_x = (self.right() - size.width).max(self.left());
        let max_y = (self.bottom() - size.height).max(self.top());
        Point::new(
            pos.x.clamp(self.left(), max_x),
            pos.y.clamp(self.top(), max_y),
        )
    }
}

impl From<Rect> for Size {
    fn from(rect: Rect) -> Self {
        Self::new(rect.width, rect.height)
    }
}

/// Straight, alpha-premultiplied-free sRGB colour stored as 8-bit channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Opaque colour.
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Colour with an explicit alpha channel.
    #[must_use]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse `#RRGGBB` or `#RRGGBBAA`.
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let value = hex.strip_prefix('#').unwrap_or(hex);
        let parse = |range: std::ops::Range<usize>| {
            value
                .get(range)
                .and_then(|part| u8::from_str_radix(part, 16).ok())
        };
        match value.len() {
            6 => Some(Self::rgb(parse(0..2)?, parse(2..4)?, parse(4..6)?)),
            8 => Some(Self::rgba(
                parse(0..2)?,
                parse(2..4)?,
                parse(4..6)?,
                parse(6..8)?,
            )),
            _ => None,
        }
    }

    /// Multiply the alpha channel, useful for hover/selection overlays.
    #[must_use]
    pub fn with_alpha(self, alpha: u8) -> Self {
        Self { a: alpha, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_helpers() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.right(), 110.0);
        assert_eq!(r.bottom(), 70.0);
        assert_eq!(r.center(), Point::new(60.0, 45.0));
        assert!(r.contains(Point::new(10.0, 20.0)));
        assert!(!r.contains(Point::new(111.0, 30.0)));
        assert!(r.intersects(Rect::new(50.0, 0.0, 20.0, 30.0)));
    }

    #[test]
    fn color_hex_parsing() {
        assert_eq!(Color::from_hex("#e8573f"), Some(Color::rgb(232, 87, 63)));
        assert_eq!(
            Color::from_hex("11223344"),
            Some(Color::rgba(0x11, 0x22, 0x33, 0x44))
        );
        assert_eq!(Color::from_hex("#12345"), None);
    }

    #[test]
    fn union_covers_both_rectangles() {
        let a = Rect::new(10.0, 20.0, 30.0, 40.0);
        let b = Rect::new(50.0, 5.0, 10.0, 10.0);
        assert_eq!(a.union(b), Rect::new(10.0, 5.0, 50.0, 55.0));
        assert_eq!(a.union(a), a);
    }

    #[test]
    fn clamp_keeps_items_inside() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert_eq!(
            r.clamp_position(Size::new(10.0, 10.0), Point::new(-5.0, 500.0)),
            Point::new(0.0, 90.0)
        );
    }
}
