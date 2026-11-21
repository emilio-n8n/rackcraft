//! Vector canvas rendering and view transform.

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use rackcraft_engine::{
    Canvas, Color, Document, DrawPath, Item, PathCommand, Point, SnapKind, SnapResult,
};
use rackcraft_io::Master;

use crate::theme;

/// Zoom and pan state for the central canvas.
#[derive(Debug, Clone)]
pub struct View {
    /// Pixels per millimetre.
    pub zoom: f32,
    /// Pan offset from the canvas area's top-left, in pixels.
    pub pan: Vec2,
    /// Whether the view has been fitted once.
    pub initialized: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            zoom: 0.55,
            pan: Vec2::new(32.0, 32.0),
            initialized: false,
        }
    }
}

impl View {
    /// Fit a document canvas inside a screen area with a margin.
    pub fn fit(&mut self, area: Rect, canvas: &Canvas) {
        let width = canvas.width_mm.max(1.0) as f32;
        let height = canvas.height_mm.max(1.0) as f32;
        let margin = 36.0;
        let available_w = (area.width() - margin * 2.0).max(80.0);
        let available_h = (area.height() - margin * 2.0).max(80.0);
        self.zoom = (available_w / width)
            .min(available_h / height)
            .clamp(0.08, 8.0);
        self.pan = Vec2::new(
            (area.width() - width * self.zoom) * 0.5,
            (area.height() - height * self.zoom) * 0.5,
        );
        self.initialized = true;
    }

    /// Fit with padding when the view has never been initialized.
    pub fn ensure_fit(&mut self, area: Rect, canvas: &Canvas) {
        if !self.initialized || !area.width().is_finite() || area.width() < 10.0 {
            self.fit(area, canvas);
        }
    }

    /// Convert a document point (mm) to screen coordinates.
    #[must_use]
    pub fn doc_to_screen(&self, origin: Pos2, point: Point) -> Pos2 {
        pos2(
            origin.x + self.pan.x + point.x as f32 * self.zoom,
            origin.y + self.pan.y + point.y as f32 * self.zoom,
        )
    }

    /// Convert a screen position to document millimetres.
    #[must_use]
    pub fn screen_to_doc(&self, origin: Pos2, screen: Pos2) -> Point {
        let zoom = self.zoom.max(1e-6);
        Point::new(
            f64::from((screen.x - origin.x - self.pan.x) / zoom),
            f64::from((screen.y - origin.y - self.pan.y) / zoom),
        )
    }

    /// Rectangle of the document canvas in screen coordinates.
    #[must_use]
    pub fn canvas_rect(&self, origin: Pos2, canvas: &Canvas) -> Rect {
        Rect::from_min_size(
            pos2(origin.x + self.pan.x, origin.y + self.pan.y),
            vec2(
                canvas.width_mm as f32 * self.zoom,
                canvas.height_mm as f32 * self.zoom,
            ),
        )
    }

    /// Zoom around a screen point, keeping the point under the cursor stable.
    pub fn zoom_around(&mut self, area: Rect, screen: Pos2, factor: f32) {
        let before = self.screen_to_doc(area.min, screen);
        self.zoom = (self.zoom * factor).clamp(0.06, 24.0);
        self.pan = Vec2::new(
            screen.x - area.min.x - before.x as f32 * self.zoom,
            screen.y - area.min.y - before.y as f32 * self.zoom,
        );
    }

    /// Zoom relative to the current zoom (percent readout).
    #[must_use]
    pub fn zoom_percent(&self) -> u32 {
        (self.zoom * 100.0).round().max(1.0) as u32
    }
}

