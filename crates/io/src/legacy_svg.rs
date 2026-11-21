//! Import of the SVG/XHTML produced by `vss2xhtml` (libvisio-tools).
//!
//! Legacy `.vss` files cannot expose their full shape data through
//! `vss2xhtml`, but the rendered SVG gives usable vector artwork. RackCraft
//! treats each top-level SVG as one master, adds four edge connectors and
//! imports fill/stroke styles.

use std::path::PathBuf;

use kurbo::Affine;
use rackcraft_engine::{Color, ConnectionPoint, DrawPath, PathCommand, Point, ShapeField};
use roxmltree::{Document as XmlDoc, Node, ParsingOptions};
use svgtypes::{SimplePathSegment, SimplifyingPathParser, TransformListParser, TransformListToken};

use crate::error::{IoError, Result};
use crate::model::{StencilLibrary, infer_tags};

const MAX_MASTERS: usize = 2_000;
const MAX_PATH_COMMANDS: usize = 100_000;
const SVG_MM_PER_POINT: f64 = 25.4 / 72.0;

/// Convert a `vss2xhtml` XHTML/SVG document into a stencil library.
pub fn load_vss2xhtml(
    xhtml: &[u8],
    display_name: &str,
    source: Option<String>,
) -> Result<StencilLibrary> {
    let text = String::from_utf8_lossy(xhtml);
    let document = XmlDoc::parse_with_options(
        &text,
        ParsingOptions {
            allow_dtd: true,
            ..ParsingOptions::default()
        },
    )
    .map_err(|err| IoError::xml("vss2xhtml output", err.to_string()))?;

    let svg_roots: Vec<Node<'_, '_>> = document
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "svg")
        .filter(|node| {
            node.ancestors()
                .skip(1)
                .all(|ancestor| ancestor.tag_name().name() != "svg")
        })
        .take(MAX_MASTERS)
        .collect();
    if svg_roots.is_empty() {
        return Err(IoError::Conversion(
            "vss2xhtml produced no SVG shapes".to_owned(),
        ));
    }

    let mut library = StencilLibrary {
        name: display_name.to_owned(),
        source,
        masters: Vec::new(),
        warnings: Vec::new(),
    };
    for (index, svg) in svg_roots.iter().enumerate() {
        let master = parse_svg_master(svg, index, display_name)?;
        library.masters.push(master);
    }
    library.warnings.push(
        "Converted with vss2xhtml: geometry is available but Visio master names and \
         shape data are limited. Install LibreOffice and re-save as .vssx for full metadata."
            .to_owned(),
    );
    Ok(library)
}

fn parse_svg_master(
    svg: &Node<'_, '_>,
    index: usize,
    library_name: &str,
) -> Result<crate::model::Master> {
    let (view_x, view_y, view_width, view_height) = view_box(svg);
    let width_mm = (view_width * SVG_MM_PER_POINT).max(1.0);
    let height_mm = (view_height * SVG_MM_PER_POINT).max(1.0);

    let mut graphics = Vec::new();
    let identity = Affine::IDENTITY;
    for child in svg.children().filter(Node::is_element) {
        collect_element(
            &child,
            identity,
            SvgStyle::default(),
            view_x,
            view_y,
            view_width,
            view_height,
            &mut graphics,
        );
    }

    let texts = collect_texts(svg);
    let title = texts
        .iter()
        .find(|text| {
            text.chars().any(char::is_alphabetic) && (2..=80).contains(&text.chars().count())
        })
        .cloned()
        .unwrap_or_else(|| format!("Legacy shape {}", index + 1));
    let name = if title.is_empty() {
        format!("{library_name} shape {}", index + 1)
    } else {
        title
    };
    let category = "Legacy .vss".to_owned();
    let tags = infer_tags(&name, &category);
    let mut keywords = format!("{name} {category} {library_name} vss legacy").to_lowercase();
    for text in &texts {
        keywords.push(' ');
        keywords.push_str(&text.to_lowercase());
    }
    let shape_data = vec![
        ShapeField {
            label: "Product Description".into(),
            value: name.clone(),
            prompt: String::new(),
        },
        ShapeField {
            label: "Source".into(),
            value: "vss2xhtml SVG conversion".into(),
            prompt: String::new(),
        },
    ];

    Ok(crate::model::Master {
        id: format!("legacy-{}", index + 1),
        name,
        category,
        width_mm,
        height_mm,
        graphics,
        connection_points: vec![
            ConnectionPoint::new("top", 0.5, 0.0).named("Top"),
            ConnectionPoint::new("right", 1.0, 0.5).named("Right"),
            ConnectionPoint::new("bottom", 0.5, 1.0).named("Bottom"),
            ConnectionPoint::new("left", 0.0, 0.5).named("Left"),
        ],
        shape_data,
        tags,
        mount: None,
        keywords,
        icon: None,
    })
}

