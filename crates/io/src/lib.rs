//! Visio stencil import and BOM export for RackCraft.
//!
//! - [`load_stencil_file`] dispatches modern `.vssx` and legacy `.vss`.
//! - [`load_vssx_bytes`] parses an OPC/Visio package in memory.
//! - [`export_bom_xlsx`] and [`export_bom_csv`] write inventories.
//!
//! The parser never panics on malformed input: damaged masters are skipped and
//! reported through [`StencilLibrary::warnings`].

pub mod bom_export;
pub mod error;
pub mod legacy;
pub mod legacy_native;
pub mod legacy_svg;
pub mod model;
pub mod pdf;
pub mod vssx;

use std::path::Path;

pub use bom_export::{BOM_HEADERS, bom_csv_bytes, bom_xlsx_bytes, export_bom_csv, export_bom_xlsx};
pub use error::{IoError, Result};
pub use legacy::{describe_legacy, find_executable, soffice_available, vss2xhtml_available};
pub use model::{
    IconImage, Master, StencilFormat, StencilLibrary, detect_format, infer_category, infer_tags,
};
pub use pdf::{export_pdf, pdf_bytes};
pub use vssx::load_vssx_bytes;

/// Bundled stencils compiled into the binary so `--sample` works offline.
pub const BUNDLED_STENCILS: &[(&str, &[u8])] = &[
    (
        "RackCraft Data Center",
        include_bytes!("../../../samples/rackcraft-datacenter.vssx"),
    ),
    (
        "RackCraft Modular",
        include_bytes!("../../../samples/rackcraft-modular.vssx"),
    ),
    (
        "Network Topology",
        include_bytes!("../../../samples/network-topology.vssx"),
    ),
];

/// Result of loading the bundled sample libraries.
#[derive(Debug, Default)]
pub struct BundledStencils {
    /// Successfully parsed libraries.
    pub libraries: Vec<StencilLibrary>,
    /// Per-library load errors (kept non-fatal so the app still opens).
    pub errors: Vec<String>,
}

/// Load the stencil libraries embedded in the binary.
#[must_use]
pub fn load_bundled_stencils() -> BundledStencils {
    let mut bundled = BundledStencils::default();
    for (name, bytes) in BUNDLED_STENCILS {
        match load_vssx_bytes(bytes, name, None) {
            Ok(library) => bundled.libraries.push(library),
            Err(err) => bundled.errors.push(format!("{name}: {err}")),
        }
    }
    bundled
}

/// Load a stencil from a file, detecting `.vssx` and legacy `.vss`.
pub fn load_stencil_file(path: impl AsRef<Path>) -> Result<StencilLibrary> {
    let path = path.as_ref();
    let bytes = std::fs::read(path)?;
    match detect_format(&bytes) {
        StencilFormat::Vssx => {
            let name = path
                .file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("Imported stencil");
            load_vssx_bytes(&bytes, name, Some(path.display().to_string()))
        }
        StencilFormat::Vss => legacy::convert_and_load(path),
        StencilFormat::Unknown => Err(IoError::UnknownFormat(path.display().to_string())),
    }
}

/// Load a stencil from memory. Legacy `.vss` cannot be converted without a path.
pub fn load_stencil_bytes(bytes: &[u8], display_name: &str) -> Result<StencilLibrary> {
    match detect_format(bytes) {
        StencilFormat::Vssx => load_vssx_bytes(bytes, display_name, None),
        StencilFormat::Vss => {
            match legacy_native::load_native_fallback(bytes, display_name, None) {
                Ok(library) => Ok(library),
                Err(_) => Err(legacy::legacy_error(bytes)),
            }
        }
        StencilFormat::Unknown => Err(IoError::UnknownFormat(display_name.to_owned())),
    }
}