/// Paint the canvas paper, shadow and engineering grid.
pub fn paint_background(painter: &egui::Painter, area: Rect, view: &View, canvas: &Canvas) {
    painter.rect_filled(area, CornerRadius::ZERO, Color32::from_rgb(15, 17, 22));
    let canvas_rect = view.canvas_rect(area.min, canvas);
    painter.rect_filled(
        canvas_rect.translate(vec2(0.0, 7.0)).expand(1.0),
        CornerRadius::same(4),
        Color32::from_black_alpha(80),
    );
    painter.rect(
        canvas_rect,
        CornerRadius::same(4),
        color_to_egui(canvas.background),
        Stroke::new(1.0, theme::BORDER_STRONG),
        StrokeKind::Inside,
    );

    if canvas.show_grid && canvas.grid_mm > 0.0 {
        let spacing = canvas.grid_mm as f32 * view.zoom;
        if spacing >= 6.0 {
            let clip = painter.clip_rect();
            let clip = clip.intersect(canvas_rect);
            if clip.is_positive() {
                let mut x = canvas_rect.left() + spacing;
                let mut index = 1;
                while x < canvas_rect.right() {
                    let colour = if index % 5 == 0 {
                        Color32::from_rgba_unmultiplied(96, 108, 128, 35)
                    } else {
                        Color32::from_rgba_unmultiplied(82, 92, 110, 18)
                    };
                    painter.line_segment(
                        [pos2(x, clip.top()), pos2(x, clip.bottom())],
                        Stroke::new(1.0, colour),
                    );
                    x += spacing;
                    index += 1;
                }
                let mut y = canvas_rect.top() + spacing;
                let mut index = 1;
                while y < canvas_rect.bottom() {
                    let colour = if index % 5 == 0 {
                        Color32::from_rgba_unmultiplied(96, 108, 128, 35)
                    } else {
                        Color32::from_rgba_unmultiplied(82, 92, 110, 18)
                    };
                    painter.line_segment(
                        [pos2(clip.left(), y), pos2(clip.right(), y)],
                        Stroke::new(1.0, colour),
                    );
                    y += spacing;
                    index += 1;
                }
            }
        }
    }
}

/// Paint every item in painter order, including selection, ports and labels.
pub fn paint_document(
    painter: &egui::Painter,
    area: Rect,
    view: &View,
    document: &Document,
    selected: Option<u64>,
    hovered: Option<u64>,
    drag_preview: Option<(u64, Point)>,
) {
    if document.items.is_empty() {
        let center = area.center() + vec2(0.0, -18.0);
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            "Drop a stencil here",
            FontId::proportional(19.0),
            theme::MUTED,
        );
        painter.text(
            center + vec2(0.0, 27.0),
            egui::Align2::CENTER_CENTER,
            "Drag from the stencil browser, or press Load sample",
            FontId::proportional(12.0),
            theme::FAINT,
        );
    }

    let mut items: Vec<&Item> = document.items.iter().collect();
    items.sort_by_key(|item| (item.z_index, item.id));
    for item in items {
        let position = drag_preview
            .filter(|(id, _)| *id == item.id)
            .map_or(item.position, |(_, position)| position);
        paint_item(
            painter,
            area,
            view,
            item,
            position,
            selected == Some(item.id),
            hovered == Some(item.id),
        );
    }
}

fn paint_item(
    painter: &egui::Painter,
    area: Rect,
    view: &View,
    item: &Item,
    position: Point,
    selected: bool,
    hovered: bool,
) {
    let origin = area.min;
    let top_left = view.doc_to_screen(origin, position);
    let size = vec2(
        item.size.width as f32 * view.zoom,
        item.size.height as f32 * view.zoom,
    );
    let bounds = Rect::from_min_size(top_left, size);
    if !bounds.expand(40.0).intersects(area) {
        return;
    }

    paint_art(painter, origin, view, item, position, None);

    if selected || hovered {
        let colour = if selected {
            theme::ACCENT
        } else {
            theme::BORDER_STRONG
        };
        painter.rect_stroke(
            bounds.expand(2.0),
            CornerRadius::same(2),
            Stroke::new(if selected { 1.6 } else { 1.0 }, colour),
            StrokeKind::Outside,
        );
    }

    if selected {
        for corner in [
            bounds.left_top(),
            bounds.right_top(),
            bounds.left_bottom(),
            bounds.right_bottom(),
        ] {
            let handle = Rect::from_center_size(corner, vec2(5.0, 5.0));
            painter.rect(
                handle,
                CornerRadius::same(1),
                theme::ACCENT,
                Stroke::new(1.0, theme::PANEL),
                StrokeKind::Inside,
            );
        }
        // Connection points are the heart of the snapping workflow.
        if item.connection_points.len() <= 128 {
            for connection in item.absolute_connections() {
                let point = view.doc_to_screen(origin, connection);
                if area.contains(point) {
                    painter.circle_filled(point, 2.4, theme::SNAP);
                    painter.circle_stroke(
                        point,
                        3.6,
                        Stroke::new(1.0, Color32::from_black_alpha(120)),
                    );
                }
            }
        }
    }

    if selected || hovered || item.is_rack() {
        let label = if item.name.chars().count() > 34 {
            format!("{}…", item.name.chars().take(33).collect::<String>())
        } else {
            item.name.clone()
        };
        let y = if item.is_rack() {
            bounds.top() + 7.0
        } else {
            bounds.bottom() + 3.0
        };
        painter.text(
            pos2(bounds.left(), y),
            egui::Align2::LEFT_TOP,
            label,
            FontId::proportional(if item.is_rack() { 11.5 } else { 9.5 }),
            if selected {
                theme::ACCENT_LIGHT
            } else {
                theme::MUTED
            },
        );
    }
}

