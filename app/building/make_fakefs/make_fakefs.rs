// make_fakefs.rs: Rust CLI to build a Docker image with a fake filesystem for testing fileZoom
// Usage: cargo run --bin make_fakefs -- <command>
// Commands: build, generate-fixtures, apply-permissions

use std::process::{Command, exit};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use fileZoom::building::make_fakefs_lib as make_fakefs_lib;

fn build_image() {
    let current = env::current_dir().expect("Failed to get current dir");
    match make_fakefs_lib::build_image_with_fixtures(None, &current) {
        Ok(()) => println!("Docker image 'filezoom-fakefs' built successfully."),
        Err(e) => {
            eprintln!("Failed to build image: {}", e);
            exit(1);
        }
    }
}

fn build_image_with_fixtures(fixtures: Option<&Path>) {
    let current = env::current_dir().expect("Failed to get current dir");
    match make_fakefs_lib::build_image_with_fixtures(fixtures, &current) {
        Ok(()) => println!("Docker image 'filezoom-fakefs' built successfully (using temp context)."),
        Err(e) => {
            eprintln!("Failed to build image: {}", e);
            exit(1);
        }
    }
}

fn generate_fixtures() -> PathBuf {
    // Create fixtures in OS tmp dir and return the path so callers can clean up
    let mut fixtures_dir = env::temp_dir();
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    fixtures_dir.push(format!("filezoom_fixtures_{}_{}", std::process::id(), stamp));

    if fixtures_dir.exists() {
        let _ = fs::remove_dir_all(&fixtures_dir);
    }
    fs::create_dir_all(&fixtures_dir).expect("Failed to create fixtures dir");
    // Example: create a test file and directory
    fs::write(fixtures_dir.join("file1.txt"), b"test file\n").unwrap();
    fs::create_dir_all(fixtures_dir.join("dirA")).unwrap();
    println!("Fixtures generated in {}", fixtures_dir.display());
    fixtures_dir
}

fn apply_permissions(fixtures_dir: &Path) {
    // Example: set file permissions (rw-r--r--)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file = fixtures_dir.join("file1.txt");
        let perms = fs::Permissions::from_mode(0o644);
        fs::set_permissions(&file, perms).unwrap();
        println!("Permissions set for {}", file.display());
    }
    #[cfg(not(unix))]
    {
        println!("Permission setting is only supported on Unix");
    }
}

fn run_image_in_terminal(terminal_override: Option<&str>, foreground: bool) {
    // Generate fixtures in tmp and build using a temporary build context so all writes stay in /tmp
    let fixtures_dir = generate_fixtures();

    // Optionally apply permissions to the temp fixtures
    apply_permissions(&fixtures_dir);

    build_image_with_fixtures(Some(&fixtures_dir));

    // Clean up fixtures now that the image is built
    let _ = fs::remove_dir_all(&fixtures_dir);

    let docker_cmd = "docker run --rm -it --name filezoom-fakefs-run filezoom-fakefs";

    // If foreground is requested, run in the current terminal and return.
    if foreground {
        println!("Running container in foreground in current terminal...");
        let status = Command::new("sh")
            .arg("-c")
            .arg(docker_cmd)
            .status()
            .expect("Failed to run docker run");
        if !status.success() {
            eprintln!("Running the container failed");
            exit(1);
        }
        return;
    }

    // Try several common terminal emulators to open a new window attached to the container
    // Choose the terminal list based on OS, allow override
    let mut candidates: Vec<&str> = Vec::new();
    if let Some(t) = terminal_override {
        candidates.push(t);
    } else {
        // Defaults per OS
        if cfg!(target_os = "macos") {
            // Prefer Apple Terminal via osascript on macOS
            candidates.extend(["osascript", "iTerm", "xterm"].iter().copied());
        } else {
            // Linux defaults: prefer GNOME (common on Fedora), then xterm, alacritty, konsole
            candidates.extend(["gnome-terminal", "xterm", "alacritty", "konsole"].iter().copied());
        }
    }

    for term in candidates {
        // Build args per terminal type
        let args: Vec<String> = match term {
            "Terminal" => {
                // Use AppleScript to tell the Terminal app to run the command
                let script = format!("tell application \"Terminal\" to do script \"{}\"", docker_cmd.replace('"', "\\\""));
                vec!["-e".into(), script]
            }
            "iTerm" => {
                // Fallback: try opening iTerm2 via AppleScript
                let script = format!("tell application \"iTerm\" to create window with default profile command \"{}\"", docker_cmd.replace('"', "\\\""));
                vec!["-e".into(), script]
            }
            "gnome-terminal" => {
                let docker_exec = format!("{}; exec bash", docker_cmd);
                vec!["--".into(), "bash".into(), "-lc".into(), docker_exec]
            }
            other => {
                // Generic terminals that accept -e or -c
                let docker_exec = format!("{}; exec bash", docker_cmd);
                if other == "xterm" {
                    vec!["-e".into(), "sh".into(), "-c".into(), docker_cmd.to_string()]
                } else {
                    vec!["-e".into(), "bash".into(), "-lc".into(), docker_exec]
                }
            }
        };

        let mut cmd = std::process::Command::new(term);
        cmd.args(&args);
        match cmd.spawn() {
            Ok(child) => {
                println!("Launched terminal '{}' (pid={}) attached to container.", term, child.id());
                return;
            }
            Err(_) => {
                // try next terminal
            }
        }
    }

    // If no terminal emulator was found, fall back to running in the current terminal
    println!("No supported terminal emulator found. Running docker in the current terminal...");
    let status = Command::new("sh")
        .arg("-c")
        .arg(docker_cmd)
        .status()
        .expect("Failed to run docker run");
    if !status.success() {
        eprintln!("Running the container failed");
        exit(1);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: make_fakefs <build|generate-fixtures|apply-permissions|run>");
        exit(1);
    }
    match args[1].as_str() {
        "build" => build_image(),
        "generate-fixtures" => { let _ = generate_fixtures(); },
        "apply-permissions" => {
            let fixtures_dir = Path::new("../tests/fixtures");
            apply_permissions(fixtures_dir);
        }
        "run" => {
            // parse optional flags after 'run', e.g. --terminal=NAME or --terminal NAME
            let mut term_override: Option<String> = None;
            let mut foreground = false;
            let mut i = 2;
            while i < args.len() {
                let a = &args[i];
                if a.starts_with("--terminal=") {
                    if let Some(val) = a.splitn(2, '=').nth(1) {
                        term_override = Some(val.to_string());
                    }
                } else if a == "--terminal" || a == "-t" {
                    if i + 1 < args.len() {
                        term_override = Some(args[i + 1].clone());
                        i += 1;
                    }
                } else if a == "--foreground" || a == "--fg" {
                    foreground = true;
                }
                i += 1;
            }
            run_image_in_terminal(term_override.as_deref(), foreground);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            exit(1);
        }
    }
}

// Note: unit/integration tests for fixture generation and permission helpers
// have been moved to the test-only helper module under
// `app/src/test_helpers/` and the integration test file
// `app/building/make_fakefs/make_fakefs_tests.rs`. Keeping tests out of the
// binary avoids duplication and makes it easier for CI to run the suite with
// the `test-helpers` feature enabled.
