//! BOM export to formatted Excel workbooks and UTF-8 CSV.

use std::path::Path;

use rackcraft_engine::Bom;
use rust_xlsxwriter::{Color as XlsxColor, Format, Workbook};

use crate::error::{IoError, Result};

/// Column headers shared by CSV and XLSX exports.
pub const BOM_HEADERS: [&str; 6] = [
    "Manufacturer",
    "Part Number",
    "Product Description",
    "Quantity",
    "Locations",
    "Item IDs",
];

/// Write a formatted `.xlsx` inventory workbook.
pub fn export_bom_xlsx(bom: &Bom, path: impl AsRef<Path>, title: &str) -> Result<()> {
    let bytes = bom_xlsx_bytes(bom, title)?;
    std::fs::write(path.as_ref(), bytes)?;
    Ok(())
}

/// Build an `.xlsx` workbook in memory (used by tests and future web builds).
pub fn bom_xlsx_bytes(bom: &Bom, title: &str) -> Result<Vec<u8>> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Inventory")?;

    let title_format = Format::new()
        .set_bold()
        .set_font_size(16)
        .set_font_color(XlsxColor::RGB(0xF5F7FA))
        .set_background_color(XlsxColor::RGB(0x1F2532))
        .set_align(rust_xlsxwriter::FormatAlign::Left)
        .set_align(rust_xlsxwriter::FormatAlign::VerticalCenter);
    let subtitle_format = Format::new()
        .set_font_size(10)
        .set_font_color(XlsxColor::RGB(0x6B7485));
    let header_format = Format::new()
        .set_bold()
        .set_font_color(XlsxColor::RGB(0xF5F7FA))
        .set_background_color(XlsxColor::RGB(0xE8573F))
        .set_align(rust_xlsxwriter::FormatAlign::Center)
        .set_align(rust_xlsxwriter::FormatAlign::VerticalCenter);
    let number_format = Format::new().set_align(rust_xlsxwriter::FormatAlign::Center);
    let total_format = Format::new()
        .set_bold()
        .set_border_top(rust_xlsxwriter::FormatBorder::Thin)
        .set_background_color(XlsxColor::RGB(0xF0F2F5));

    worksheet.merge_range(0, 0, 0, 5, title, &title_format)?;
    let subtitle = format!(
        "RackCraft inventory · {} placed items · {} distinct part numbers",
        bom.total_quantity,
        bom.distinct_parts()
    );
    worksheet.merge_range(1, 0, 1, 5, &subtitle, &subtitle_format)?;
    worksheet.set_row_height(0, 26.0)?;

    for (column, header) in BOM_HEADERS.iter().enumerate() {
        let column = u16::try_from(column)
            .map_err(|_| IoError::ExportTarget(Path::new(".").to_path_buf()))?;
        worksheet.write_string_with_format(3, column, *header, &header_format)?;
    }

    for (row_index, line) in bom.lines.iter().enumerate() {
        let row = u32::try_from(row_index + 4)
            .map_err(|_| IoError::ExportTarget(Path::new(".").to_path_buf()))?;
        worksheet.write_string(row, 0, &line.manufacturer)?;
        worksheet.write_string(row, 1, &line.part_number)?;
        worksheet.write_string(row, 2, &line.description)?;
        worksheet.write_number_with_format(row, 3, f64::from(line.quantity), &number_format)?;
        let locations = line.locations.join("; ");
        worksheet.write_string(row, 4, &locations)?;
        let ids = line
            .item_ids
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        worksheet.write_string(row, 5, &ids)?;
    }

    let total_row = u32::try_from(bom.lines.len() + 4)
        .map_err(|_| IoError::ExportTarget(Path::new(".").to_path_buf()))?;
    worksheet.write_string_with_format(total_row, 0, "Total", &total_format)?;
    worksheet.write_blank(total_row, 1, &total_format)?;
    worksheet.write_blank(total_row, 2, &total_format)?;
    worksheet.write_number_with_format(
        total_row,
        3,
        f64::from(bom.total_quantity),
        &total_format,
    )?;
    worksheet.write_blank(total_row, 4, &total_format)?;
    worksheet.write_blank(total_row, 5, &total_format)?;

    worksheet.set_column_width(0, 22.0)?;
    worksheet.set_column_width(1, 20.0)?;
    worksheet.set_column_width(2, 48.0)?;
    worksheet.set_column_width(3, 11.0)?;
    worksheet.set_column_width(4, 36.0)?;
    worksheet.set_column_width(5, 18.0)?;
    if bom.lines.is_empty() {
        worksheet.autofilter(3, 0, 3, 5)?;
    } else {
        let last = u32::try_from(bom.lines.len() + 3)
            .map_err(|_| IoError::ExportTarget(Path::new(".").to_path_buf()))?;
        worksheet.autofilter(3, 0, last, 5)?;
    }
    worksheet.set_freeze_panes(4, 0)?;

    Ok(workbook.save_to_buffer()?)
}