/// Paint a master as a placement ghost.
pub fn paint_ghost(painter: &egui::Painter, area: Rect, view: &View, item: &Item, position: Point) {
    let origin = area.min;
    let top_left = view.doc_to_screen(origin, position);
    let bounds = Rect::from_min_size(
        top_left,
        vec2(
            item.size.width as f32 * view.zoom,
            item.size.height as f32 * view.zoom,
        ),
    );
    paint_art(
        painter,
        origin,
        view,
        item,
        position,
        Some((theme::ACCENT, 0.42)),
    );
    painter.rect_stroke(
        bounds.expand(1.0),
        CornerRadius::same(2),
        Stroke::new(1.2, theme::ACCENT_LIGHT),
        StrokeKind::Outside,
    );
    let label = format!("{} · drop to place", item.name);
    let pos = pos2(bounds.left(), bounds.top() - 18.0);
    painter.text(
        pos,
        egui::Align2::LEFT_TOP,
        label,
        FontId::proportional(10.5),
        theme::ACCENT_LIGHT,
    );
}

/// Paint the green magnetic snap indicator.
pub fn paint_snap(painter: &egui::Painter, area: Rect, view: &View, snap: &SnapResult) {
    let Some(indicator) = snap.indicator else {
        return;
    };
    let point = view.doc_to_screen(area.min, indicator);
    if !area.expand(10.0).contains(point) {
        return;
    }
    if snap.kind == Some(SnapKind::RackUnit) {
        painter.line_segment(
            [point - vec2(150.0, 0.0), point + vec2(150.0, 0.0)],
            Stroke::new(1.2, Color32::from_rgba_unmultiplied(84, 209, 132, 150)),
        );
    }
    painter.circle_filled(
        point,
        9.0,
        Color32::from_rgba_unmultiplied(84, 209, 132, 35),
    );
    painter.circle_filled(point, 4.2, theme::SNAP);
    painter.circle_stroke(point, 8.0, Stroke::new(1.2, theme::SNAP));
    painter.line_segment(
        [point - vec2(14.0, 0.0), point + vec2(14.0, 0.0)],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(84, 209, 132, 160)),
    );
    painter.line_segment(
        [point - vec2(0.0, 14.0), point + vec2(0.0, 14.0)],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(84, 209, 132, 160)),
    );

    let label = match snap.kind {
        Some(SnapKind::RackUnit) => "U SNAP",
        Some(SnapKind::ConnectionPoint) => "PORT SNAP",
        Some(SnapKind::Grid) => "GRID",
        None => return,
    };
    let bubble = Rect::from_min_size(point + vec2(14.0, -26.0), vec2(76.0, 18.0));
    painter.rect_filled(bubble, CornerRadius::same(4), Color32::from_rgb(24, 35, 28));
    painter.rect_stroke(
        bubble,
        CornerRadius::same(4),
        Stroke::new(1.0, theme::SNAP.gamma_multiply(0.5)),
        StrokeKind::Inside,
    );
    painter.text(
        bubble.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::monospace(9.5),
        theme::SNAP,
    );
}

