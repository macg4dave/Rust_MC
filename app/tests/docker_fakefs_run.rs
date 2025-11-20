use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

/// Integration test that builds a Docker image containing a large set of
/// generated files and (optionally) runs the container in the current
/// terminal. This test is ignored by default and will only run when the
/// developer explicitly opts in by setting `RUN_DOCKER_FAKEFS=1` in the
/// environment. To open a GUI terminal window instead, also set
/// `ATTACH_TERMINAL=1` (desktop-only).
#[test]
#[ignore]
fn docker_fakefs_run() {
    if env::var("RUN_DOCKER_FAKEFS").is_err() {
        eprintln!("Skipping docker fakefs test; set RUN_DOCKER_FAKEFS=1 to run");
        return;
    }

    // Ensure docker is available
    match Command::new("docker").arg("info").status() {
        Ok(s) if s.success() => {}
        _ => {
            eprintln!("Docker does not appear to be available; skipping test");
            return;
        }
    }

    // Create a temporary fixtures directory with many files and folders.
    let mut fixtures_dir: PathBuf = env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    fixtures_dir.push(format!(
        "filezoom_docker_test_{}_{}",
        std::process::id(),
        stamp
    ));

    if fixtures_dir.exists() {
        let _ = fs::remove_dir_all(&fixtures_dir);
    }
    fs::create_dir_all(&fixtures_dir).expect("failed to create fixtures dir");

    // Generate a moderate number of files in multiple subdirectories.
    for i in 0..400usize {
        let dir = fixtures_dir.join(format!("nested/dir_{:03}", i % 40));
        fs::create_dir_all(&dir).expect("failed to create nested dir");
        let fpath = dir.join(format!("file_{:04}.txt", i));
        let mut f = fs::File::create(&fpath).expect("failed to create file");
        writeln!(f, "This is test file {}", i).expect("failed to write");
    }

    // Build the docker image using the repository/build helpers. The helper
    // will run `cargo build --release` and `docker build` as needed.
    let current = env::current_dir().expect("failed to get current dir");
    match fileZoom::building::make_fakefs_lib::build_image_with_fixtures(
        Some(&fixtures_dir),
        &current,
    ) {
        Ok(()) => println!("Built filezoom-fakefs image successfully."),
        Err(e) => panic!("Failed to build Docker image: {}", e),
    }

    // Cleanup the fixtures directory (the build helper copied it into the
    // temporary build context already).
    let _ = fs::remove_dir_all(&fixtures_dir);

    // Decide whether to attach to the current terminal (foreground) or
    // attempt a GUI-launch flow. Default: run in foreground so the developer
    // sees deterministic behavior when they opt in.
    if env::var("ATTACH_TERMINAL").is_ok() {
        // Attempt to run the `make_fakefs` helper binary to open a GUI
        // terminal attached to the container. If that fails, fall back to
        // a foreground docker run in the current terminal.
        let run_res = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("make_fakefs")
            .arg("--")
            .arg("run")
            .status();
        match run_res {
            Ok(s) if s.success() => return,
            _ => {
                println!("GUI terminal helper failed; falling back to foreground run");
                let status = Command::new("sh")
                    .arg("-c")
                    .arg("docker run --rm -it --name filezoom-fakefs-run filezoom-fakefs")
                    .status()
                    .expect("Failed to run docker run");
                assert!(status.success());
                return;
            }
        }
    }

    // Default: run the container with an isolated fixtures-only volume.
    // Create a temporary Docker volume, populate it from the image's
    // `/work/tests/fixtures`, mount it into the app container read-only,
    // then remove the volume after the test so changes are rolled back.
    {
        let vol_name = format!("filezoom_fixtures_{}_{}", std::process::id(), stamp);

        // Create the volume
        let s = Command::new("docker")
            .args(["volume", "create", &vol_name])
            .status()
            .expect("failed to create docker volume");
        if !s.success() {
            panic!("failed to create docker volume");
        }

        // Populate the volume from the built image's fixtures directory.
        // Use a short lived container that mounts the volume at /data and
        // copies `/work/tests/fixtures` into it.
        // Copy fixtures into the volume root so they appear at `/work/tests/fixtures`
        // when the volume is mounted at `/work/tests` in the app container.
        let copy_cmd = format!(
            "docker run --rm -v {vol}:/data filezoom-fakefs sh -c 'mkdir -p /data && cp -a /work/tests/fixtures /data/'",
            vol = vol_name
        );
        let status = Command::new("sh")
            .arg("-c")
            .arg(&copy_cmd)
            .status()
            .expect("Failed to populate fixtures volume");
        if !status.success() {
            // Cleanup the volume before bailing out
            let _ = Command::new("docker")
                .args(["volume", "rm", "-f", &vol_name])
                .status();
            panic!("Failed to populate fixtures volume");
        }

        // Prepare the run command that mounts only the fixtures volume into
        // the container at the expected path. Keep the container rootfs
        // `--read-only` to restrict access, but mount the fixtures volume as
        // writable so the app can move and modify files during the test.
        // Use `--tmpfs /tmp:rw` for runtime writable temp storage.
        let run_cmd = format!(
            "docker run --rm -it --name filezoom-fakefs-run -v {vol}:/work/tests --read-only --tmpfs /tmp:rw filezoom-fakefs",
            vol = vol_name
        );

        println!(
            "Running container with isolated fixtures (volume={})...",
            vol_name
        );

        let run_status = Command::new("sh")
            .arg("-c")
            .arg(&run_cmd)
            .status()
            .expect("Failed to run docker run");

        // Remove the volume to rollback any changes.
        let _ = Command::new("docker")
            .args(["volume", "rm", "-f", &vol_name])
            .status();

        if !run_status.success() {
            panic!("Docker run exited with non-zero status");
        }
    }
}
