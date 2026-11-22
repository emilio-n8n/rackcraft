//! Native Visio `.vssx` (OPC + Visio XML) stencil parser.
//!
//! The parser deliberately works on local tag names, not namespace prefixes, so
//! it accepts the many Visio producers found in the wild. Geometry is flattened
//! through group transforms, converted to millimetres and normalized to the
//! master page, then stored as `rackcraft_engine::DrawPath`s.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::Arc;

use base64::Engine as _;
use rackcraft_engine::{ConnectionPoint, DrawPath, MountSpec, PathCommand, Point, ShapeField};
use roxmltree::{Document as XmlDoc, Node};
use zip::ZipArchive;

use crate::error::{IoError, Result};
use crate::model::{
    DEFAULT_FILL, DEFAULT_STROKE, IconImage, Master, StencilLibrary, first_u32, infer_category,
    infer_tags, shape_value,
};

const MASTERS_PART: &str = "visio/masters/masters.xml";
const MASTERS_RELS_PART: &str = "visio/masters/_rels/masters.xml.rels";
const MAX_MASTERS: usize = 2_000;
const MAX_SHAPES: usize = 20_000;
const MAX_COMMANDS_PER_PATH: usize = 50_000;
const MAX_CONNECTIONS: usize = 512;
const MAX_WARNINGS: usize = 50;

type Archive<'a> = ZipArchive<Cursor<&'a [u8]>>;

/// Parse a `.vssx` package from memory.
pub fn load_vssx_bytes(
    bytes: &[u8],
    display_name: &str,
    source: Option<String>,
) -> Result<StencilLibrary> {
    let mut archive = Archive::new(Cursor::new(bytes))?;
    let masters_xml = read_part(&mut archive, MASTERS_PART)
        .ok_or_else(|| IoError::MissingPart(MASTERS_PART.to_owned()))?;
    let masters_xml = String::from_utf8_lossy(&masters_xml);
    let master_doc =
        XmlDoc::parse(&masters_xml).map_err(|err| IoError::xml(MASTERS_PART, err.to_string()))?;
    let rels = read_part(&mut archive, MASTERS_RELS_PART)
        .map(|bytes| parse_relationships(&String::from_utf8_lossy(&bytes)))
        .unwrap_or_default();

    let mut library = StencilLibrary {
        name: display_name.to_owned(),
        source,
        package: None,
        masters: Vec::new(),
        warnings: Vec::new(),
    };

    let root = master_doc.root_element();
    let master_nodes: Vec<Node<'_, '_>> = root
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "Master")
        .collect();
    if master_nodes.is_empty() {
        return Err(IoError::NoMasters);
    }

    for (index, node) in master_nodes.into_iter().take(MAX_MASTERS).enumerate() {
        let part_path = master_part_path(index, &node, &rels);
        let part_xml = match read_part(&mut archive, &part_path) {
            Some(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            None => {
                push_warning(
                    &mut library,
                    format!("master part {part_path} is missing; using an empty placeholder"),
                );
                String::new()
            }
        };

        match parse_master(index, &node, &part_xml, &part_path, display_name) {
            Ok(mut master) => {
                master.foreign_vector_entries = collect_vector_entries(&mut archive, &part_path);
                library.masters.push(master);
            }
            Err(err) => push_warning(
                &mut library,
                format!("skipped master {index} ({part_path}): {err}"),
            ),
        }
    }

    if library.masters.is_empty() {
        return Err(IoError::NoMasters);
    }
    if library
        .masters
        .iter()
        .any(|master| !master.foreign_vector_entries.is_empty())
    {
        library.package = Some(Arc::from(bytes));
    }
    Ok(library)
}

fn collect_vector_entries(archive: &mut Archive<'_>, part_path: &str) -> Vec<String> {
    let rels_path = part_rels_path(part_path);
    let Some(bytes) = read_part(archive, &rels_path) else {
        return Vec::new();
    };
    let mut entries: Vec<String> = parse_relationships(&String::from_utf8_lossy(&bytes))
        .into_values()
        .filter(|target| {
            target.rsplit_once('.').is_some_and(|(_, extension)| {
                extension.eq_ignore_ascii_case("emf") || extension.eq_ignore_ascii_case("wmf")
            })
        })
        .collect();
    entries.sort();
    entries.dedup();
    entries
}

fn part_rels_path(part_path: &str) -> String {
    let (folder, filename) = part_path.rsplit_once('/').unwrap_or(("", part_path));
    format!("{folder}/_rels/{filename}.rels")
}

fn push_warning(library: &mut StencilLibrary, message: String) {
    if library.warnings.len() < MAX_WARNINGS {
        library.warnings.push(message);
    }
}

fn read_part(archive: &mut Archive<'_>, name: &str) -> Option<Vec<u8>> {
    let mut file = archive.by_name(name).ok()?;
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut bytes).ok()?;
    Some(bytes)
}

fn parse_relationships(xml: &str) -> BTreeMap<String, String> {
    let Ok(doc) = XmlDoc::parse(xml) else {
        return BTreeMap::new();
    };
    let mut map = BTreeMap::new();
    for node in doc
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "Relationship")
    {
        let Some(id) = node.attribute("Id") else {
            continue;
        };
        let Some(target) = node.attribute("Target") else {
            continue;
        };
        map.insert(id.to_owned(), normalize_part_path(target));
    }
    map
}

