//! Pure-Rust native fallback for legacy binary `.vss` (OLE/CFB) stencils.
//!
//! The Visio binary format is proprietary. Without libvisio/soffice, RackCraft
//! cannot recover master geometry, but it can still inspect the OLE container:
//! stream names, embedded PNG/ICO previews and printable metadata strings. That
//! information becomes a placeholder stencil so an import never fails silently.

use std::io::{Cursor, Read};

use rackcraft_engine::{Color, DrawPath, PathCommand, Point, ShapeField};
use serde::Serialize;

use crate::error::{IoError, Result};
use crate::model::{IconImage, Master, StencilLibrary, infer_tags};

const MAX_STREAM_BYTES: usize = 8 * 1024 * 1024;
const MAX_STRINGS: usize = 80;

/// Information recovered from a legacy compound file.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LegacyInspection {
    /// All OLE stream paths.
    pub streams: Vec<String>,
    /// Printable metadata strings found in the streams.
    pub text: Vec<String>,
    /// First decoded preview icon, if one was present.
    pub icon: Option<IconImage>,
}

/// Inspect a legacy `.vss` compound file without external tools.
pub fn inspect(bytes: &[u8]) -> Result<LegacyInspection> {
    let mut compound = cfb::CompoundFile::open(Cursor::new(bytes.to_vec()))
        .map_err(|err| IoError::Conversion(format!("invalid OLE/CFB document: {err}")))?;
    let entries: Vec<_> = compound
        .walk()
        .map(|entry| (entry.path().to_path_buf(), entry.is_stream()))
        .collect();
    let mut inspection = LegacyInspection::default();
    for (path, is_stream) in entries {
        if !is_stream {
            continue;
        }
        let name = path.strip_prefix("/").map_or_else(
            |_| path.display().to_string(),
            |path| path.display().to_string(),
        );
        inspection.streams.push(name);
        let Ok(mut stream) = compound.open_stream(&path) else {
            continue;
        };
        let mut data = Vec::new();
        if stream
            .by_ref()
            .take((MAX_STREAM_BYTES + 1) as u64)
            .read_to_end(&mut data)
            .is_err()
            || data.len() > MAX_STREAM_BYTES
        {
            continue;
        }
        for text in extract_strings(&data) {
            if !inspection.text.contains(&text) && inspection.text.len() < MAX_STRINGS {
                inspection.text.push(text);
            }
        }
        if inspection.icon.is_none()
            && let Some(icon) = find_icon(&data)
        {
            inspection.icon = Some(icon);
        }
    }
    Ok(inspection)
}

/// Build a usable placeholder library from the OLE container.
pub fn load_native_fallback(
    bytes: &[u8],
    display_name: &str,
    source: Option<String>,
) -> Result<StencilLibrary> {
    let inspection = inspect(bytes)?;
    let category = "Legacy .vss (native)".to_owned();
    let name = display_name.to_owned();
    let tags = infer_tags(&name, &category);
    let streams_preview = inspection.streams.join(", ");
    let text_preview = inspection.text.join(" | ");
    let mut shape_data = vec![
        ShapeField {
            label: "Product Description".into(),
            value: name.clone(),
            prompt: String::new(),
        },
        ShapeField {
            label: "Source".into(),
            value: ".vss OLE compound file (native fallback)".into(),
            prompt: String::new(),
        },
    ];
    if !streams_preview.is_empty() {
        shape_data.push(ShapeField {
            label: "Legacy streams".into(),
            value: truncate(&streams_preview, 400),
            prompt: String::new(),
        });
    }
    if !text_preview.is_empty() {
        shape_data.push(ShapeField {
            label: "Legacy metadata".into(),
            value: truncate(&text_preview, 600),
            prompt: String::new(),
        });
    }
    let mut keywords = format!("{name} {category} vss legacy").to_lowercase();
    for text in &inspection.text {
        keywords.push(' ');
        keywords.push_str(&text.to_lowercase());
    }
    let master = Master {
        id: "legacy-native-1".into(),
        name,
        category: category.clone(),
        width_mm: 100.0,
        height_mm: 70.0,
        graphics: placeholder_graphics(),
        connection_points: vec![
            rackcraft_engine::ConnectionPoint::new("top", 0.5, 0.0).named("Top"),
            rackcraft_engine::ConnectionPoint::new("right", 1.0, 0.5).named("Right"),
            rackcraft_engine::ConnectionPoint::new("bottom", 0.5, 1.0).named("Bottom"),
            rackcraft_engine::ConnectionPoint::new("left", 0.0, 0.5).named("Left"),
        ],
        shape_data,
        tags,
        mount: None,
        keywords,
        icon: inspection.icon,
    };
    let library = StencilLibrary {
        name: display_name.to_owned(),
        source,
        masters: vec![master],
        warnings: vec![
            "Native .vss fallback: only container metadata and preview icons could be recovered. \
             Install libvisio-tools for vector shapes (Arch/CachyOS: `sudo pacman -S libvisio`; \
             Fedora: `sudo dnf install libvisio-tools`; Debian/Ubuntu: `sudo apt install libvisio-tools`) \
             or LibreOffice (`soffice --headless --convert-to vssx`)."
                .to_owned(),
            format!(
                "OLE streams inspected: {}",
                truncate(&streams_preview, 500)
            ),
        ],
    };
    Ok(library)
}

