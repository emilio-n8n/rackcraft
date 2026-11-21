//! Vector PDF export.
//!
//! Page 1 is a vector rendering of the complete schematic on A4 landscape with
//! a project cartouche. Following pages contain the grouped inventory. The
//! writer only uses the standard Helvetica PDF fonts (WinAnsi encoding), so no
//! font files or platform libraries are required.

use std::path::Path;

use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str, TextStr};
use rackcraft_engine::{
    Arrow, Color, Connector, Document, DrawPath, LineStyle, PathCommand, Point,
};
use time::OffsetDateTime;

use crate::error::Result;

const PAGE_W: f32 = 841.89;
const PAGE_H: f32 = 595.28;
const MARGIN: f32 = 28.0;
const FONT: Name<'static> = Name(b"F1");
const FONT_BOLD: Name<'static> = Name(b"F2");
const INVENTORY_ROWS: usize = 22;

/// Export a document to a vector PDF file.
///
/// # Errors
///
/// Returns an error when the file cannot be written.
pub fn export_pdf(document: &Document, path: impl AsRef<Path>) -> Result<()> {
    let bytes = pdf_bytes(document);
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Build the complete PDF in memory.
///
/// The result is deterministic for a given document and current UTC date.
#[must_use]
pub fn pdf_bytes(document: &Document) -> Vec<u8> {
    let bom = document.bom();
    let inventory_pages = bom.lines.len().div_ceil(INVENTORY_ROWS).max(1);
    let page_count = 1 + inventory_pages;
    let date = current_date();

    let mut pdf = Pdf::new();
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let regular_font_id = Ref::new(3);
    let bold_font_id = Ref::new(4);
    let info_id = Ref::new(5);
    let mut page_ids = Vec::with_capacity(page_count);
    let mut content_ids = Vec::with_capacity(page_count);
    let mut next_ref = 6;
    for _ in 0..page_count {
        page_ids.push(Ref::new(next_ref));
        content_ids.push(Ref::new(next_ref + 1));
        next_ref += 2;
    }

    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_count as i32);
    pdf.document_info(info_id)
        .title(TextStr(document.name.as_str()))
        .author(TextStr("RackCraft contributors"))
        .creator(TextStr("RackCraft"))
        .producer(TextStr("RackCraft vector PDF exporter"));
    pdf.type1_font(regular_font_id)
        .base_font(Name(b"Helvetica"))
        .encoding_predefined(Name(b"WinAnsiEncoding"));
    pdf.type1_font(bold_font_id)
        .base_font(Name(b"Helvetica-Bold"))
        .encoding_predefined(Name(b"WinAnsiEncoding"));

    for (index, (&page_id, &content_id)) in page_ids.iter().zip(&content_ids).enumerate() {
        let mut page = pdf.page(page_id);
        page.media_box(Rect::new(0.0, 0.0, PAGE_W, PAGE_H));
        page.parent(page_tree_id);
        page.contents(content_id);
        page.resources()
            .fonts()
            .pair(FONT, regular_font_id)
            .pair(FONT_BOLD, bold_font_id);
        page.finish();

        let content = if index == 0 {
            render_schematic(document, &bom, &document.name, &date, page_count)
        } else {
            let page_index = index - 1;
            render_inventory(
                document,
                &bom,
                &document.name,
                &date,
                page_index,
                inventory_pages,
                page_count,
            )
        };
        pdf.stream(content_id, &content.finish());
    }

    pdf.finish()
}

fn current_date() -> String {
    let now = OffsetDateTime::now_utc().date();
    format!(
        "{:04}-{:02}-{:02}",
        now.year(),
        now.month() as u8,
        now.day()
    )
}