fn normalize_part_path(target: &str) -> String {
    let target = target.replace('\\', "/");
    let target = target.trim_start_matches('/');
    let base = if target.starts_with("visio/") {
        ""
    } else {
        "visio/masters/"
    };
    let full = format!("{base}{target}");
    let mut parts: Vec<&str> = Vec::new();
    for segment in full.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            segment => parts.push(segment),
        }
    }
    parts.join("/")
}

fn master_part_path(
    index: usize,
    master: &Node<'_, '_>,
    rels: &BTreeMap<String, String>,
) -> String {
    let relationship_id = master
        .children()
        .find(|child| child.is_element() && child.tag_name().name() == "Rel")
        .and_then(|rel| {
            rel.attributes()
                .find(|attr| attr.name() == "id")
                .map(|attr| attr.value().to_owned())
        });
    relationship_id
        .and_then(|id| rels.get(&id).cloned())
        .unwrap_or_else(|| {
            let id = master.attribute("ID").unwrap_or("0");
            format!("visio/masters/master{id}.xml")
        })
        .or_else_fallback(index)
}

trait FallbackPart {
    fn or_else_fallback(self, index: usize) -> Self;
}

impl FallbackPart for String {
    fn or_else_fallback(self, index: usize) -> Self {
        if self.is_empty() {
            format!("visio/masters/master{}.xml", index + 1)
        } else {
            self
        }
    }
}

fn parse_master(
    index: usize,
    node: &Node<'_, '_>,
    part_xml: &str,
    part_path: &str,
    library_name: &str,
) -> Result<Master> {
    let id = node
        .attribute("ID")
        .map(str::to_owned)
        .unwrap_or_else(|| (index + 1).to_string());
    let name = node
        .attribute("NameU")
        .or_else(|| node.attribute("Name"))
        .or_else(|| node.attribute("Prompt"))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("Master {id}"));

    let page_sheet = node
        .children()
        .find(|child| child.is_element() && child.tag_name().name() == "PageSheet");
    let width_in = page_sheet
        .and_then(|sheet| cell_value(&sheet, "PageWidth"))
        .filter(|value| *value > 0.0)
        .unwrap_or(1.0);
    let height_in = page_sheet
        .and_then(|sheet| cell_value(&sheet, "PageHeight"))
        .filter(|value| *value > 0.0)
        .unwrap_or(1.0);
    let width_mm = width_in * 25.4;
    let height_mm = height_in * 25.4;

    let icon = node
        .children()
        .find(|child| child.is_element() && child.tag_name().name() == "Icon")
        .and_then(|icon| icon.text())
        .and_then(decode_icon);

    let content = if part_xml.is_empty() {
        ParsedContent::default()
    } else {
        parse_master_contents(part_xml, part_path, width_in, height_in)?
    };

    let mut category = shape_value(&content.shape_data, &["Category", "Stencil Category"])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| infer_category(&name));
    if category == "General"
        && let Some(description) = shape_value(
            &content.shape_data,
            &["Product Description", "Description", "Product"],
        )
    {
        let inferred = infer_category(description);
        if inferred != "General" {
            category = inferred;
        }
    }
    if category == "General" {
        let inferred = infer_category(library_name);
        if inferred != "General" {
            category = inferred;
        }
    }
    if category.is_empty() {
        category = "General".into();
    }

    let declared_units = shape_value(&content.shape_data, &["Rack Units", "Rack Unit", "RU"])
        .and_then(first_u32)
        .filter(|units| (1..=100).contains(units));
    let mut tags = infer_tags(&name, &category);
    tags.sort();
    tags.dedup();

    let mount = if tags.iter().any(|tag| tag == "rack") {
        let units = declared_units
            .or_else(|| first_u32(&name).filter(|units| (1..=100).contains(units)))
            .unwrap_or(24);
        let mut spec = MountSpec::new(units);
        apply_custom_insets(&mut spec, &content.shape_data, width_mm, height_mm);
        Some(spec)
    } else {
        None
    };

    let mut keywords = format!("{name} {category} {library_name}").to_lowercase();
    for tag in &tags {
        keywords.push(' ');
        keywords.push_str(tag);
    }
    for field in &content.shape_data {
        keywords.push(' ');
        keywords.push_str(&field.value.to_lowercase());
    }

    Ok(Master {
        id,
        name,
        category,
        width_mm,
        height_mm,
        graphics: content.graphics,
        connection_points: content.connections,
        shape_data: content.shape_data,
        tags,
        mount,
        keywords,
        icon,
        foreign_vector_entries: Vec::new(),
    })
}

fn apply_custom_insets(
    spec: &mut MountSpec,
    shape_data: &[ShapeField],
    width_mm: f64,
    height_mm: f64,
) {
    if width_mm <= 0.0 || height_mm <= 0.0 {
        return;
    }
    if let Some(value) = shape_value(shape_data, &["Mount Left"]).and_then(parse_number) {
        spec.left = (value / width_mm).clamp(0.0, 0.45);
    }
    if let Some(value) = shape_value(shape_data, &["Mount Right"]).and_then(parse_number) {
        spec.right = (value / width_mm).clamp(0.0, 0.45);
    }
    if let Some(value) = shape_value(shape_data, &["Mount Top"]).and_then(parse_number) {
        spec.top = (value / height_mm).clamp(0.0, 0.45);
    }
    if let Some(value) = shape_value(shape_data, &["Mount Bottom"]).and_then(parse_number) {
        spec.bottom = (value / height_mm).clamp(0.0, 0.45);
    }
}

fn parse_number(text: &str) -> Option<f64> {
    text.trim().replace(',', ".").parse().ok()
}

