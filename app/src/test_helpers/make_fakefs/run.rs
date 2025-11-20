use std::process::Command;
use std::fs;

use crate::fixtures;

pub fn run_image_in_terminal(terminal_override: Option<&str>, foreground: bool) {
    let fixtures_dir = fixtures::generate_fixtures();

    fixtures::apply_permissions(&fixtures_dir);

    crate::build::build_image_with_fixtures(Some(&fixtures_dir));

    let _ = fs::remove_dir_all(&fixtures_dir);

    let docker_cmd = "docker run --rm -it --name filezoom-fakefs-run filezoom-fakefs";

    if foreground {
        println!("Running container in foreground in current terminal...");
        let status = Command::new("sh")
            .arg("-c")
            .arg(docker_cmd)
            .status()
            .expect("Failed to run docker run");
        if !status.success() {
            eprintln!("Running the container failed");
            std::process::exit(1);
        }
        return;
    }

    let mut candidates: Vec<&str> = Vec::new();
    if let Some(t) = terminal_override {
        candidates.push(t);
    } else {
        if cfg!(target_os = "macos") {
            candidates.extend(["osascript", "iTerm", "xterm"].iter().copied());
        } else {
            candidates.extend(["gnome-terminal", "xterm", "alacritty", "konsole"].iter().copied());
        }
    }

    for term in candidates {
        let args: Vec<String> = match term {
            "Terminal" => {
                let script = format!("tell application \"Terminal\" to do script \"{}\"", docker_cmd.replace('"', "\\\""));
                vec!["-e".into(), script]
            }
            "iTerm" => {
                let script = format!("tell application \"iTerm\" to create window with default profile command \"{}\"", docker_cmd.replace('"', "\\\""));
                vec!["-e".into(), script]
            }
            "gnome-terminal" => {
                let docker_exec = format!("{}; exec bash", docker_cmd);
                vec!["--".into(), "bash".into(), "-lc".into(), docker_exec]
            }
            other => {
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
            Err(_) => {}
        }
    }

    println!("No supported terminal emulator found. Running docker in the current terminal...");
    let status = Command::new("sh")
        .arg("-c")
        .arg(docker_cmd)
        .status()
        .expect("Failed to run docker run");
    if !status.success() {
        eprintln!("Running the container failed");
        std::process::exit(1);
    }
}

/// Run the image with an isolated fixtures-only Docker volume and open a
/// terminal window by default. If `foreground` is true, run in the current
/// terminal instead of opening a new window.
pub fn run_image_isolated(terminal_override: Option<&str>, foreground: bool) {
    let fixtures_dir = fixtures::generate_fixtures();

    fixtures::apply_permissions(&fixtures_dir);

    crate::build::build_image_with_fixtures(Some(&fixtures_dir));

    // Clean up the local fixtures copy; we'll populate a Docker volume next.
    let _ = fs::remove_dir_all(&fixtures_dir);

    // Create a unique volume name and populate it from the built image.
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs()).unwrap_or(0);
    let vol_name = format!("filezoom_fixtures_{}_{}", std::process::id(), stamp);

    let s = std::process::Command::new("docker")
        .args(["volume", "create", &vol_name])
        .status()
        .expect("failed to create docker volume");
    if !s.success() {
        eprintln!("failed to create docker volume");
        std::process::exit(1);
    }

    // Copy fixtures into the volume root so they appear at `/work/tests/fixtures`
    // when the volume is mounted at `/work/tests` in the app container.
    let copy_cmd = format!(
        "docker run --rm -v {vol}:/data filezoom-fakefs sh -c 'mkdir -p /data && cp -a /work/tests/fixtures /data/'",
        vol = vol_name
    );
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&copy_cmd)
        .status()
        .expect("Failed to populate fixtures volume");
    if !status.success() {
        // Cleanup the volume before bailing out
        let _ = std::process::Command::new("docker").args(["volume", "rm", "-f", &vol_name]).status();
        eprintln!("Failed to populate fixtures volume");
        std::process::exit(1);
    }

    let run_cmd = format!(
        "docker run --rm -it --name filezoom-fakefs-run -v {vol}:/work/tests --read-only --tmpfs /tmp:rw filezoom-fakefs",
        vol = vol_name
    );

    if foreground {
        println!("Running container with isolated fixtures in foreground (volume={})...", vol_name);
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&run_cmd)
            .status()
            .expect("Failed to run docker run");
        // Remove the volume to rollback any changes.
        let _ = std::process::Command::new("docker").args(["volume", "rm", "-f", &vol_name]).status();
        if !status.success() {
            eprintln!("Docker run exited with non-zero status");
            std::process::exit(1);
        }
        return;
    }

    // Attempt to open in a GUI terminal; fall back to current terminal.
    let mut candidates: Vec<&str> = Vec::new();
    if let Some(t) = terminal_override {
        candidates.push(t);
    } else {
        if cfg!(target_os = "macos") {
            // prefer Terminal/iTerm and drive them via `osascript -e`.
            candidates.extend(["Terminal", "iTerm", "xterm"].iter().copied());
        } else {
            candidates.extend(["gnome-terminal", "xterm", "alacritty", "konsole"].iter().copied());
        }
    }

    for term in candidates {
        // On macOS we invoke `osascript -e '<applescript>'` to tell the
        // desired terminal to open and run the docker command. For other
        // terminals we use their usual CLI flags.
        if cfg!(target_os = "macos") && (term == "Terminal" || term == "iTerm") {
            let script = if term == "Terminal" {
                format!("tell application \"Terminal\" to do script \"{}\"", run_cmd.replace('"', "\\\""))
            } else {
                format!("tell application \"iTerm\" to create window with default profile command \"{}\"", run_cmd.replace('"', "\\\""))
            };
            let mut cmd = std::process::Command::new("osascript");
            cmd.arg("-e").arg(script);
            match cmd.spawn() {
                Ok(child) => {
                    println!("Launched terminal '{}' (pid={}) attached to container.", term, child.id());
                    return;
                }
                Err(_) => {}
            }
            continue;
        }

        let args: Vec<String> = match term {
            "gnome-terminal" => {
                let docker_exec = format!("{}; exec bash", run_cmd);
                vec!["--".into(), "bash".into(), "-lc".into(), docker_exec]
            }
            other => {
                let docker_exec = format!("{}; exec bash", run_cmd);
                if other == "xterm" {
                    vec!["-e".into(), "sh".into(), "-c".into(), run_cmd.clone()]
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
            Err(_) => {}
        }
    }

    println!("No supported terminal emulator found. Running docker in the current terminal...");
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&run_cmd)
        .status()
        .expect("Failed to run docker run");
    // Remove the volume to rollback any changes.
    let _ = std::process::Command::new("docker").args(["volume", "rm", "-f", &vol_name]).status();
    if !status.success() {
        eprintln!("Docker run exited with non-zero status");
        std::process::exit(1);
    }
}
