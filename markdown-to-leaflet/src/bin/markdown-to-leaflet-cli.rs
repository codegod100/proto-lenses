//! CLI binary: reads Markdown from stdin, emits Leaflet JSON to stdout.

use std::io::{self, Read, Write};

fn main() {
    let mut input = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut input) {
        eprintln!("Error reading stdin: {e}");
        std::process::exit(1);
    }

    let schema = match markdown_to_leaflet::parse_markdown_to_leaflet(&input, "<stdin>") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Conversion error: {e}");
            std::process::exit(1);
        }
    };

    let json = match leaflet_protocol::emit_leaflet_document(&schema) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Emission error: {e}");
            std::process::exit(1);
        }
    };

    match serde_json::to_string(&json) {
        Ok(s) => {
            if let Err(e) = io::stdout().write_all(s.as_bytes()) {
                eprintln!("Error writing stdout: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("JSON serialization error: {e}");
            std::process::exit(1);
        }
    }
}