#[derive(Debug, Default)]
struct ParsedContent {
    graphics: Vec<DrawPath>,
    connections: Vec<ConnectionPoint>,
    shape_data: Vec<ShapeField>,
}

struct ParseContext {
    page_width: f64,
    page_height: f64,
    content: ParsedContent,
    shapes_seen: usize,
}

fn parse_master_contents(
    xml: &str,
    part: &str,
    page_width: f64,
    page_height: f64,
) -> Result<ParsedContent> {
    let doc = XmlDoc::parse(xml).map_err(|err| IoError::xml(part, err.to_string()))?;
    let mut ctx = ParseContext {
        page_width,
        page_height,
        content: ParsedContent::default(),
        shapes_seen: 0,
    };
    let root = doc.root_element();
    for shapes in root
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "Shapes")
    {
        for shape in shapes
            .children()
            .filter(|node| node.is_element() && node.tag_name().name() == "Shape")
        {
            visit_shape(&shape, Affine::IDENTITY, &mut ctx, 0);
        }
    }
    Ok(ctx.content)
}

fn visit_shape(shape: &Node<'_, '_>, parent: Affine, ctx: &mut ParseContext, depth: usize) {
    if ctx.shapes_seen >= MAX_SHAPES || depth > 32 {
        return;
    }
    ctx.shapes_seen += 1;

    let width = cell_value(shape, "Width").unwrap_or(1.0).max(0.01);
    let height = cell_value(shape, "Height").unwrap_or(1.0).max(0.01);
    let matrix = parent.compose(shape_transform(shape, width, height));

    merge_shape_data(&mut ctx.content.shape_data, parse_shape_data(shape));

    let has_geometry = shape.children().any(|child| {
        child.is_element()
            && child.tag_name().name() == "Section"
            && child
                .attribute("N")
                .is_some_and(|name| name.eq_ignore_ascii_case("Geometry"))
    });
    if has_geometry {
        for section in shape.children().filter(|child| {
            child.is_element()
                && child.tag_name().name() == "Section"
                && child
                    .attribute("N")
                    .is_some_and(|name| name.eq_ignore_ascii_case("Geometry"))
        }) {
            parse_geometry_section(&section, shape, matrix, width, height, ctx);
        }
    } else if has_foreign_data(shape) {
        // Bitmap-only masters still need a visible box in the canvas.
        let fill = shape_fill(shape).unwrap_or(DEFAULT_FILL);
        let stroke = shape_line(shape)
            .map(|(colour, _)| colour)
            .unwrap_or(DEFAULT_STROKE);
        let commands = vec![
            PathCommand::MoveTo {
                to: normalize_point(matrix, 0.0, 0.0, ctx),
            },
            PathCommand::LineTo {
                to: normalize_point(matrix, width, 0.0, ctx),
            },
            PathCommand::LineTo {
                to: normalize_point(matrix, width, height, ctx),
            },
            PathCommand::LineTo {
                to: normalize_point(matrix, 0.0, height, ctx),
            },
            PathCommand::Close,
        ];
        ctx.content
            .graphics
            .push(DrawPath::filled(commands, fill).with_stroke(stroke, 0.25));
    }

    parse_connections(shape, matrix, ctx);

    if let Some(shapes) = shape
        .children()
        .find(|child| child.is_element() && child.tag_name().name() == "Shapes")
    {
        for child in shapes
            .children()
            .filter(|node| node.is_element() && node.tag_name().name() == "Shape")
        {
            visit_shape(&child, matrix, ctx, depth + 1);
        }
    }
}

fn parse_connections(shape: &Node<'_, '_>, matrix: Affine, ctx: &mut ParseContext) {
    let Some(section) = find_section(shape, "Connection") else {
        return;
    };
    for row in section
        .children()
        .filter(|child| child.is_element() && child.tag_name().name() == "Row")
    {
        if ctx.content.connections.len() >= MAX_CONNECTIONS {
            return;
        }
        let Some(x) = cell_value(&row, "X") else {
            continue;
        };
        let Some(y) = cell_value(&row, "Y") else {
            continue;
        };
        let point = normalize_point(matrix, x, y, ctx);
        let x = point.x.clamp(0.0, 1.0);
        let y = point.y.clamp(0.0, 1.0);
        if ctx
            .content
            .connections
            .iter()
            .any(|existing| (existing.x - x).abs() < 0.0005 && (existing.y - y).abs() < 0.0005)
        {
            continue;
        }
        let prompt = cell_string(&row, "Prompt")
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        let index = ctx.content.connections.len() + 1;
        let id = row
            .attribute("IX")
            .map(|ix| format!("conn-{ix}"))
            .unwrap_or_else(|| format!("conn-{index}"));
        let mut connection = ConnectionPoint::new(id, x, y);
        connection.name = prompt.or(Some(format!("Connector {index}")));
        ctx.content.connections.push(connection);
    }
}

