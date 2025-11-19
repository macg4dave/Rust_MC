// make_fakefs.rs: Rust CLI to build/deploy and run a Docker image with a fake filesystem for testing fileZoom and starts the program in the docker image
// Usage: cargo run --bin make_fakefs -- <command>


mod advanced;
mod build;
mod fixtures;
mod run;

use std::env;
use std::process::{exit};
use std::path::PathBuf;

// fixtures, run and build logic have been moved into the modules under this directory to make
// the binary easier to maintain. Public test helpers remain available through the binary
// by delegating to those modules.

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: make_fakefs <build|generate-fixtures|apply-permissions|run>");
        exit(1);
    }
    let cmd = args[1].as_str();

    match cmd {
        "build" => build::build_image(),
        "generate-fixtures" => {
            let _ = fixtures::generate_fixtures();
        }
        "apply-permissions" => {
            let fixtures = if args.len() > 2 {
                PathBuf::from(&args[2])
            } else {
                // default: tests/fixtures inside the crate
                let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                p.push("tests");
                p.push("fixtures");
                p
            };
            fixtures::apply_permissions(&fixtures);
        }
        "run" => {
            // parse optional flags
            let mut terminal_override: Option<&str> = None;
            let mut foreground = false;
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--foreground" | "-f" => {
                        foreground = true;
                        i += 1;
                    }
                    s if s.starts_with("--terminal=") => {
                        terminal_override = Some(&s[11..]);
                        i += 1;
                    }
                    "--terminal" => {
                        if i + 1 < args.len() {
                            terminal_override = Some(&args[i + 1]);
                            i += 2;
                        } else {
                            eprintln!("--terminal requires an argument");
                            exit(1);
                        }
                    }
                    _ => {
                        // ignore unknown for now
                        i += 1;
                    }
                }
            }
            run::run_image_in_terminal(terminal_override, foreground);
        }
        _ => {
            eprintln!("Usage: make_fakefs <build|generate-fixtures|apply-permissions|run>");
            exit(1);
        }
    }



}