/// Paint the zoom / hint HUD without intercepting input.
pub fn paint_hud(painter: &egui::Painter, area: Rect, view: &View, document: &Document) {
    let title = format!("{}  ·  {} items", document.name, document.items.len());
    let title_pos = area.left_top() + vec2(14.0, 12.0);
    let title_rect = Rect::from_min_size(title_pos - vec2(8.0, 5.0), vec2(260.0, 22.0));
    painter.rect_filled(
        title_rect,
        CornerRadius::same(5),
        Color32::from_rgba_unmultiplied(18, 20, 26, 215),
    );
    painter.text(
        title_pos,
        egui::Align2::LEFT_TOP,
        title,
        FontId::proportional(11.0),
        theme::MUTED,
    );

    let hints = "Wheel: zoom   ·   Middle/space-drag: pan   ·   Del: remove";
    painter.text(
        area.left_bottom() + vec2(14.0, -26.0),
        egui::Align2::LEFT_TOP,
        hints,
        FontId::proportional(10.0),
        theme::FAINT,
    );

    let zoom_text = format!("{}%", view.zoom_percent());
    let measure = Rect::from_min_size(area.right_bottom() - vec2(82.0, 32.0), vec2(68.0, 22.0));
    painter.rect_filled(
        measure,
        CornerRadius::same(5),
        Color32::from_rgba_unmultiplied(18, 20, 26, 225),
    );
    painter.rect_stroke(
        measure,
        CornerRadius::same(5),
        Stroke::new(1.0, theme::BORDER),
        StrokeKind::Inside,
    );
    painter.text(
        measure.center(),
        egui::Align2::CENTER_CENTER,
        zoom_text,
        FontId::monospace(10.5),
        theme::MUTED,
    );
}

/// Draw a stencil preview from the normalized artwork (no textures).
pub fn paint_master_thumbnail(painter: &egui::Painter, rect: Rect, master: &Master) {
    painter.rect_filled(rect, CornerRadius::same(4), theme::SUNKEN);
    let ratio = if master.height_mm > 0.0 {
        (master.width_mm / master.height_mm) as f32
    } else {
        1.0
    };
    let padding = 5.0;
    let available = rect.shrink(padding);
    let mut width = available.width();
    let mut height = width / ratio.max(0.01);
    if height > available.height() {
        height = available.height();
        width = height * ratio;
    }
    if width < 2.0 || height < 2.0 {
        return;
    }
    let box_rect = Rect::from_center_size(available.center(), vec2(width, height));

    for path in &master.graphics {
        let subpaths = flatten_path(path, |point| {
            pos2(
                box_rect.left() + point.x as f32 * box_rect.width(),
                box_rect.top() + point.y as f32 * box_rect.height(),
            )
        });
        for subpath in subpaths {
            if subpath.points.len() < 2 {
                continue;
            }
            let fill = path.fill.map(color_to_egui);
            let stroke = path
                .stroke
                .map(|colour| Stroke::new(0.7, color_to_egui(colour)));
            paint_subpath(painter, &subpath, fill, stroke);
        }
    }
}

fn paint_art(
    painter: &egui::Painter,
    origin: Pos2,
    view: &View,
    item: &Item,
    position: Point,
    tint: Option<(Color32, f32)>,
) {
    for path in &item.graphics {
        let subpaths = flatten_path(path, |point| {
            view.doc_to_screen(
                origin,
                Point::new(
                    position.x + point.x * item.size.width,
                    position.y + point.y * item.size.height,
                ),
            )
        });
        let stroke_width = (path.stroke_width_mm as f32 * view.zoom).clamp(0.7, 5.0);
        for subpath in subpaths {
            if subpath.points.len() < 2 {
                continue;
            }
            if let Some((colour, alpha)) = tint {
                let fill = Some(colour.gamma_multiply(alpha));
                let stroke = Some(Stroke::new(1.1, colour));
                paint_subpath(painter, &subpath, fill, stroke);
            } else {
                let fill = path.fill.map(color_to_egui);
                let stroke = path
                    .stroke
                    .map(|colour| Stroke::new(stroke_width, color_to_egui(colour)));
                paint_subpath(painter, &subpath, fill, stroke);
            }
        }
    }
}

#[derive(Debug)]
struct Subpath {
    points: Vec<Pos2>,
    closed: bool,
}

