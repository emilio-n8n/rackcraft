//! RackCraft native application entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rackcraft_ui::RunOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args
        .iter()
        .any(|argument| argument == "--help" || argument == "-h")
    {
        println!(
            "RackCraft {} — rack & structured-cabling designer\n\n\
             Usage: rackcraft [--sample]\n\n\
             Options:\n  \
             --sample   Open the built-in sample rack\n  \
             --version  Print version and exit\n  \
             --help     Show this help\n\n\
             Project files are saved as .json. Visio 2013+ .vssx stencils can be imported;\n\
             legacy .vss files require conversion with LibreOffice.\n",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    if args.iter().any(|argument| argument == "--version") {
        println!("RackCraft {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let unknown: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|argument| *argument != "--sample")
        .collect();
    if !unknown.is_empty() {
        eprintln!(
            "Unknown argument(s): {}\nRun `rackcraft --help` for usage.",
            unknown.join(", ")
        );
        std::process::exit(2);
    }

    let sample = args.iter().any(|argument| argument == "--sample");
    rackcraft_ui::run(RunOptions { sample })?;
    Ok(())
}