fn view_box(svg: &Node<'_, '_>) -> (f64, f64, f64, f64) {
    if let Some(value) = svg.attribute("viewBox") {
        let nums: Vec<f64> = value
            .split(|character: char| character.is_whitespace() || character == ',')
            .filter_map(|part| part.parse().ok())
            .collect();
        if nums.len() == 4 && nums[2] > 0.0 && nums[3] > 0.0 {
            return (nums[0], nums[1], nums[2], nums[3]);
        }
    }
    let width = svg
        .attribute("width")
        .and_then(parse_svg_length)
        .unwrap_or(136.8);
    let height = svg
        .attribute("height")
        .and_then(parse_svg_length)
        .unwrap_or(72.0);
    (0.0, 0.0, width, height)
}

fn parse_svg_length(value: &str) -> Option<f64> {
    let value = value.trim();
    for (suffix, factor) in [
        ("in", 72.0),
        ("pt", 1.0),
        ("px", 1.0),
        ("mm", 72.0 / 25.4),
        ("cm", 72.0 / 2.54),
    ] {
        if let Some(number) = value.strip_suffix(suffix) {
            return number
                .trim()
                .parse::<f64>()
                .ok()
                .map(|number| number * factor);
        }
    }
    value.parse().ok()
}

#[derive(Clone, Copy)]
struct SvgStyle {
    fill: Option<Color>,
    stroke: Option<Color>,
    stroke_width_mm: f64,
}

