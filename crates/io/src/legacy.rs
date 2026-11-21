//! Legacy binary `.vss` detection and optional LibreOffice conversion.

use std::ffi::OsStr;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{IoError, Result};
use crate::model::{StencilFormat, StencilLibrary, detect_format};
use crate::vssx::load_vssx_bytes;

/// Describe a legacy compound file enough to make diagnostics useful.
#[must_use]
pub fn describe_legacy(bytes: &[u8]) -> String {
    if detect_format(bytes) != StencilFormat::Vss {
        return String::new();
    }
    match cfb::CompoundFile::open(Cursor::new(bytes.to_vec())) {
        Ok(file) => {
            if file.exists("/VisioDocument") {
                " (valid Visio compound file)".to_owned()
            } else {
                " (OLE compound file, no VisioDocument stream)".to_owned()
            }
        }
        Err(err) => format!(" (damaged OLE compound file: {err})"),
    }
}

/// Whether LibreOffice's headless converter can be found on `PATH`.
#[must_use]
pub fn soffice_available() -> bool {
    Command::new("soffice")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Convert a `.vss` file to `.vssx` in a temporary folder and parse the result.
///
/// The returned library keeps the original file stem as its display name and
/// the original path as its source.
pub fn convert_and_load(path: &Path) -> Result<StencilLibrary> {
    let original = std::fs::read(path)?;
    if detect_format(&original) != StencilFormat::Vss {
        return Err(IoError::UnknownFormat(path.display().to_string()));
    }
    let folder = tempfile::Builder::new()
        .prefix("rackcraft-vss-")
        .tempdir()?;
    let output = Command::new("soffice")
        .args(["--headless", "--convert-to", "vssx", "--outdir"])
        .arg(folder.path())
        .arg(path)
        .output();

    let output = match output {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(IoError::LegacyVss {
                detail: describe_legacy(&original),
            });
        }
        Err(err) => return Err(IoError::Io(err)),
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(IoError::Conversion(format!(
            "soffice exited with {}: {}",
            output.status,
            stderr.trim()
        )));
    }

    let converted = find_converted(folder.path())?;
    let bytes = std::fs::read(&converted)?;
    let name = display_name(path);
    load_vssx_bytes(&bytes, &name, Some(path.display().to_string()))
}

fn find_converted(folder: &Path) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case(OsStr::new("vssx")))
        {
            candidates.push(path);
        }
    }
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| IoError::Conversion("soffice produced no .vssx file".to_owned()))
}

fn display_name(path: &Path) -> String {
    path.file_stem()
        .and_then(OsStr::to_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("Converted stencil")
        .to_owned()
}

/// Build the actionable legacy-file error used when no conversion is possible.
#[must_use]
pub fn legacy_error(bytes: &[u8]) -> IoError {
    IoError::LegacyVss {
        detail: describe_legacy(bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_legacy_is_not_described_as_vss() {
        assert!(describe_legacy(b"PK\x03\x04junk").is_empty());
    }

    #[test]
    fn damaged_cfb_is_still_detected() {
        let bytes = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0, 0];
        assert!(describe_legacy(&bytes).contains("OLE compound"));
    }

    #[test]
    fn legacy_error_has_conversion_instructions() {
        let error = legacy_error(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0, 0]);
        let message = error.to_string();
        assert!(message.contains("soffice"));
        assert!(message.contains(".vssx"));
    }
}
