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