fn render_schematic(
    document: &Document,
    bom: &rackcraft_engine::Bom,
    title: &str,
    date: &str,
    page_count: usize,
) -> Content {
    let mut content = Content::new();
    draw_text(
        &mut content,
        FONT_BOLD,
        17.0,
        MARGIN,
        PAGE_H - 34.0,
        Color::rgb(24, 28, 36),
        &truncate(title, 70),
    );
    let subtitle = format!(
        "{} · {} items · {} links · {} part numbers",
        date,
        document.items.len(),
        document.connectors.len(),
        bom.distinct_parts()
    );
    draw_text(
        &mut content,
        FONT,
        8.5,
        MARGIN,
        PAGE_H - 48.0,
        Color::rgb(105, 113, 128),
        &subtitle,
    );
    let brand = "RACKCRAFT";
    draw_text(
        &mut content,
        FONT_BOLD,
        8.0,
        PAGE_W - MARGIN - text_width(brand, 8.0, true),
        PAGE_H - 34.0,
        Color::rgb(232, 87, 63),
        brand,
    );

    let drawing_left = MARGIN;
    let drawing_right = PAGE_W - MARGIN;
    let drawing_bottom = 82.0;
    let drawing_top = PAGE_H - 70.0;
    let drawing_width = drawing_right - drawing_left;
    let drawing_height = drawing_top - drawing_bottom;
    let (min_x, min_y, max_x, max_y) = document_bounds(document);
    let doc_w = (max_x - min_x).max(1.0) as f32;
    let doc_h = (max_y - min_y).max(1.0) as f32;
    let scale = ((drawing_width - 20.0) / doc_w)
        .min((drawing_height - 20.0) / doc_h)
        .clamp(0.02, 4.0);
    let fitted_w = doc_w * scale;
    let fitted_h = doc_h * scale;
    let offset_x = drawing_left + (drawing_width - fitted_w) * 0.5;
    let offset_y = drawing_top - (drawing_height - fitted_h) * 0.5;
    let map = |point: Point| -> (f32, f32) {
        (
            offset_x + (point.x as f32 - min_x as f32) * scale,
            offset_y - (point.y as f32 - min_y as f32) * scale,
        )
    };

    // Paper card for the drawing area.
    content.set_fill_rgb(0.985, 0.988, 0.994);
    content.set_stroke_rgb(0.78, 0.80, 0.84);
    content.set_line_width(0.7);
    content.rect(drawing_left, drawing_bottom, drawing_width, drawing_height);
    content.fill_nonzero_and_stroke();

    for item in &document.items {
        for path in &item.graphics {
            draw_path(&mut content, path, item.position, item.size, scale, &map);
        }
    }

    for connector in &document.connectors {
        draw_connector(&mut content, document, connector, &map);
    }

    for item in &document.items {
        draw_item_label(&mut content, item, &map);
    }

    draw_cartouche(
        &mut content,
        title,
        date,
        document.items.len(),
        bom.total_quantity,
        bom.distinct_parts(),
    );
    draw_page_number(&mut content, 1, page_count);
    content
}