impl Default for SvgStyle {
    fn default() -> Self {
        Self {
            fill: Some(Color::rgb(40, 44, 52)),
            stroke: None,
            stroke_width_mm: 0.3,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_element(
    node: &Node<'_, '_>,
    parent: Affine,
    inherited: SvgStyle,
    view_x: f64,
    view_y: f64,
    view_width: f64,
    view_height: f64,
    out: &mut Vec<DrawPath>,
) {
    if node.tag_name().name() == "defs" {
        return;
    }
    let transform = node_transform(node);
    let affine = parent * transform;
    let style = node_style(node, inherited);
    let normalizer = |x: f64, y: f64| ((x - view_x) / view_width, (y - view_y) / view_height);

    let element = node.tag_name().name();
    let mut commands = Vec::new();
    match element {
        "path" => {
            if let Some(data) = node.attribute("d") {
                for segment in SimplifyingPathParser::from(data).take(MAX_PATH_COMMANDS) {
                    let Ok(segment) = segment else {
                        continue;
                    };
                    match segment {
                        SimplePathSegment::MoveTo { x, y } => {
                            let p = affine * kurbo::Point::new(x, y);
                            let (nx, ny) = normalizer(p.x, p.y);
                            commands.push(PathCommand::MoveTo {
                                to: Point::new(nx, ny),
                            });
                        }
                        SimplePathSegment::LineTo { x, y } => {
                            let p = affine * kurbo::Point::new(x, y);
                            let (nx, ny) = normalizer(p.x, p.y);
                            commands.push(PathCommand::LineTo {
                                to: Point::new(nx, ny),
                            });
                        }
                        SimplePathSegment::Quadratic { x1, y1, x, y } => {
                            let c = affine * kurbo::Point::new(x1, y1);
                            let p = affine * kurbo::Point::new(x, y);
                            let (cx, cy) = normalizer(c.x, c.y);
                            let (nx, ny) = normalizer(p.x, p.y);
                            commands.push(PathCommand::QuadTo {
                                ctrl: Point::new(cx, cy),
                                to: Point::new(nx, ny),
                            });
                        }
                        SimplePathSegment::CurveTo {
                            x1,
                            y1,
                            x2,
                            y2,
                            x,
                            y,
                        } => {
                            let c1 = affine * kurbo::Point::new(x1, y1);
                            let c2 = affine * kurbo::Point::new(x2, y2);
                            let p = affine * kurbo::Point::new(x, y);
                            let (c1x, c1y) = normalizer(c1.x, c1.y);
                            let (c2x, c2y) = normalizer(c2.x, c2.y);
                            let (nx, ny) = normalizer(p.x, p.y);
                            commands.push(PathCommand::CubicTo {
                                c1: Point::new(c1x, c1y),
                                c2: Point::new(c2x, c2y),
                                to: Point::new(nx, ny),
                            });
                        }
                        SimplePathSegment::ClosePath => commands.push(PathCommand::Close),
                    }
                }
            }
        }
        "rect" => {
            let x = attribute_number(node, "x").unwrap_or(0.0);
            let y = attribute_number(node, "y").unwrap_or(0.0);
            let width = attribute_number(node, "width").unwrap_or(0.0);
            let height = attribute_number(node, "height").unwrap_or(0.0);
            if width > 0.0 && height > 0.0 {
                for (px, py) in [
                    (x, y),
                    (x + width, y),
                    (x + width, y + height),
                    (x, y + height),
                ] {
                    let p = affine * kurbo::Point::new(px, py);
                    let (nx, ny) = normalizer(p.x, p.y);
                    commands.push(if commands.is_empty() {
                        PathCommand::MoveTo {
                            to: Point::new(nx, ny),
                        }
                    } else {
                        PathCommand::LineTo {
                            to: Point::new(nx, ny),
                        }
                    });
                }
                commands.push(PathCommand::Close);
            }
        }
        "circle" | "ellipse" => {
            let cx = attribute_number(node, "cx").unwrap_or(0.0);
            let cy = attribute_number(node, "cy").unwrap_or(0.0);
            let rx = if element == "circle" {
                attribute_number(node, "r").unwrap_or(0.0)
            } else {
                attribute_number(node, "rx").unwrap_or(0.0)
            };
            let ry = if element == "circle" {
                rx
            } else {
                attribute_number(node, "ry").unwrap_or(rx)
            };
            if rx > 0.0 && ry > 0.0 {
                append_ellipse(&mut commands, affine, cx, cy, rx, ry, &normalizer);
            }
        }
        "line" => {
            let x1 = attribute_number(node, "x1").unwrap_or(0.0);
            let y1 = attribute_number(node, "y1").unwrap_or(0.0);
            let x2 = attribute_number(node, "x2").unwrap_or(0.0);
            let y2 = attribute_number(node, "y2").unwrap_or(0.0);
            for (index, (x, y)) in [(x1, y1), (x2, y2)].into_iter().enumerate() {
                let p = affine * kurbo::Point::new(x, y);
                let (nx, ny) = normalizer(p.x, p.y);
                commands.push(if index == 0 {
                    PathCommand::MoveTo {
                        to: Point::new(nx, ny),
                    }
                } else {
                    PathCommand::LineTo {
                        to: Point::new(nx, ny),
                    }
                });
            }
        }
        "g" | "a" | "switch" => {}
        _ => {}
    }

    if !commands.is_empty() {
        let drawable = commands.iter().any(|command| {
            matches!(
                command,
                PathCommand::LineTo { .. }
                    | PathCommand::QuadTo { .. }
                    | PathCommand::CubicTo { .. }
            )
        });
        if drawable && (style.fill.is_some() || style.stroke.is_some()) {
            out.push(DrawPath {
                commands,
                fill: style.fill,
                stroke: style.stroke,
                stroke_width_mm: style.stroke_width_mm,
            });
        }
    }

    for child in node.children().filter(Node::is_element) {
        collect_element(
            &child,
            affine,
            style,
            view_x,
            view_y,
            view_width,
            view_height,
            out,
        );
    }
}

fn append_ellipse(
    commands: &mut Vec<PathCommand>,
    affine: Affine,
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    normalize: &impl Fn(f64, f64) -> (f64, f64),
) {
    const KAPPA: f64 = 0.552_284_749_830_793_4;
    let kx = rx * KAPPA;
    let ky = ry * KAPPA;
    let point = |x: f64, y: f64| {
        let p = affine * kurbo::Point::new(x, y);
        let (nx, ny) = normalize(p.x, p.y);
        Point::new(nx, ny)
    };
    commands.push(PathCommand::MoveTo {
        to: point(cx + rx, cy),
    });
    for (c1x, c1y, c2x, c2y, ex, ey) in [
        (cx + rx, cy + ky, cx + kx, cy + ry, cx, cy + ry),
        (cx - kx, cy + ry, cx - rx, cy + ky, cx - rx, cy),
        (cx - rx, cy - ky, cx - kx, cy - ry, cx, cy - ry),
        (cx + kx, cy - ry, cx + rx, cy - ky, cx + rx, cy),
    ] {
        commands.push(PathCommand::CubicTo {
            c1: point(c1x, c1y),
            c2: point(c2x, c2y),
            to: point(ex, ey),
        });
    }
    commands.push(PathCommand::Close);
}

fn node_transform(node: &Node<'_, '_>) -> Affine {
    let Some(value) = node.attribute("transform") else {
        return Affine::IDENTITY;
    };
    let mut affine = Affine::IDENTITY;
    for token in TransformListParser::from(value) {
        let Ok(token) = token else {
            continue;
        };
        let transform = match token {
            TransformListToken::Matrix { a, b, c, d, e, f } => Affine::new([a, b, c, d, e, f]),
            TransformListToken::Translate { tx, ty } => Affine::translate(kurbo::Vec2::new(tx, ty)),
            TransformListToken::Scale { sx, sy } => Affine::new([sx, 0.0, 0.0, sy, 0.0, 0.0]),
            TransformListToken::Rotate { angle } => Affine::rotate(angle.to_radians()),
            TransformListToken::SkewX { angle } => {
                Affine::new([1.0, 0.0, angle.to_radians().tan(), 1.0, 0.0, 0.0])
            }
            TransformListToken::SkewY { angle } => {
                Affine::new([1.0, angle.to_radians().tan(), 0.0, 1.0, 0.0, 0.0])
            }
        };
        affine *= transform;
    }
    affine
}

fn node_style(node: &Node<'_, '_>, inherited: SvgStyle) -> SvgStyle {
    let mut style = inherited;
    for attribute in node.attributes() {
        apply_style(attribute.name(), attribute.value(), &mut style);
    }
    if let Some(css) = node.attribute("style") {
        for declaration in css.split(';') {
            if let Some((name, value)) = declaration.split_once(':') {
                apply_style(name.trim(), value.trim(), &mut style);
            }
        }
    }
    style
}

fn apply_style(name: &str, value: &str, style: &mut SvgStyle) {
    match name {
        "fill" => style.fill = parse_paint(value),
        "stroke" => style.stroke = parse_paint(value),
        "stroke-width" => {
            if let Some(width) = parse_svg_length(value) {
                style.stroke_width_mm = (width * SVG_MM_PER_POINT).max(0.05);
            }
        }
        "display" if value.trim() == "none" => {
            style.fill = None;
            style.stroke = None;
        }
        _ => {}
    }
}

fn parse_paint(value: &str) -> Option<Color> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("transparent") {
        return None;
    }
    if let Some(color) = Color::from_hex(value) {
        return Some(color);
    }
    match value.to_ascii_lowercase().as_str() {
        "black" => Some(Color::rgb(0, 0, 0)),
        "white" => Some(Color::rgb(255, 255, 255)),
        "red" => Some(Color::rgb(255, 0, 0)),
        "green" => Some(Color::rgb(0, 128, 0)),
        "blue" => Some(Color::rgb(0, 0, 255)),
        "gray" | "grey" => Some(Color::rgb(128, 128, 128)),
        _ => value
            .strip_prefix("rgb(")
            .and_then(|inner| inner.strip_suffix(')'))
            .and_then(|inner| {
                let numbers: Vec<u8> = inner
                    .split(',')
                    .filter_map(|part| part.trim().parse().ok())
                    .collect();
                (numbers.len() == 3).then(|| Color::rgb(numbers[0], numbers[1], numbers[2]))
            }),
    }
}

fn attribute_number(node: &Node<'_, '_>, name: &str) -> Option<f64> {
    node.attribute(name).and_then(parse_svg_length)
}

fn collect_texts(svg: &Node<'_, '_>) -> Vec<String> {
    let mut texts = Vec::new();
    for text in svg
        .descendants()
        .filter(|node| node.tag_name().name() == "text")
    {
        let value: String = text
            .descendants()
            .filter(|node| node.is_text())
            .filter_map(|node| node.text())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if !value.is_empty() && !texts.contains(&value) {
            texts.push(value);
        }
    }
    texts
}

/// Path to the externally provided `.vss` converter, used for diagnostics.
#[must_use]
pub fn default_vss2xhtml_name() -> PathBuf {
    PathBuf::from("vss2xhtml")
}

#[cfg(test)]
mod tests {
    use super::*;

    const XHTML: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:svg="http://www.w3.org/2000/svg">
<body>
<svg:svg version="1.1" width="1.9in" height="1in" viewBox="0 0 136.8 72">
  <svg:g transform="translate(2 3)">
    <svg:path d="M0,0 L100,0 L100,50 Z" style="fill: #262626; stroke: none;"/>
    <svg:rect x="5" y="5" width="20" height="15" style="fill: #e8573f;"/>
  </svg:g>
  <svg:text x="10" y="20">Core Router</svg:text>
</svg:svg>
</body></html>"##;

    #[test]
    fn parses_vss2xhtml_shapes() {
        let library = load_vss2xhtml(XHTML.as_bytes(), "Legacy", None).unwrap();
        assert_eq!(library.masters.len(), 1);
        let master = &library.masters[0];
        assert_eq!(master.name, "Core Router");
        assert_eq!(master.category, "Legacy .vss");
        assert_eq!(master.graphics.len(), 2);
        assert_eq!(master.connection_points.len(), 4);
        assert!((master.width_mm - 48.26).abs() < 0.1);
        assert!(!library.warnings.is_empty());
        assert!(matches!(
            master.graphics[0].commands[0],
            PathCommand::MoveTo { .. }
        ));
    }

    #[test]
    fn path_colors_and_transform_are_kept() {
        let library = load_vss2xhtml(XHTML.as_bytes(), "Legacy", None).unwrap();
        let master = &library.masters[0];
        assert_eq!(master.graphics[0].fill, Some(Color::rgb(38, 38, 38)));
        let PathCommand::MoveTo { to } = master.graphics[0].commands[0] else {
            panic!("expected move");
        };
        assert!((to.x - 2.0 / 136.8).abs() < 1e-6);
    }

    #[test]
    fn empty_svg_is_rejected() {
        let error = load_vss2xhtml(
            b"<html><body><p>no shapes</p></body></html>",
            "Legacy",
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("no SVG"));
    }
}