fn flatten_path(path: &DrawPath, map: impl Fn(Point) -> Pos2) -> Vec<Subpath> {
    let mut subpaths = Vec::new();
    let mut points: Vec<Pos2> = Vec::new();
    let mut closed = false;

    let mut flush = |points: &mut Vec<Pos2>, closed: &mut bool| {
        if points.len() >= 2 {
            subpaths.push(Subpath {
                points: std::mem::take(points),
                closed: *closed,
            });
        } else {
            points.clear();
        }
        *closed = false;
    };

    for command in &path.commands {
        match *command {
            PathCommand::MoveTo { to } => {
                flush(&mut points, &mut closed);
                points.push(map(to));
            }
            PathCommand::LineTo { to } => points.push(map(to)),
            PathCommand::QuadTo { ctrl, to } => {
                let Some(&start) = points.last() else {
                    points.push(map(to));
                    continue;
                };
                let ctrl = map(ctrl);
                let end = map(to);
                for step in 1..=12 {
                    let t = step as f32 / 12.0;
                    points.push(quad_point(start, ctrl, end, t));
                }
            }
            PathCommand::CubicTo { c1, c2, to } => {
                let Some(&start) = points.last() else {
                    points.push(map(to));
                    continue;
                };
                let c1 = map(c1);
                let c2 = map(c2);
                let end = map(to);
                for step in 1..=16 {
                    let t = step as f32 / 16.0;
                    points.push(cubic_point(start, c1, c2, end, t));
                }
            }
            PathCommand::Close => {
                if !points.is_empty() {
                    let first = points[0];
                    points.push(first);
                    closed = true;
                }
                flush(&mut points, &mut closed);
            }
        }
    }
    flush(&mut points, &mut closed);
    subpaths
}

fn quad_point(p0: Pos2, p1: Pos2, p2: Pos2, t: f32) -> Pos2 {
    let mt = 1.0 - t;
    pos2(
        mt * mt * p0.x + 2.0 * mt * t * p1.x + t * t * p2.x,
        mt * mt * p0.y + 2.0 * mt * t * p1.y + t * t * p2.y,
    )
}

fn cubic_point(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;
    pos2(
        mt2 * mt * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t2 * t * p3.x,
        mt2 * mt * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t2 * t * p3.y,
    )
}

fn paint_subpath(
    painter: &egui::Painter,
    subpath: &Subpath,
    fill: Option<Color32>,
    stroke: Option<Stroke>,
) {
    if subpath.points.len() < 2 {
        return;
    }
    let stroke = stroke.unwrap_or(Stroke::NONE);
    if let Some(fill) = fill.filter(|fill| fill.a() > 0) {
        let points = if subpath.closed || subpath.points.len() >= 3 {
            let mut points = subpath.points.clone();
            if points.first() != points.last()
                && let Some(first) = points.first().copied()
            {
                points.push(first);
            }
            points
        } else {
            subpath.points.clone()
        };
        painter.add(egui::Shape::convex_polygon(points, fill, stroke));
    } else if stroke != Stroke::NONE {
        if subpath.closed {
            painter.add(egui::Shape::closed_line(subpath.points.clone(), stroke));
        } else {
            painter.add(egui::Shape::line(subpath.points.clone(), stroke));
        }
    }
}

/// Convert an engine colour to egui.
#[must_use]
pub fn color_to_egui(color: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect::from_min_size(pos2(20.0, 30.0), vec2(800.0, 600.0))
    }

    #[test]
    fn screen_document_round_trip() {
        let view = View {
            zoom: 1.4,
            pan: vec2(17.0, 9.0),
            initialized: true,
        };
        let doc = Point::new(123.0, 456.0);
        let screen = view.doc_to_screen(area().min, doc);
        let back = view.screen_to_doc(area().min, screen);
        assert!((back.x - doc.x).abs() < 1e-3);
        assert!((back.y - doc.y).abs() < 1e-3);
    }

    #[test]
    fn zoom_keeps_pointer_anchor() {
        let mut view = View::default();
        let anchor = pos2(420.0, 260.0);
        let before = view.screen_to_doc(area().min, anchor);
        view.zoom_around(area(), anchor, 1.8);
        let after = view.screen_to_doc(area().min, anchor);
        assert!((before.x - after.x).abs() < 1e-6);
        assert!((before.y - after.y).abs() < 1e-6);
    }

    #[test]
    fn fit_places_canvas_inside_area() {
        let mut view = View::default();
        let canvas = Canvas::default();
        view.fit(area(), &canvas);
        let rect = view.canvas_rect(area().min, &canvas);
        assert!(rect.width() <= area().width());
        assert!(rect.height() <= area().height());
        assert!(view.initialized);
    }
}