fn render_inventory(
    document: &Document,
    bom: &rackcraft_engine::Bom,
    title: &str,
    date: &str,
    page_index: usize,
    inventory_pages: usize,
    page_count: usize,
) -> Content {
    let mut content = Content::new();
    draw_text(
        &mut content,
        FONT_BOLD,
        15.0,
        MARGIN,
        PAGE_H - 36.0,
        Color::rgb(24, 28, 36),
        &format!("Inventory — {}", truncate(title, 60)),
    );
    draw_text(
        &mut content,
        FONT,
        8.0,
        MARGIN,
        PAGE_H - 50.0,
        Color::rgb(105, 113, 128),
        &format!(
            "Project · {date} · page {} of {inventory_pages}",
            page_index + 1
        ),
    );

    let table_top = PAGE_H - 82.0;
    let table_bottom = 70.0;
    let header_h = 22.0;
    let row_h = (table_top - table_bottom - header_h) / INVENTORY_ROWS as f32;
    let col_part = MARGIN + 6.0;
    let col_desc = MARGIN + 190.0;
    let col_qty = PAGE_W - MARGIN - 8.0;

    content.set_fill_rgb(0.14, 0.17, 0.22);
    content.rect(
        MARGIN,
        table_top - header_h,
        PAGE_W - MARGIN * 2.0,
        header_h,
    );
    content.fill_nonzero();
    draw_text(
        &mut content,
        FONT_BOLD,
        8.0,
        col_part,
        table_top - 14.0,
        Color::rgb(255, 255, 255),
        "PART NUMBER",
    );
    draw_text(
        &mut content,
        FONT_BOLD,
        8.0,
        col_desc,
        table_top - 14.0,
        Color::rgb(255, 255, 255),
        "DESCRIPTION",
    );
    draw_text(
        &mut content,
        FONT_BOLD,
        8.0,
        col_qty - 46.0,
        table_top - 14.0,
        Color::rgb(255, 255, 255),
        "QUANTITY",
    );

    let start = page_index * INVENTORY_ROWS;
    let end = (start + INVENTORY_ROWS).min(bom.lines.len());
    let mut y = table_top - header_h;
    for (row_index, line) in bom.lines[start..end].iter().enumerate() {
        if row_index % 2 == 1 {
            content.set_fill_rgb(0.965, 0.97, 0.98);
            content.rect(MARGIN, y - row_h, PAGE_W - MARGIN * 2.0, row_h);
            content.fill_nonzero();
        }
        let text_y = y - row_h + 6.0;
        let part = if line.part_number.is_empty() {
            "—"
        } else {
            &line.part_number
        };
        draw_text(
            &mut content,
            FONT,
            8.0,
            col_part,
            text_y,
            Color::rgb(24, 28, 36),
            &truncate(part, 30),
        );
        let description = if line.manufacturer.is_empty() {
            line.description.clone()
        } else {
            format!("{} — {}", line.manufacturer, line.description)
        };
        draw_text(
            &mut content,
            FONT,
            8.0,
            col_desc,
            text_y,
            Color::rgb(64, 72, 86),
            &truncate(&description, 82),
        );
        let quantity = line.quantity.to_string();
        let qty_width = text_width(&quantity, 8.0, true);
        draw_text(
            &mut content,
            FONT_BOLD,
            8.0,
            col_qty - qty_width,
            text_y,
            Color::rgb(24, 28, 36),
            &quantity,
        );
        content.set_stroke_rgb(0.84, 0.86, 0.89);
        content.set_line_width(0.35);
        content.move_to(MARGIN, y - row_h);
        content.line_to(PAGE_W - MARGIN, y - row_h);
        content.stroke();
        y -= row_h;
    }

    content.set_stroke_rgb(0.35, 0.39, 0.46);
    content.set_line_width(0.7);
    content.rect(
        MARGIN,
        table_top - header_h - row_h * (end - start) as f32,
        PAGE_W - MARGIN * 2.0,
        header_h + row_h * (end - start) as f32,
    );
    content.stroke();

    if page_index + 1 == inventory_pages {
        draw_text(
            &mut content,
            FONT_BOLD,
            9.0,
            col_desc,
            46.0,
            Color::rgb(24, 28, 36),
            &format!(
                "Total quantity: {} · {} distinct part numbers",
                bom.total_quantity,
                bom.distinct_parts()
            ),
        );
    }
    draw_cartouche_footer(&mut content, document, title, date);
    draw_page_number(&mut content, page_index + 2, page_count);
    content
}

fn draw_cartouche(
    content: &mut Content,
    title: &str,
    date: &str,
    item_count: usize,
    total_quantity: u32,
    parts: usize,
) {
    let x = MARGIN;
    let y = 26.0;
    let w = PAGE_W - MARGIN * 2.0;
    let h = 48.0;
    content.set_fill_rgb(0.985, 0.988, 0.994);
    content.set_stroke_rgb(0.65, 0.68, 0.73);
    content.set_line_width(0.8);
    content.rect(x, y, w, h);
    content.fill_nonzero_and_stroke();
    content.set_fill_rgb(0.14, 0.17, 0.22);
    content.rect(x, y + h - 13.0, w, 13.0);
    content.fill_nonzero();
    draw_text(
        content,
        FONT_BOLD,
        8.0,
        x + 8.0,
        y + h - 9.0,
        Color::rgb(255, 255, 255),
        "PROJECT CARTOUCHE",
    );
    draw_text(
        content,
        FONT_BOLD,
        11.0,
        x + 8.0,
        y + 20.0,
        Color::rgb(24, 28, 36),
        &truncate(title, 60),
    );
    draw_text(
        content,
        FONT,
        7.5,
        x + 8.0,
        y + 9.0,
        Color::rgb(105, 113, 128),
        &format!(
            "RackCraft · {item_count} items · {total_quantity} inventory units · {parts} part numbers · {date}"
        ),
    );
    let license = "MIT OR Apache-2.0";
    draw_text(
        content,
        FONT,
        6.5,
        x + w - 8.0 - text_width(license, 6.5, false),
        y + 9.0,
        Color::rgb(140, 148, 162),
        license,
    );
}