/// Export both formats next to each other, returning the two written paths.
pub fn export_bom_both(
    bom: &rackcraft_engine::Bom,
    folder: impl AsRef<Path>,
    stem: &str,
    title: &str,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    let folder = folder.as_ref();
    let xlsx = folder.join(format!("{stem}.xlsx"));
    let csv = folder.join(format!("{stem}.csv"));
    export_bom_xlsx(bom, &xlsx, title)?;
    export_bom_csv(bom, &csv)?;
    Ok((xlsx, csv))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Shape;

    #[test]
    fn bundled_libraries_parse_offline() {
        let bundled = load_bundled_stencils();
        assert!(
            bundled.errors.is_empty(),
            "bundled stencil errors: {:?}",
            bundled.errors
        );
        assert_eq!(bundled.libraries.len(), 3);
        let total: usize = bundled
            .libraries
            .iter()
            .map(|library| library.masters.len())
            .sum();
        assert_eq!(total, 12);
    }

    #[test]
    fn sample_library_has_expected_masters() {
        let bundled = load_bundled_stencils();
        let data_center = &bundled.libraries[0];
        let names: Vec<&str> = data_center
            .masters
            .iter()
            .map(|master| master.name.as_str())
            .collect();
        assert!(names.contains(&"RackCraft 25U Open Frame Rack"));
        assert!(names.contains(&"24-Port CAT6 Patch Panel"));
        assert!(names.contains(&"1U Cable Manager"));
        let rack = data_center
            .masters
            .iter()
            .find(|master| master.mount.is_some())
            .unwrap();
        assert_eq!(rack.mount.as_ref().unwrap().units, 25);
        assert_eq!(rack.connection_points.len(), 25);
        assert!(rack.icon.is_some());
    }

    #[test]
    fn network_library_has_four_way_symbols() {
        let bundled = load_bundled_stencils();
        let network = &bundled.libraries[2];
        assert_eq!(network.name, "Network Topology");
        assert_eq!(network.masters.len(), 6);
        for master in &network.masters {
            assert_eq!(master.connection_points.len(), 4, "{}", master.name);
            assert!(master.mount.is_none());
            assert!(master.graphics.len() >= 3);
        }
        let firewall = network
            .masters
            .iter()
            .find(|master| master.name == "Firewall")
            .unwrap();
        assert_eq!(
            crate::model::shape_value(&firewall.shape_data, &["Part Number"]),
            Some("RC-NET-FW")
        );
    }

    #[test]
    fn sample_library_has_shape_data() {
        let bundled = load_bundled_stencils();
        let modular = &bundled.libraries[1];
        let panel = modular
            .masters
            .iter()
            .find(|master| master.name.contains("Modular"))
            .unwrap();
        assert_eq!(
            crate::model::shape_value(&panel.shape_data, &["Part Number"]),
            Some("RC-MPP-24")
        );
        assert_eq!(panel.connection_points.len(), 24);
        let paths = panel.outline_paths();
        assert!(!paths.is_empty());
        assert!(paths[0].bounding_box().width() > 400.0);
        let keystone = modular
            .masters
            .iter()
            .find(|master| master.name.contains("Keystone"))
            .unwrap();
        assert_eq!(keystone.category, "Keystone jacks");
        assert!(keystone.icon.is_some());
    }

    #[test]
    fn instantiated_keystone_glues_to_modular_panel() {
        let bundled = load_bundled_stencils();
        let modular = &bundled.libraries[1];
        let panel_master = modular
            .masters
            .iter()
            .find(|master| master.name.contains("Modular"))
            .unwrap();
        let keystone_master = modular
            .masters
            .iter()
            .find(|master| master.name.contains("Keystone"))
            .unwrap();
        let panel = panel_master.to_item(
            "RackCraft Modular",
            rackcraft_engine::Point::new(100.0, 100.0),
        );
        let mut doc = rackcraft_engine::Document::new("Snap");
        doc.add_item(panel);
        let slot = panel_master.connection_points[4].clone();
        let target_x = 100.0 + slot.x * panel_master.width_mm;
        let target_y = 100.0 + slot.y * panel_master.height_mm;
        let keystone = keystone_master.to_item(
            "RackCraft Modular",
            rackcraft_engine::Point::new(target_x - 7.35 + 2.0, target_y - 8.9 - 1.5),
        );
        let result = rackcraft_engine::snap_item(
            &doc,
            &keystone,
            keystone.position,
            &rackcraft_engine::SnapSettings::default(),
        );
        assert_eq!(
            result.kind,
            Some(rackcraft_engine::SnapKind::ConnectionPoint)
        );
        assert!((result.position.x - (target_x - 7.35)).abs() < 1e-9);
        assert!((result.position.y - (target_y - 8.9)).abs() < 1e-9);
    }
}
