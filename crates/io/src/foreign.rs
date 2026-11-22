//! Lazy conversion of Visio `ForeignData` EMF/WMF artwork to vector paths.

use std::io::{Cursor, Read};

use rackcraft_engine::DrawPath;
use zip::ZipArchive;

use crate::error::{IoError, Result};

const MAX_VECTOR_PART_BYTES: usize = 32 * 1024 * 1024;

/// Convert every EMF/WMF entry referenced by a master into normalized paths.
///
/// The resulting paths are relative to each image's own view box. Vendors such
/// as Cisco embed the switch faceplate artwork this way instead of Visio
/// geometry, so this is what turns a plain 1U rectangle into a real switch.
pub fn vector_graphics(package: &[u8], entries: &[String]) -> Result<Vec<DrawPath>> {
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    let mut archive = ZipArchive::new(Cursor::new(package))?;
    let mut graphics = Vec::new();
    let mut seen = Vec::new();
    for entry in entries {
        if seen.iter().any(|seen| seen == entry) {
            continue;
        }
        seen.push(entry.clone());
        let mut file = match archive.by_name(entry) {
            Ok(file) => file,
            Err(_) => continue,
        };
        let mut data = Vec::new();
        file.by_ref()
            .take((MAX_VECTOR_PART_BYTES + 1) as u64)
            .read_to_end(&mut data)
            .map_err(IoError::Io)?;
        if data.len() > MAX_VECTOR_PART_BYTES {
            return Err(IoError::Conversion(format!(
                "ForeignData part {entry} exceeds 32 MiB"
            )));
        }
        let extension = std::path::Path::new(entry)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default()
            .to_ascii_lowercase();
        if extension != "emf" && extension != "wmf" {
            continue;
        }
        let svg = metafile::to_svg(&data, metafile::RenderOptions::default()).map_err(|err| {
            IoError::Conversion(format!("could not render {entry} metafile: {err}"))
        })?;
        graphics.extend(crate::legacy_svg::svg_graphics(svg.svg.as_bytes())?);
    }
    Ok(graphics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_entry_list_returns_nothing() {
        assert!(vector_graphics(b"not a zip", &[]).unwrap().is_empty());
    }
}