fn draw_cartouche_footer(content: &mut Content, document: &Document, title: &str, date: &str) {
    draw_text(
        content,
        FONT,
        7.0,
        MARGIN,
        34.0,
        Color::rgb(105, 113, 128),
        &format!(
            "{title} · {} items · {} links · {date}",
            document.items.len(),
            document.connectors.len()
        ),
    );
}

fn draw_page_number(content: &mut Content, page: usize, total: usize) {
    let label = format!("Page {page} / {total}");
    draw_text(
        content,
        FONT,
        7.0,
        PAGE_W - MARGIN - text_width(&label, 7.0, false),
        12.0,
        Color::rgb(140, 148, 162),
        &label,
    );
}

fn draw_path(
    content: &mut Content,
    path: &DrawPath,
    position: Point,
    size: rackcraft_engine::Size,
    scale: f32,
    map: &impl Fn(Point) -> (f32, f32),
) {
    let mut has_path = false;
    let mut current: Option<(f32, f32)> = None;
    for command in &path.commands {
        match *command {
            PathCommand::MoveTo { to } => {
                let p = map(Point::new(
                    position.x + to.x * size.width,
                    position.y + to.y * size.height,
                ));
                content.move_to(p.0, p.1);
                current = Some(p);
                has_path = true;
            }
            PathCommand::LineTo { to } => {
                let p = map(Point::new(
                    position.x + to.x * size.width,
                    position.y + to.y * size.height,
                ));
                content.line_to(p.0, p.1);
                current = Some(p);
                has_path = true;
            }
            PathCommand::QuadTo { ctrl, to } => {
                let ctrl = map(Point::new(
                    position.x + ctrl.x * size.width,
                    position.y + ctrl.y * size.height,
                ));
                let end = map(Point::new(
                    position.x + to.x * size.width,
                    position.y + to.y * size.height,
                ));
                if let Some(start) = current {
                    let c1 = (
                        start.0 + (ctrl.0 - start.0) * 2.0 / 3.0,
                        start.1 + (ctrl.1 - start.1) * 2.0 / 3.0,
                    );
                    let c2 = (
                        end.0 + (ctrl.0 - end.0) * 2.0 / 3.0,
                        end.1 + (ctrl.1 - end.1) * 2.0 / 3.0,
                    );
                    content.cubic_to(c1.0, c1.1, c2.0, c2.1, end.0, end.1);
                } else {
                    content.move_to(end.0, end.1);
                }
                current = Some(end);
                has_path = true;
            }
            PathCommand::CubicTo { c1, c2, to } => {
                let c1 = map(Point::new(
                    position.x + c1.x * size.width,
                    position.y + c1.y * size.height,
                ));
                let c2 = map(Point::new(
                    position.x + c2.x * size.width,
                    position.y + c2.y * size.height,
                ));
                let end = map(Point::new(
                    position.x + to.x * size.width,
                    position.y + to.y * size.height,
                ));
                content.cubic_to(c1.0, c1.1, c2.0, c2.1, end.0, end.1);
                current = Some(end);
                has_path = true;
            }
            PathCommand::Close => {
                content.close_path();
                has_path = true;
            }
        }
    }
    if !has_path {
        return;
    }
    if let Some(fill) = path.fill {
        content.set_fill_rgb(
            f32::from(fill.r) / 255.0,
            f32::from(fill.g) / 255.0,
            f32::from(fill.b) / 255.0,
        );
    }
    if let Some(stroke) = path.stroke {
        content.set_stroke_rgb(
            f32::from(stroke.r) / 255.0,
            f32::from(stroke.g) / 255.0,
            f32::from(stroke.b) / 255.0,
        );
        content.set_line_width((path.stroke_width_mm as f32 * scale).clamp(0.25, 3.0));
    }
    if path.fill.is_some() && path.stroke.is_some() {
        content.fill_nonzero_and_stroke();
    } else if path.fill.is_some() {
        content.fill_nonzero();
    } else if path.stroke.is_some() {
        content.stroke();
    }
}