fn placeholder_graphics() -> Vec<DrawPath> {
    let mut paths = Vec::new();
    let outline = vec![
        PathCommand::MoveTo {
            to: Point::new(0.06, 0.05),
        },
        PathCommand::LineTo {
            to: Point::new(0.94, 0.05),
        },
        PathCommand::LineTo {
            to: Point::new(0.94, 0.95),
        },
        PathCommand::LineTo {
            to: Point::new(0.06, 0.95),
        },
        PathCommand::Close,
    ];
    paths.push(
        DrawPath::filled(outline, Color::rgb(245, 247, 251))
            .with_stroke(Color::rgb(94, 106, 128), 0.4),
    );
    let band = vec![
        PathCommand::MoveTo {
            to: Point::new(0.06, 0.05),
        },
        PathCommand::LineTo {
            to: Point::new(0.94, 0.05),
        },
        PathCommand::LineTo {
            to: Point::new(0.94, 0.20),
        },
        PathCommand::LineTo {
            to: Point::new(0.06, 0.20),
        },
        PathCommand::Close,
    ];
    paths.push(DrawPath::filled(band, Color::rgb(232, 87, 63)));
    for row in 0..3 {
        let y = 0.32 + row as f64 * 0.17;
        let line = vec![
            PathCommand::MoveTo {
                to: Point::new(0.18, y),
            },
            PathCommand::LineTo {
                to: Point::new(0.80, y),
            },
        ];
        paths.push(DrawPath {
            commands: line,
            fill: None,
            stroke: Some(Color::rgb(128, 138, 154)),
            stroke_width_mm: 0.4,
        });
    }
    paths
}

fn extract_strings(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    for offset in 0..2 {
        let mut current = String::new();
        let mut index = offset;
        while index + 1 < data.len() {
            let value = u16::from_le_bytes([data[index], data[index + 1]]);
            index += 2;
            let character = char::from_u32(u32::from(value));
            match character {
                Some(character)
                    if character == ' '
                        || character.is_ascii_graphic()
                        || ('\u{00A0}'..='\u{00FF}').contains(&character) =>
                {
                    current.push(character);
                }
                _ => {
                    push_metadata_string(&mut current, &mut strings);
                }
            }
        }
        push_metadata_string(&mut current, &mut strings);
    }

    let mut ascii = String::new();
    for &byte in data {
        if byte.is_ascii_graphic() || byte == b' ' {
            ascii.push(char::from(byte));
        } else {
            push_metadata_string(&mut ascii, &mut strings);
        }
    }
    push_metadata_string(&mut ascii, &mut strings);
    strings
}

