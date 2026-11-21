//! Legacy binary `.vss` import.
//!
//! Order of preference:
//! 1. LibreOffice `soffice` → `.vssx` (best fidelity, full shape data).
//! 2. `vss2xhtml` from libvisio-tools → SVG/XHTML (vector artwork, limited data).
//! 3. Native pure-Rust OLE/CFB inspection → metadata, preview icons and a
//!    clearly labelled placeholder master.

use std::ffi::OsStr;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{IoError, Result};
use crate::legacy_native;
use crate::legacy_svg;
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

/// Whether LibreOffice's headless converter can be found.
#[must_use]
pub fn soffice_available() -> bool {
    find_executable("soffice").is_some()
}

/// Whether `vss2xhtml` (libvisio-tools) can be found.
#[must_use]
pub fn vss2xhtml_available() -> bool {
    find_vss2xhtml().is_some()
}

fn find_vss2xhtml() -> Option<PathBuf> {
    find_executable("vss2xhtml")
}

/// Locate an external converter in `RACKCRAFT_<NAME>`, next to the RackCraft
/// binary (including the AppImage bundle), or on `PATH`.
#[must_use]
pub fn find_executable(name: &str) -> Option<PathBuf> {
    let env_name = format!("RACKCRAFT_{}", name.to_ascii_uppercase());
    if let Ok(path) = std::env::var(&env_name) {
        let path = PathBuf::from(path);
        if is_executable(&path) {
            return Some(path);
        }
    }

    if let Ok(executable) = std::env::current_exe()
        && let Some(folder) = executable.parent()
    {
        for candidate in [
            folder.join(name),
            folder.join(format!("{name}.exe")),
            folder.join("../lib/rackcraft-vss").join(name),
            folder
                .join("../lib/rackcraft-vss")
                .join(format!("{name}.exe")),
        ] {
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }

    let path = std::env::var_os("PATH")?;
    for folder in std::env::split_paths(&path) {
        for filename in [name.to_owned(), format!("{name}.exe")] {
            let candidate = folder.join(filename);
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Convert a `.vss` file using the best available backend.
///
/// The returned library keeps the original file stem as its display name and
/// the original path as its source. If no external converter exists, a native
/// placeholder library is returned with actionable warnings.
pub fn convert_and_load(path: &Path) -> Result<StencilLibrary> {
    let original = std::fs::read(path)?;
    if detect_format(&original) != StencilFormat::Vss {
        return Err(IoError::UnknownFormat(path.display().to_string()));
    }
    let name = display_name(path);
    let source = Some(path.display().to_string());
    let mut diagnostics = Vec::new();

    if let Some(soffice) = find_executable("soffice") {
        match convert_with_soffice(&soffice, path, &name, source.clone()) {
            Ok(mut library) => {
                library.warnings.extend(diagnostics);
                return Ok(library);
            }
            Err(err) => diagnostics.push(format!("LibreOffice conversion failed: {err}")),
        }
    }

    if let Some(vss2xhtml) = find_vss2xhtml() {
        match convert_with_vss2xhtml(&vss2xhtml, path, &name, source.clone()) {
            Ok(mut library) => {
                library.warnings.extend(diagnostics);
                return Ok(library);
            }
            Err(err) => diagnostics.push(format!("vss2xhtml conversion failed: {err}")),
        }
    }

    match legacy_native::load_native_fallback(&original, &name, source) {
        Ok(mut library) => {
            library.warnings.extend(diagnostics);
            Ok(library)
        }
        Err(err) => Err(IoError::LegacyVss {
            detail: format!("{} Error: {err}", describe_legacy(&original)),
        }),
    }
}

fn convert_with_soffice(
    soffice: &Path,
    path: &Path,
    name: &str,
    source: Option<String>,
) -> Result<StencilLibrary> {
    let folder = tempfile::Builder::new()
        .prefix("rackcraft-vss-")
        .tempdir()?;
    let output = Command::new(soffice)
        .args(["--headless", "--convert-to", "vssx", "--outdir"])
        .arg(folder.path())
        .arg(path)
        .output()
        .map_err(|err| IoError::Conversion(format!("cannot run soffice: {err}")))?;
    if !output.status.success() {
        return Err(IoError::Conversion(format!(
            "soffice exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let converted = find_converted(folder.path())?;
    let bytes = std::fs::read(&converted)?;
    load_vssx_bytes(&bytes, name, source)
}

fn convert_with_vss2xhtml(
    tool: &Path,
    path: &Path,
    name: &str,
    source: Option<String>,
) -> Result<StencilLibrary> {
    let output = Command::new(tool)
        .arg(path)
        .output()
        .map_err(|err| IoError::Conversion(format!("cannot run vss2xhtml: {err}")))?;
    if !output.status.success() {
        return Err(IoError::Conversion(format!(
            "vss2xhtml exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    if output.stdout.len() > 64 * 1024 * 1024 {
        return Err(IoError::Conversion(
            "vss2xhtml output exceeded 64 MiB".to_owned(),
        ));
    }
    if output.stdout.is_empty() {
        return Err(IoError::Conversion(
            "vss2xhtml produced no output".to_owned(),
        ));
    }
    legacy_svg::load_vss2xhtml(&output.stdout, name, source)
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

/// Build the actionable legacy-file error used when no backend can read a file.
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
        assert!(message.contains("soffice") || message.contains("vss2xhtml"));
        assert!(message.contains(".vssx"));
    }

    #[test]
    fn current_executable_is_recognised_as_executable() {
        let current = std::env::current_exe().unwrap();
        assert!(is_executable(&current));
        assert!(!is_executable(&current.with_extension("missing")));
    }
}
