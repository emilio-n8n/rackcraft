//! RackCraft's native egui user interface.
//!
//! The UI owns presentation and interaction only. Document mutations go
//! through `rackcraft_engine::History`; stencil parsing and exports live in
//! `rackcraft_io`.

mod app;
mod browser;
mod canvas;
mod inspector;
mod inventory;
mod theme;

pub use app::{RackCraftApp, RunOptions};

/// Run the desktop application.
///
/// # Errors
///
/// Returns an eframe error when the window or graphics backend cannot start.
pub fn run(options: RunOptions) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("RackCraft")
            .with_inner_size([1440.0, 920.0])
            .with_min_inner_size([1024.0, 680.0])
            .with_icon(app_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "RackCraft",
        native_options,
        Box::new(move |cc| Ok(Box::new(RackCraftApp::new(cc, options)) as Box<dyn eframe::App>)),
    )
}

/// Build a simple rack-shaped application icon at runtime.
fn app_icon() -> eframe::egui::IconData {
    let size = 64_u32;
    let mut rgba = vec![0_u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let index = ((y * size + x) * 4) as usize;
            let corner = (x < 6 && y < 6)
                || (x < 6 && y >= size - 6)
                || (x >= size - 6 && y < 6)
                || (x >= size - 6 && y >= size - 6);
            let inside = (6..size - 6).contains(&x) && (6..size - 6).contains(&y);
            let bar = inside
                && ((14..=50).contains(&x) && (20..=23).contains(&y)
                    || (14..=50).contains(&x) && (32..=35).contains(&y)
                    || (14..=50).contains(&x) && (44..=47).contains(&y)
                    || (14..=17).contains(&x) && (18..=48).contains(&y)
                    || (47..=50).contains(&x) && (18..=48).contains(&y));
            let (r, g, b, a) = if bar {
                (255, 255, 255, 255)
            } else if inside {
                (32, 36, 45, 255)
            } else if corner {
                (0, 0, 0, 0)
            } else {
                (232, 87, 63, 255)
            };
            rgba[index] = r;
            rgba[index + 1] = g;
            rgba[index + 2] = b;
            rgba[index + 3] = a;
        }
    }
    eframe::egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}