/// Write a UTF-8 CSV inventory.
pub fn export_bom_csv(bom: &Bom, path: impl AsRef<Path>) -> Result<()> {
    let bytes = bom_csv_bytes(bom)?;
    std::fs::write(path.as_ref(), bytes)?;
    Ok(())
}

/// Render the CSV in memory.
pub fn bom_csv_bytes(bom: &Bom) -> Result<Vec<u8>> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(Vec::new());
    writer.write_record(BOM_HEADERS)?;
    for line in &bom.lines {
        let quantity = line.quantity.to_string();
        let locations = line.locations.join("; ");
        let ids = line
            .item_ids
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        writer.write_record([
            line.manufacturer.as_str(),
            line.part_number.as_str(),
            line.description.as_str(),
            quantity.as_str(),
            locations.as_str(),
            ids.as_str(),
        ])?;
    }
    writer.flush()?;
    writer
        .into_inner()
        .map_err(|err| IoError::Conversion(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::io::Write;

    use rackcraft_engine::{Document, Item, Point, Size, StencilRef};

    fn bom() -> Bom {
        let mut doc = Document::new("Export test");
        for description in ["CAT6 panel, 24 ports", "Panel \"B\""] {
            let mut item = Item::new(
                StencilRef::default(),
                "Panel",
                Point::default(),
                Size::new(482.6, 44.45),
            );
            item.set_shape_value("Manufacturer", "RackCraft");
            item.set_shape_value("Part Number", "RC-PP-24");
            item.set_shape_value("Product Description", description);
            item.project.location = "Room A".into();
            doc.add_item(item);
        }
        doc.bom()
    }

    #[test]
    fn csv_round_trips_escaping() {
        let bytes = bom_csv_bytes(&bom()).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.starts_with("Manufacturer,Part Number"));
        assert!(text.contains("\"Panel \"\"B\"\"\""));
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(text.as_bytes());
        let rows: Vec<csv::StringRecord> = reader
            .records()
            .filter_map(std::result::Result::ok)
            .collect();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|row| &row[2] == "Panel \"B\""));
        assert!(rows.iter().all(|row| &row[3] == "1"));
    }

    #[test]
    fn xlsx_contains_inventory_data() {
        let bytes = bom_xlsx_bytes(&bom(), "RackCraft test").unwrap();
        assert!(bytes.starts_with(b"PK"));
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut all_xml = String::new();
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).unwrap();
            if file.name().ends_with(".xml") {
                let mut text = String::new();
                file.read_to_string(&mut text).unwrap();
                all_xml.push_str(&text);
            }
        }
        assert!(all_xml.contains("RC-PP-24"));
        assert!(all_xml.contains("RackCraft"));
        assert!(all_xml.contains("Total"));
    }

    #[test]
    fn files_are_written_to_disk() {
        let folder = tempfile::tempdir().unwrap();
        let xlsx = folder.path().join("inventory.xlsx");
        let csv = folder.path().join("inventory.csv");
        export_bom_xlsx(&bom(), &xlsx, "Inventory").unwrap();
        export_bom_csv(&bom(), &csv).unwrap();
        assert!(xlsx.metadata().unwrap().len() > 1000);
        let mut text = String::new();
        std::fs::File::open(csv)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        assert!(text.contains("RC-PP-24"));
        let mut file = std::fs::File::create(folder.path().join("discard")).unwrap();
        file.write_all(b"x").unwrap();
    }
}