fn push_metadata_string(candidate: &mut String, output: &mut Vec<String>) {
    let trimmed = candidate.trim();
    if (4..=160).contains(&trimmed.chars().count())
        && trimmed.chars().any(char::is_alphabetic)
        && !looks_binary(trimmed)
        && !output.iter().any(|existing| existing == trimmed)
        && output.len() < MAX_STRINGS
    {
        output.push(trimmed.to_owned());
    }
    candidate.clear();
}

fn looks_binary(text: &str) -> bool {
    let total = text.chars().count().max(1);
    let control = text
        .chars()
        .filter(|character| character.is_control() && *character != ' ')
        .count();
    control * 8 > total
}

fn find_icon(data: &[u8]) -> Option<IconImage> {
    if let Some(index) = data
        .windows(8)
        .position(|window| window == b"\x89PNG\r\n\x1a\n")
    {
        return decode_png(&data[index..]);
    }
    if let Some(index) = data
        .windows(8)
        .position(|window| window == [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])
    {
        return crate::vssx::parse_ico(&data[index..]);
    }
    let ico = [0_u8, 0, 1, 0];
    data.windows(4)
        .position(|window| window == ico)
        .and_then(|index| crate::vssx::parse_ico(&data[index..]))
}

fn decode_png(data: &[u8]) -> Option<IconImage> {
    let decoder = png::Decoder::new(Cursor::new(data));
    let mut reader = decoder.read_info().ok()?;
    if reader.info().width == 0
        || reader.info().height == 0
        || reader.info().width > 512
        || reader.info().height > 512
    {
        return None;
    }
    let mut buffer = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut buffer).ok()?;
    let bytes = &buffer[..frame.buffer_size()];
    let width = frame.width as usize;
    let height = frame.height as usize;
    let mut rgba = Vec::with_capacity(width * height * 4);
    match frame.color_type {
        png::ColorType::Rgba => rgba.extend_from_slice(bytes),
        png::ColorType::Rgb => {
            let (pixels, _) = bytes.as_chunks::<3>();
            for pixel in pixels {
                rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
            }
        }
        png::ColorType::Grayscale => {
            for &value in bytes {
                rgba.extend_from_slice(&[value, value, value, 255]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            let (pixels, _) = bytes.as_chunks::<2>();
            for pixel in pixels {
                rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]]);
            }
        }
        png::ColorType::Indexed => return None,
    }
    Some(IconImage {
        width: u32::try_from(width).ok()?,
        height: u32::try_from(height).ok()?,
        rgba,
    })
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_owned()
    } else {
        format!(
            "{}…",
            text.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn compound_with_streams() -> Vec<u8> {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("legacy.vss");
        {
            let mut compound = cfb::create(&path).unwrap();
            let mut stream = compound.create_stream("/VisioDocument").unwrap();
            for value in "RackCraft Legacy Rack".encode_utf16() {
                stream.write_all(&value.to_le_bytes()).unwrap();
            }
            stream.write_all(&[0, 0]).unwrap();
            let mut summary = compound.create_stream("/\u{5}SummaryInformation").unwrap();
            summary
                .write_all(b"Manufactured by Legacy Networks\0")
                .unwrap();
        }
        std::fs::read(path).unwrap()
    }

    #[test]
    fn native_fallback_extracts_streams_and_text() {
        let bytes = compound_with_streams();
        let inspection = inspect(&bytes).unwrap();
        assert!(
            inspection
                .streams
                .iter()
                .any(|name| name.contains("VisioDocument"))
        );
        assert!(
            inspection
                .text
                .iter()
                .any(|text| text.contains("RackCraft Legacy"))
        );
        let library = load_native_fallback(&bytes, "Legacy Rack", None).unwrap();
        assert_eq!(library.masters.len(), 1);
        assert!(!library.masters[0].graphics.is_empty());
        assert!(!library.warnings.is_empty());
        assert!(library.warnings[0].contains("pacman"));
    }
}
