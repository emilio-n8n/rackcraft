//! RackCraft native application entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use rackcraft_ui::RunOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args
        .iter()
        .any(|argument| argument == "--help" || argument == "-h")
    {
        println!(
            "RackCraft {} — rack & structured-cabling designer\n\n\
             Usage: rackcraft [--sample] [--open <project.json>]\n\n\
             Options:\n  \
             --sample          Open the built-in sample rack\n  \
             --open <file>     Open a RackCraft .json project\n  \
             --version         Print version and exit\n  \
             --help            Show this help\n\n\
             Project files are saved as .json. Visio 2013+ .vssx stencils can be imported;\n\
             legacy .vss files use vss2xhtml/LibreOffice when available.\n",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    if args.iter().any(|argument| argument == "--version") {
        println!("RackCraft {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let mut sample = false;
    let mut open: Option<PathBuf> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--sample" => sample = true,
            "--open" => {
                index += 1;
                let Some(path) = args.get(index) else {
                    eprintln!("--open requires a .json project path");
                    std::process::exit(2);
                };
                open = Some(PathBuf::from(path));
            }
            path if !path.starts_with('-') && path.ends_with(".json") => {
                open = Some(PathBuf::from(path));
            }
            unknown => {
                eprintln!("Unknown argument: {unknown}\nRun `rackcraft --help` for usage.");
                std::process::exit(2);
            }
        }
        index += 1;
    }
    rackcraft_ui::run(RunOptions { sample, open })?;
    Ok(())
}
