//! Export the built-in sample rack to a vector PDF.
//!
//! ```bash
//! cargo run -p rackcraft-io --example export_sample_pdf -- /tmp/rackcraft.pdf
//! ```

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "rackcraft-sample.pdf".to_owned());
    let document = rackcraft_engine::sample::document();
    rackcraft_io::export_pdf(&document, &path)?;
    println!("wrote {path}");
    Ok(())
}