fn parse_geometry_section(
    section: &Node<'_, '_>,
    shape: &Node<'_, '_>,
    matrix: Affine,
    width: f64,
    height: f64,
    ctx: &mut ParseContext,
) {
    if cell_value(section, "NoShow").is_some_and(|value| value != 0.0) {
        return;
    }
    let no_fill = cell_value(section, "NoFill").is_some_and(|value| value != 0.0);
    let no_line = cell_value(section, "NoLine").is_some_and(|value| value != 0.0);
    let fill = if no_fill {
        None
    } else {
        shape_fill(shape).or(Some(DEFAULT_FILL))
    };
    let (stroke, stroke_width_mm) = if no_line {
        (None, 0.25)
    } else {
        let (colour, width) = shape_line(shape).unwrap_or((DEFAULT_STROKE, 0.25));
        (Some(colour), width)
    };

    let mut commands = Vec::new();
    let mut start: Option<Point> = None;
    let mut last: Option<Point> = None;
    let mut has_segment = false;

    for row in section
        .children()
        .filter(|child| child.is_element() && child.tag_name().name() == "Row")
    {
        if commands.len() > MAX_COMMANDS_PER_PATH {
            break;
        }
        let Some(row_type) = row.attribute("T") else {
            continue;
        };
        let relative = row_type.starts_with("Rel");
        let Some(endpoint) = geometry_point(&row, relative, width, height) else {
            continue;
        };
        let end = normalize_point(matrix, endpoint.x, endpoint.y, ctx);

        match row_type {
            "MoveTo" | "RelMoveTo" => {
                commands.push(PathCommand::MoveTo { to: end });
                start = Some(end);
                last = Some(end);
            }
            "LineTo" | "RelLineTo" | "EllipticalArcTo" | "RelEllipticalArcTo" | "NURBSTo"
            | "SplineStart" | "SplineKnot" => {
                ensure_move(&mut commands, &mut start, last, end);
                commands.push(PathCommand::LineTo { to: end });
                last = Some(end);
                has_segment = true;
            }
            "CubBezTo" | "RelCubBezTo" => {
                let Some(c1) = geometry_point_named(&row, "A", "B", relative, width, height) else {
                    continue;
                };
                let Some(c2) = geometry_point_named(&row, "C", "D", relative, width, height) else {
                    continue;
                };
                ensure_move(&mut commands, &mut start, last, end);
                commands.push(PathCommand::CubicTo {
                    c1: normalize_point(matrix, c1.x, c1.y, ctx),
                    c2: normalize_point(matrix, c2.x, c2.y, ctx),
                    to: end,
                });
                last = Some(end);
                has_segment = true;
            }
            "QuadBezTo" | "RelQuadBezTo" => {
                let Some(ctrl) = geometry_point_named(&row, "A", "B", relative, width, height)
                else {
                    continue;
                };
                ensure_move(&mut commands, &mut start, last, end);
                commands.push(PathCommand::QuadTo {
                    ctrl: normalize_point(matrix, ctrl.x, ctrl.y, ctx),
                    to: end,
                });
                last = Some(end);
                has_segment = true;
            }
            "Ellipse" => {
                let cx = cell_value(&row, "X").unwrap_or(width * 0.5);
                let cy = cell_value(&row, "Y").unwrap_or(height * 0.5);
                let a = geometry_point_named(&row, "A", "B", false, width, height);
                let c = geometry_point_named(&row, "C", "D", false, width, height);
                let rx = a
                    .map(|point| (point.x - cx).hypot(point.y - cy))
                    .filter(|value| *value > 1e-6)
                    .unwrap_or(width * 0.5);
                let ry = c
                    .map(|point| (point.x - cx).hypot(point.y - cy))
                    .filter(|value| *value > 1e-6)
                    .unwrap_or(height * 0.5);
                append_ellipse(&mut commands, matrix, cx, cy, rx, ry, ctx);
                start = None;
                last = None;
                has_segment = true;
            }
            _ => {}
        }
    }

    if commands.is_empty() || !has_segment {
        return;
    }
    if fill.is_some() && !matches!(commands.last(), Some(PathCommand::Close)) {
        commands.push(PathCommand::Close);
    }

    let mut path = DrawPath {
        commands,
        fill,
        stroke,
        stroke_width_mm,
    };
    if !path.is_drawable() {
        return;
    }
    if path.stroke.is_none() && path.fill.is_none() {
        path.fill = Some(DEFAULT_FILL);
    }
    ctx.content.graphics.push(path);
}

fn ensure_move(
    commands: &mut Vec<PathCommand>,
    start: &mut Option<Point>,
    last: Option<Point>,
    end: Point,
) {
    if commands.is_empty() {
        commands.push(PathCommand::MoveTo { to: end });
        *start = Some(end);
    } else if start.is_none() {
        let reference = last.unwrap_or(end);
        commands.push(PathCommand::MoveTo { to: reference });
        *start = Some(reference);
    }
}

fn append_ellipse(
    commands: &mut Vec<PathCommand>,
    matrix: Affine,
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    ctx: &ParseContext,
) {
    const KAPPA: f64 = 0.552_284_749_830_793_4;
    let kx = rx * KAPPA;
    let ky = ry * KAPPA;
    commands.push(PathCommand::MoveTo {
        to: normalize_point(matrix, cx + rx, cy, ctx),
    });
    let segments = [
        (cx + rx, cy + ky, cx + kx, cy + ry, cx, cy + ry),
        (cx - kx, cy + ry, cx - rx, cy + ky, cx - rx, cy),
        (cx - rx, cy - ky, cx - kx, cy - ry, cx, cy - ry),
        (cx + kx, cy - ry, cx + rx, cy - ky, cx + rx, cy),
    ];
    for (c1x, c1y, c2x, c2y, ex, ey) in segments {
        commands.push(PathCommand::CubicTo {
            c1: normalize_point(matrix, c1x, c1y, ctx),
            c2: normalize_point(matrix, c2x, c2y, ctx),
            to: normalize_point(matrix, ex, ey, ctx),
        });
    }
    commands.push(PathCommand::Close);
}

fn geometry_point(row: &Node<'_, '_>, relative: bool, width: f64, height: f64) -> Option<Point> {
    let mut x = cell_value(row, "X")?;
    let mut y = cell_value(row, "Y")?;
    if relative {
        x *= width;
        y *= height;
    }
    Some(Point::new(x, y))
}

