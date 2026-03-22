/// Solo Compiler CLI
///
/// Usage:
///   solo build <file.solo>
///   solo run <file.solo>
///   solo check <file.solo>

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Solo Compiler v0.1.0");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("  solo build <file.solo>  - Compile to executable");
        eprintln!("  solo run <file.solo>    - Compile and run");
        eprintln!("  solo check <file.solo>  - Type check only");
        eprintln!("  solo version            - Show version");
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "version" => {
            println!("Solo Compiler v0.1.0");
            println!("Part of the My Language family");
        }
        "build" | "run" | "check" => {
            if args.len() < 3 {
                eprintln!("Error: Missing input file");
                eprintln!("Usage: solo {} <file.solo>", command);
                process::exit(1);
            }

            let filename = &args[2];
            let source = fs::read_to_string(filename).unwrap_or_else(|err| {
                eprintln!("Error reading file '{}': {}", filename, err);
                process::exit(1);
            });

            if let Err(err) = solo::compile(&source) {
                eprintln!("Compilation error: {}", err);
                process::exit(1);
            }

            println!("✓ Compilation successful");

            if command == "run" {
                // TODO(#runtime): Execute compiled binary
                // Tracking: See ROADMAP.adoc "Phase 1: Solo Compiler"
                // Depends on: Code generation completion
                println!("Execution not yet implemented (see ROADMAP.adoc)");
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Run 'solo' without arguments for help");
            process::exit(1);
        }
    }
}
