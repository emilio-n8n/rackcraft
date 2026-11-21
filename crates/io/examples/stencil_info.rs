//! Print a summary of a Visio stencil. Useful for debugging imports.
//!
//! ```bash
//! cargo run -p rackcraft-io --example stencil_info -- samples/rackcraft-datacenter.vssx
//! ```

use std::error::Error;

use rackcraft_io::load_stencil_file;

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: stencil_info <file.vssx|file.vss>")?;
    let library = load_stencil_file(&path)?;
    println!(
        "{} — {} masters ({} warnings)",
        library.name,
        library.masters.len(),
        library.warnings.len()
    );
    for master in &library.masters {
        println!(
            "\n[{}] {}\n  {} · {:.1} × {:.1} mm\n  {} paths · {} connections · {} shape-data fields",
            master.id,
            master.name,
            master.category,
            master.width_mm,
            master.height_mm,
            master.graphics.len(),
            master.connection_points.len(),
            master.shape_data.len()
        );
        for field in &master.shape_data {
            if !field.value.trim().is_empty() {
                println!("    {}: {}", field.label, field.value);
            }
        }
        if let Some(mount) = &master.mount {
            println!("    rack: {}U", mount.units);
        }
    }
    for warning in &library.warnings {
        eprintln!("warning: {warning}");
    }
    Ok(())
}