fn geometry_point_named(
    row: &Node<'_, '_>,
    x_cell: &str,
    y_cell: &str,
    relative: bool,
    width: f64,
    height: f64,
) -> Option<Point> {
    let mut x = cell_value(row, x_cell)?;
    let mut y = cell_value(row, y_cell)?;
    if relative {
        x *= width;
        y *= height;
    }
    Some(Point::new(x, y))
}

fn normalize_point(matrix: Affine, x: f64, y: f64, ctx: &ParseContext) -> Point {
    let page = matrix.apply(Point::new(x, y));
    let width = if ctx.page_width > 0.0 {
        ctx.page_width
    } else {
        1.0
    };
    let height = if ctx.page_height > 0.0 {
        ctx.page_height
    } else {
        1.0
    };
    Point::new(page.x / width, 1.0 - page.y / height)
}

fn parse_shape_data(shape: &Node<'_, '_>) -> Vec<ShapeField> {
    let mut fields = Vec::new();
    for section in shape.children().filter(|child| {
        child.is_element()
            && child.tag_name().name() == "Section"
            && child.attribute("N").is_some_and(|name| {
                name.eq_ignore_ascii_case("Property") || name.eq_ignore_ascii_case("ShapeData")
            })
    }) {
        for row in section
            .children()
            .filter(|child| child.is_element() && child.tag_name().name() == "Row")
        {
            let label = cell_string(&row, "Label")
                .as_deref()
                .map(str::trim)
                .filter(|label| !label.is_empty())
                .map(str::to_owned);
            let Some(label) = label else {
                continue;
            };
            let value = cell_string(&row, "Value").unwrap_or_default();
            let prompt = cell_string(&row, "Prompt")
                .as_deref()
                .map(str::trim)
                .filter(|prompt| !prompt.is_empty())
                .unwrap_or_default()
                .to_owned();
            fields.push(ShapeField {
                label,
                value: value.trim().to_owned(),
                prompt,
            });
        }
    }
    fields
}

fn merge_shape_data(target: &mut Vec<ShapeField>, incoming: Vec<ShapeField>) {
    for field in incoming {
        if let Some(existing) = target
            .iter_mut()
            .find(|existing| existing.label.eq_ignore_ascii_case(&field.label))
        {
            if existing.value.trim().is_empty() && !field.value.trim().is_empty() {
                existing.value = field.value;
            }
            if existing.prompt.is_empty() {
                existing.prompt = field.prompt;
            }
        } else {
            target.push(field);
        }
    }
}

fn shape_fill(shape: &Node<'_, '_>) -> Option<rackcraft_engine::Color> {
    let section = find_section(shape, "Fill")?;
    let pattern = cell_value(&section, "FillPattern").unwrap_or(1.0);
    if pattern == 0.0 {
        return None;
    }
    cell_string(&section, "FillForegnd").and_then(|value| parse_visio_color(&value))
}

fn shape_line(shape: &Node<'_, '_>) -> Option<(rackcraft_engine::Color, f64)> {
    let section = find_section(shape, "Line")?;
    let colour = cell_string(&section, "LineColor")
        .and_then(|value| parse_visio_color(&value))
        .unwrap_or(DEFAULT_STROKE);
    let width = cell_value(&section, "LineWeight")
        .map(|inches| (inches * 25.4).clamp(0.1, 3.0))
        .unwrap_or(0.25);
    Some((colour, width))
}

fn parse_visio_color(value: &str) -> Option<rackcraft_engine::Color> {
    rackcraft_engine::Color::from_hex(value.trim())
}

fn find_section<'a, 'input>(shape: &'a Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
    shape.children().find(|child| {
        child.is_element()
            && child.tag_name().name() == "Section"
            && child
                .attribute("N")
                .is_some_and(|section| section.eq_ignore_ascii_case(name))
    })
}

fn cell_value(node: &Node<'_, '_>, name: &str) -> Option<f64> {
    cell_node(node, name)
        .and_then(|cell| cell.attribute("V"))?
        .trim()
        .parse()
        .ok()
}

fn cell_string(node: &Node<'_, '_>, name: &str) -> Option<String> {
    cell_node(node, name)
        .and_then(|cell| cell.attribute("V"))
        .map(str::to_owned)
}

fn cell_node<'a, 'input>(node: &'a Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
    node.children().find(|child| {
        child.is_element()
            && child.tag_name().name() == "Cell"
            && child
                .attribute("N")
                .is_some_and(|cell_name| cell_name.eq_ignore_ascii_case(name))
    })
}

fn has_foreign_data(shape: &Node<'_, '_>) -> bool {
    shape
        .children()
        .any(|child| child.is_element() && child.tag_name().name() == "ForeignData")
}

fn shape_transform(shape: &Node<'_, '_>, width: f64, height: f64) -> Affine {
    let pin_x = cell_value(shape, "PinX").unwrap_or(0.0);
    let pin_y = cell_value(shape, "PinY").unwrap_or(0.0);
    let loc_x = cell_value(shape, "LocPinX").unwrap_or(width * 0.5);
    let loc_y = cell_value(shape, "LocPinY").unwrap_or(height * 0.5);
    let angle = cell_value(shape, "Angle").unwrap_or(0.0);
    let flip_x = cell_value(shape, "FlipX").is_some_and(|value| value.abs() > 0.5);
    let flip_y = cell_value(shape, "FlipY").is_some_and(|value| value.abs() > 0.5);
    Affine::translate(pin_x, pin_y)
        .compose(Affine::rotate(angle))
        .compose(Affine::scale(
            if flip_x { -1.0 } else { 1.0 },
            if flip_y { -1.0 } else { 1.0 },
        ))
        .compose(Affine::translate(-loc_x, -loc_y))
}

