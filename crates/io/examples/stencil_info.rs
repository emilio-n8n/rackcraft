//! Print a summary of a Visio stencil. Useful for debugging imports.
//!
//! ```bash
//! cargo run -p rackcraft-io --example stencil_info -- samples/rackcraft-datacenter.vssx
//! ```

use std::error::Error;

use rackcraft_io::load_stencil_file;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args
        .iter()
        .find(|argument| !argument.starts_with("--"))
        .ok_or("usage: stencil_info [--art] <file.vssx|file.vss>")?
        .clone();
    let materialize = args.iter().any(|argument| argument == "--art");
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
        if materialize && !master.foreign_vector_entries.is_empty() {
            match library.materialize_vector_graphics(master) {
                Ok(graphics) => println!(
                    "    metafile: {} entries, {} converted paths",
                    master.foreign_vector_entries.len(),
                    graphics.len()
                ),
                Err(err) => println!("    metafile: ERROR {err}"),
            }
        }
    }
    for warning in &library.warnings {
        eprintln!("warning: {warning}");
    }
    Ok(())
}