fn draw_connector(
    content: &mut Content,
    document: &Document,
    connector: &Connector,
    map: &impl Fn(Point) -> (f32, f32),
) {
    let Some(points) = document.connector_path(connector) else {
        return;
    };
    if points.len() < 2 {
        return;
    }
    let colour = connector.color;
    content.set_stroke_rgb(
        f32::from(colour.r) / 255.0,
        f32::from(colour.g) / 255.0,
        f32::from(colour.b) / 255.0,
    );
    content.set_fill_rgb(
        f32::from(colour.r) / 255.0,
        f32::from(colour.g) / 255.0,
        f32::from(colour.b) / 255.0,
    );
    content.set_line_width(1.2);
    if connector.line_style == LineStyle::Dashed {
        content.set_dash_pattern([4.0, 2.5], 0.0);
    } else {
        content.set_dash_pattern([], 0.0);
    }
    let first = map(points[0]);
    content.move_to(first.0, first.1);
    for point in &points[1..] {
        let p = map(*point);
        content.line_to(p.0, p.1);
    }
    content.stroke();
    content.set_dash_pattern([], 0.0);

    if connector.arrow != Arrow::None {
        let last = map(*points.last().unwrap_or(&points[0]));
        let prev = map(points[points.len() - 2]);
        draw_arrow(content, prev, last);
    }
    if connector.arrow == Arrow::Double {
        let first = map(points[0]);
        let second = map(points[1]);
        draw_arrow(content, second, first);
    }

    if !connector.label.is_empty() {
        let fraction = 0.28 + 0.14 * (connector.id % 4) as f64;
        let midpoint = polyline_point_at(&points, fraction);
        let p = map(midpoint);
        let text = truncate(&connector.label, 28);
        let width = text_width(&text, 6.0, false) + 6.0;
        content.set_fill_rgb(1.0, 1.0, 1.0);
        content.rect(p.0 - width * 0.5, p.1 + 1.0, width, 9.0);
        content.fill_nonzero();
        draw_text_centered(
            content,
            FONT,
            6.0,
            p.0,
            p.1 + 3.5,
            Color::rgb(48, 56, 68),
            &text,
        );
    }
}

fn draw_arrow(content: &mut Content, from: (f32, f32), tip: (f32, f32)) {
    let dx = tip.0 - from.0;
    let dy = tip.1 - from.1;
    let len = (dx * dx + dy * dy).sqrt().max(1e-6);
    let ux = dx / len;
    let uy = dy / len;
    let size = 5.5;
    let wing = 2.7;
    content.move_to(tip.0, tip.1);
    content.line_to(tip.0 - ux * size - uy * wing, tip.1 - uy * size + ux * wing);
    content.line_to(tip.0 - ux * size + uy * wing, tip.1 - uy * size - ux * wing);
    content.close_path();
    content.fill_nonzero();
}

fn polyline_point_at(points: &[Point], fraction: f64) -> Point {
    let total: f64 = points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum();
    if total <= 0.0 {
        return points[0];
    }
    let mut remaining = total * fraction.clamp(0.0, 1.0);
    for pair in points.windows(2) {
        let length = pair[0].distance(pair[1]);
        if remaining <= length && length > 0.0 {
            return pair[0].lerp(pair[1], remaining / length);
        }
        remaining -= length;
    }
    *points.last().unwrap_or(&points[0])
}

fn draw_item_label(
    content: &mut Content,
    item: &rackcraft_engine::Item,
    map: &impl Fn(Point) -> (f32, f32),
) {
    let top_left = map(item.position);
    let bottom_right = map(Point::new(
        item.position.x + item.size.width,
        item.position.y + item.size.height,
    ));
    let x = top_left.0;
    let y = bottom_right.1 - 8.0;
    if (bottom_right.0 - top_left.0).abs() < 16.0 {
        return;
    }
    let label = truncate(&item.name, 36);
    draw_text(content, FONT, 6.0, x, y, Color::rgb(80, 88, 102), &label);
}