/// 2D affine transform in Visio's Y-up page coordinates.
#[derive(Debug, Clone, Copy)]
struct Affine {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Affine {
    const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    const fn translate(tx: f64, ty: f64) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx,
            f: ty,
        }
    }

    const fn scale(sx: f64, sy: f64) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    fn rotate(angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Return `self ∘ other`, i.e. apply `other` first.
    fn compose(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    fn apply(self, point: Point) -> Point {
        Point::new(
            self.a * point.x + self.c * point.y + self.e,
            self.b * point.x + self.d * point.y + self.f,
        )
    }
}

fn decode_icon(encoded: &str) -> Option<IconImage> {
    let cleaned: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(cleaned)
        .ok()?;
    parse_ico(&bytes)
}

pub(crate) fn parse_ico(bytes: &[u8]) -> Option<IconImage> {
    if bytes.len() < 22 {
        return None;
    }
    let icon_type = u16::from_le_bytes([bytes[2], bytes[3]]);
    let count = u16::from_le_bytes([bytes[4], bytes[5]]);
    if icon_type != 1 || count == 0 {
        return None;
    }
    let entry = &bytes[6..22];
    let size = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as usize;
    let offset = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize;
    let data = bytes.get(offset..offset.checked_add(size)?)?;
    if data.starts_with(b"\x89PNG") {
        // PNG-compressed icons are valid but need a PNG decoder; the UI falls
        // back to vector previews for those.
        return None;
    }
    decode_dib_icon(data)
}

/// Decode an uncompressed ICO DIB payload (1, 4, 8 or 32 bits per pixel).
fn decode_dib_icon(data: &[u8]) -> Option<IconImage> {
    if data.len() < 40 {
        return None;
    }
    let header_size = u32::from_le_bytes(data[0..4].try_into().ok()?) as usize;
    if header_size < 40 || header_size > data.len() {
        return None;
    }
    let dib_width = i32::from_le_bytes(data[4..8].try_into().ok()?);
    let dib_height = i32::from_le_bytes(data[8..12].try_into().ok()?);
    let bit_count = u16::from_le_bytes(data[14..16].try_into().ok()?);
    let compression = u32::from_le_bytes(data[16..20].try_into().ok()?);
    let colours_used = u32::from_le_bytes(data[32..36].try_into().ok()?);
    if dib_width <= 0 || dib_height <= 0 || compression != 0 {
        return None;
    }
    if !matches!(bit_count, 1 | 4 | 8 | 24 | 32) {
        return None;
    }
    let width = usize::try_from(dib_width).ok()?;
    let height = (dib_height / 2) as usize;
    if width == 0 || height == 0 || width > 256 || height > 256 {
        return None;
    }

    let palette_count = if bit_count <= 8 {
        let declared = if colours_used == 0 {
            1usize << bit_count
        } else {
            colours_used as usize
        };
        declared.min(256)
    } else {
        0
    };
    let palette_bytes = palette_count.checked_mul(4)?;
    let pixel_start = header_size.checked_add(palette_bytes)?;
    let stride = if bit_count == 32 {
        width.checked_mul(4)?
    } else if bit_count == 24 {
        (width.checked_mul(3)? + 3) & !3
    } else {
        (width.checked_mul(usize::from(bit_count))?.div_ceil(32)) * 4
    };
    let pixels_len = stride.checked_mul(height)?;
    let pixels_end = pixel_start.checked_add(pixels_len)?;
    if data.len() < pixels_end {
        return None;
    }
    let palette = &data[header_size..header_size + palette_bytes];

    let mut rgba = vec![0_u8; width.checked_mul(height)?.checked_mul(4)?];
    let mut any_alpha = false;
    for row in 0..height {
        let source_row = pixel_start + (height - 1 - row) * stride;
        for column in 0..width {
            let destination = (row * width + column) * 4;
            let (r, g, b, a) = match bit_count {
                32 => {
                    let source = source_row + column * 4;
                    (
                        data[source + 2],
                        data[source + 1],
                        data[source],
                        data[source + 3],
                    )
                }
                24 => {
                    let source = source_row + column * 3;
                    (data[source + 2], data[source + 1], data[source], 255)
                }
                _ => {
                    let bit_offset = column * usize::from(bit_count);
                    let byte = data[source_row + bit_offset / 8];
                    let shift = 8 - usize::from(bit_count) - (bit_offset % 8);
                    let mask = (1_u16 << bit_count) - 1;
                    let index = usize::from((u16::from(byte) >> shift) & mask);
                    let entry = palette.get(index * 4..index * 4 + 4)?;
                    (entry[2], entry[1], entry[0], 255)
                }
            };
            any_alpha |= a != 0;
            rgba[destination] = r;
            rgba[destination + 1] = g;
            rgba[destination + 2] = b;
            rgba[destination + 3] = a;
        }
    }

    // Apply the 1-bpp AND mask when present: 1 means transparent.
    let mask_start = pixels_end;
    let mask_stride = width.div_ceil(32) * 4;
    if data.len() >= mask_start + mask_stride * height {
        for row in 0..height {
            let mask_row = mask_start + (height - 1 - row) * mask_stride;
            for column in 0..width {
                let bit = (data[mask_row + column / 8] >> (7 - (column % 8))) & 1;
                if bit != 0 {
                    rgba[(row * width + column) * 4 + 3] = 0;
                } else if !any_alpha {
                    rgba[(row * width + column) * 4 + 3] = 255;
                }
            }
        }
    } else if !any_alpha {
        for alpha in rgba.iter_mut().skip(3).step_by(4) {
            *alpha = 255;
        }
    }

    Some(IconImage {
        width: u32::try_from(width).ok()?,
        height: u32::try_from(height).ok()?,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const SIMPLE_MASTER: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<Masters xmlns="http://schemas.microsoft.com/office/visio/2012/main"
         xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <Master ID="1" NameU="Test Patch Panel" Name="Test Patch Panel">
    <PageSheet><Cell N="PageWidth" V="19"/><Cell N="PageHeight" V="1.75"/></PageSheet>
    <Icon>AAABAAEAICAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA</Icon>
    <Rel r:id="rId1"/>
  </Master>
</Masters>"##;

    const RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.microsoft.com/visio/2010/relationships/master"
                Target="master1.xml"/>
</Relationships>"#;

    const MASTER_CONTENTS: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<MasterContents xmlns="http://schemas.microsoft.com/office/visio/2012/main">
  <Shapes>
    <Shape ID="1" Type="Shape">
      <Cell N="PinX" V="9.5"/><Cell N="PinY" V="0.875"/>
      <Cell N="Width" V="19"/><Cell N="Height" V="1.75"/>
      <Cell N="LocPinX" V="9.5"/><Cell N="LocPinY" V="0.875"/>
      <Section N="Fill"><Cell N="FillForegnd" V="#2b3140"/></Section>
      <Section N="Line"><Cell N="LineColor" V="#5e6a80"/><Cell N="LineWeight" V="0.01"/></Section>
      <Section N="Property">
        <Row N="Row_1"><Cell N="Label" V="Manufacturer"/><Cell N="Value" V="RackCraft"/></Row>
        <Row N="Row_2"><Cell N="Label" V="Part Number"/><Cell N="Value" V="RC-PP-24"/></Row>
        <Row N="Row_3"><Cell N="Label" V="Product Description"/><Cell N="Value" V="24-port panel"/></Row>
        <Row N="Row_4"><Cell N="Label" V="Rack Units"/><Cell N="Value" V="1"/></Row>
      </Section>
      <Section N="Connection">
        <Row IX="0"><Cell N="X" V="1.0"/><Cell N="Y" V="0.875"/></Row>
        <Row IX="1"><Cell N="X" V="18.0"/><Cell N="Y" V="0.875"/></Row>
      </Section>
      <Section N="Geometry" IX="0">
        <Cell N="NoFill" V="0"/><Cell N="NoLine" V="0"/>
        <Row T="MoveTo" IX="1"><Cell N="X" V="0"/><Cell N="Y" V="0"/></Row>
        <Row T="LineTo" IX="2"><Cell N="X" V="19"/><Cell N="Y" V="0"/></Row>
        <Row T="LineTo" IX="3"><Cell N="X" V="19"/><Cell N="Y" V="1.75"/></Row>
        <Row T="LineTo" IX="4"><Cell N="X" V="0"/><Cell N="Y" V="1.75"/></Row>
        <Row T="LineTo" IX="5"><Cell N="X" V="0"/><Cell N="Y" V="0"/></Row>
      </Section>
      <Section N="Geometry" IX="1">
        <Cell N="NoFill" V="1"/><Cell N="NoLine" V="0"/>
        <Row T="RelMoveTo" IX="1"><Cell N="X" V="0"/><Cell N="Y" V="0.5"/></Row>
        <Row T="RelLineTo" IX="2"><Cell N="X" V="1"/><Cell N="Y" V="0.5"/></Row>
      </Section>
    </Shape>
  </Shapes>
</MasterContents>"##;

    fn make_zip() -> Vec<u8> {
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buffer);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("visio/masters/masters.xml", options)
                .unwrap();
            zip.write_all(SIMPLE_MASTER.as_bytes()).unwrap();
            zip.start_file("visio/masters/_rels/masters.xml.rels", options)
                .unwrap();
            zip.write_all(RELS.as_bytes()).unwrap();
            zip.start_file("visio/masters/master1.xml", options)
                .unwrap();
            zip.write_all(MASTER_CONTENTS.as_bytes()).unwrap();
            zip.start_file("visio/masters/_rels/master1.xml.rels", options)
                .unwrap();
            zip.write_all(
                br#"<?xml version="1.0" encoding="UTF-8"?>
                <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
                  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.emf"/>
                </Relationships>"#,
            )
            .unwrap();
            zip.start_file("visio/media/image1.emf", options).unwrap();
            zip.write_all(b"not a real metafile").unwrap();
            zip.finish().unwrap();
        }
        buffer.into_inner()
    }

    #[test]
    fn parses_master_geometry_data_and_connections() {
        let library = load_vssx_bytes(&make_zip(), "Test", None).unwrap();
        assert_eq!(library.masters.len(), 1);
        let master = &library.masters[0];
        assert_eq!(master.name, "Test Patch Panel");
        assert_eq!(master.category, "Patch panels");
        assert!((master.width_mm - 482.6).abs() < 1e-9);
        assert!((master.height_mm - 44.45).abs() < 1e-9);
        assert_eq!(master.connection_points.len(), 2);
        assert_eq!(master.graphics.len(), 2);
        assert_eq!(master.shape_data.len(), 4);
        assert!(!master.keywords.is_empty());
        assert_eq!(
            master.foreign_vector_entries,
            vec!["visio/media/image1.emf".to_owned()]
        );
        assert!(library.package.is_some());
    }

    #[test]
    fn shape_data_and_fill_are_extracted() {
        let library = load_vssx_bytes(&make_zip(), "Test", None).unwrap();
        let master = &library.masters[0];
        assert_eq!(master.shape_data[0].label, "Manufacturer");
        assert_eq!(master.shape_data[0].value, "RackCraft");
        assert_eq!(
            master.graphics[0].fill,
            Some(rackcraft_engine::Color::rgb(43, 49, 64))
        );
        let other = master
            .shape_data
            .iter()
            .find(|field| field.label == "Rack Units")
            .unwrap();
        assert_eq!(other.value, "1");
    }

    #[test]
    fn relative_geometry_is_scaled_to_the_shape() {
        let library = load_vssx_bytes(&make_zip(), "Test", None).unwrap();
        let master = &library.masters[0];
        let line = &master.graphics[1];
        assert!(matches!(line.commands[0], PathCommand::MoveTo { .. }));
        let PathCommand::LineTo { to } = line.commands[1] else {
            panic!("expected line");
        };
        assert!((to.x - 1.0).abs() < 1e-6);
        assert!((to.y - 0.5).abs() < 1e-6);
    }

    #[test]
    fn instantiated_item_moves_project_fields() {
        let library = load_vssx_bytes(&make_zip(), "Test", None).unwrap();
        let mut master = library.masters[0].clone();
        master.shape_data.push(ShapeField {
            label: "Location".into(),
            value: "Row A".into(),
            prompt: String::new(),
        });
        let item = master.to_item("Test", Point::new(10.0, 20.0));
        assert_eq!(item.project.location, "Row A");
        assert!(
            !item
                .shape_data
                .iter()
                .any(|field| field.label == "Location")
        );
        assert_eq!(item.part_number(), "RC-PP-24");
    }

    #[test]
    fn invalid_png_icon_is_ignored() {
        assert!(parse_ico(b"not an icon").is_none());
    }

    #[test]
    fn decodes_paletted_four_bit_icons() {
        let width = 2_u32;
        let height = 2_u32;
        let mut dib = Vec::new();
        dib.extend_from_slice(&40_u32.to_le_bytes());
        dib.extend_from_slice(&(width as i32).to_le_bytes());
        dib.extend_from_slice(&((height * 2) as i32).to_le_bytes());
        dib.extend_from_slice(&1_u16.to_le_bytes());
        dib.extend_from_slice(&4_u16.to_le_bytes());
        dib.extend_from_slice(&0_u32.to_le_bytes());
        dib.extend_from_slice(&(width * height).to_le_bytes());
        dib.extend_from_slice(&0_i32.to_le_bytes());
        dib.extend_from_slice(&0_i32.to_le_bytes());
        dib.extend_from_slice(&16_u32.to_le_bytes());
        dib.extend_from_slice(&0_u32.to_le_bytes());
        for index in 0..16_u8 {
            let shade = if index == 1 { 0 } else { 255 };
            dib.extend_from_slice(&[shade, shade, shade, 0]);
        }
        // Two 4-bpp rows, each padded to 32 bits. Rows are bottom-up:
        // the last row is the top one and reads white, black.
        dib.extend_from_slice(&[0x10, 0x00, 0x00, 0x00]);
        dib.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        // AND mask: two transparent rows of 32-bit padded data.
        dib.extend_from_slice(&[0, 0, 0, 0]);
        dib.extend_from_slice(&[0, 0, 0, 0]);

        let mut ico = vec![0, 0, 1, 0, 1, 0, 2, 2, 0, 0, 1, 0, 4, 0];
        ico.extend_from_slice(&(dib.len() as u32).to_le_bytes());
        ico.extend_from_slice(&22_u32.to_le_bytes());
        ico.extend_from_slice(&dib);
        let icon = parse_ico(&ico).expect("paletted icon decodes");
        assert_eq!((icon.width, icon.height), (2, 2));
        assert_eq!(&icon.rgba[0..4], &[255, 255, 255, 255]);
        assert_eq!(&icon.rgba[4..8], &[0, 0, 0, 255]);
    }

    #[test]
    fn relative_part_paths_are_normalized() {
        assert_eq!(
            normalize_part_path("master1.xml"),
            "visio/masters/master1.xml"
        );
        assert_eq!(normalize_part_path("../master1.xml"), "visio/master1.xml");
        assert_eq!(
            normalize_part_path("/visio/masters/master2.xml"),
            "visio/masters/master2.xml"
        );
    }

    #[test]
    fn affine_composition_matches_visio_order() {
        let matrix = Affine::translate(10.0, 20.0).compose(Affine::scale(2.0, 3.0));
        let point = matrix.apply(Point::new(1.0, 1.0));
        assert!((point.x - 12.0).abs() < 1e-9);
        assert!((point.y - 23.0).abs() < 1e-9);

        let rotated = Affine::rotate(std::f64::consts::FRAC_PI_2).apply(Point::new(1.0, 0.0));
        assert!(rotated.x.abs() < 1e-9);
        assert!((rotated.y - 1.0).abs() < 1e-9);
    }

    #[test]
    fn detects_formats_from_magic() {
        use crate::model::{StencilFormat, detect_format};
        assert_eq!(detect_format(b"PK\x03\x04rest"), StencilFormat::Vssx);
        assert_eq!(
            detect_format(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]),
            StencilFormat::Vss
        );
        assert_eq!(detect_format(b"nope"), StencilFormat::Unknown);
    }
}
