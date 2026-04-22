//! Convert a LaTeX file to Leaflet JSON.
//!
//! Usage:
//!     cargo run -p latex-to-leaflet --example convert -- examples/sample.tex

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.tex>", args[0]);
        process::exit(1);
    }

    let path = &args[1];
    let source = fs::read(path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", path, e);
        process::exit(1);
    });

    let schema = latex_to_leaflet::parse_latex_to_leaflet(&source, path).unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        process::exit(1);
    });

    let json = leaflet_protocol::emit_leaflet_document(&schema).unwrap_or_else(|e| {
        eprintln!("Emit error: {}", e);
        process::exit(1);
    });

    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