fn document_bounds(document: &Document) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for item in &document.items {
        let bounds = item.bounds();
        min_x = min_x.min(bounds.left());
        min_y = min_y.min(bounds.top());
        max_x = max_x.max(bounds.right());
        max_y = max_y.max(bounds.bottom());
    }
    for connector in &document.connectors {
        if let Some(points) = document.connector_path(connector) {
            for point in points {
                min_x = min_x.min(point.x);
                min_y = min_y.min(point.y);
                max_x = max_x.max(point.x);
                max_y = max_y.max(point.y);
            }
        }
    }
    if !min_x.is_finite() {
        return (
            0.0,
            0.0,
            document.canvas.width_mm,
            document.canvas.height_mm,
        );
    }
    let pad = 8.0;
    (min_x - pad, min_y - pad, max_x + pad, max_y + pad)
}

fn draw_text(
    content: &mut Content,
    font: Name<'_>,
    size: f32,
    x: f32,
    y: f32,
    colour: Color,
    text: &str,
) {
    let bytes = win_ansi(text);
    content.set_fill_rgb(
        f32::from(colour.r) / 255.0,
        f32::from(colour.g) / 255.0,
        f32::from(colour.b) / 255.0,
    );
    content.begin_text();
    content.set_font(font, size);
    content.set_text_matrix([1.0, 0.0, 0.0, 1.0, x, y]);
    content.show(Str(&bytes));
    content.end_text();
}

fn draw_text_centered(
    content: &mut Content,
    font: Name<'_>,
    size: f32,
    x: f32,
    y: f32,
    colour: Color,
    text: &str,
) {
    let width = text_width(text, size, false);
    draw_text(content, font, size, x - width * 0.5, y, colour, text);
}

fn text_width(text: &str, size: f32, bold: bool) -> f32 {
    let factor = if bold { 0.56 } else { 0.50 };
    text.chars().count() as f32 * size * factor
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        format!(
            "{}…",
            text.chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
        )
    } else {
        text.to_owned()
    }
}

fn win_ansi(text: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(text.len());
    for character in text.chars() {
        let code = match character {
            '\u{00A0}'..='\u{00FF}' => character as u32,
            '\u{2013}' => 0x96,
            '\u{2014}' => 0x97,
            '\u{2018}' => 0x91,
            '\u{2019}' => 0x92,
            '\u{201C}' => 0x93,
            '\u{201D}' => 0x94,
            '\u{2022}' => 0x95,
            '\u{2026}' => 0x85,
            '\u{20AC}' => 0x80,
            '\u{2122}' => 0x99,
            '\n' | '\r' | '\t' => 0x20,
            character if character.is_ascii() => character as u32,
            _ => u32::from(b'?'),
        };
        bytes.push(u8::try_from(code).unwrap_or(b'?'));
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_pdf_is_valid_and_contains_both_pages() {
        let document = rackcraft_engine::sample::document();
        let bytes = pdf_bytes(&document);
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.windows(5).any(|window| window == b"%%EOF"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("/Type /Page"));
        assert!(text.contains("PART NUMBER"));
        assert!(text.contains("DESCRIPTION"));
        assert!(text.contains("QUANTITY"));
        assert!(text.contains("PROJECT CARTOUCHE"));
        assert!(text.contains("RC-PP-24C6"));
    }

    #[test]
    fn large_inventory_gets_continuation_pages() {
        let mut document = rackcraft_engine::Document::new("Large");
        for index in 0..30 {
            let mut item = rackcraft_engine::Item::new(
                rackcraft_engine::StencilRef::default(),
                format!("Part {index}"),
                rackcraft_engine::Point::default(),
                rackcraft_engine::Size::new(10.0, 10.0),
            );
            item.set_shape_value("Part Number", format!("RC-{index:03}"));
            item.set_shape_value("Product Description", format!("Unique part {index}"));
            document.add_item(item);
        }
        let bytes = pdf_bytes(&document);
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("/Count 3"));
        assert!(text.contains("Page 3 / 3"));
        assert!(text.contains("RC-029"));
    }

    #[test]
    fn file_export_writes_a_pdf() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("rackcraft.pdf");
        export_pdf(&rackcraft_engine::sample::document(), &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.len() > 4_000);
    }

    #[test]
    fn win_ansi_replaces_unsupported_glyphs() {
        let bytes = win_ansi("Cat6A — é");
        assert!(bytes.contains(&0x97));
        assert!(bytes.contains(&0xE9));
    }
}
