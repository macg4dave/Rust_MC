use std::env;
use std::fs;
use std::path::PathBuf;

#[test]
fn container_modifies_fixture_only_inside_container() {
    // This test is intended to run inside the baked container image where
    // the fixtures are baked into the container filesystem. The helper
    // scripts run the container with `BAKED_FIXTURES=1` in the environment
    // so the test will run only there. When run on the host (normal dev),
    // the test will be a no-op (skipped) to avoid modifying host fixtures.
    if env::var("BAKED_FIXTURES").is_err() {
        eprintln!("Skipping container isolation test when not running inside baked container.");
        return;
    }

    let mut fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixtures.push("tests");
    fixtures.push("fixtures");

    let marker = fixtures.join("CONTAINER_MODIFIED.txt");

    // Ensure we can write to the fixtures area inside the container
    let _ = fs::remove_file(&marker);
    fs::write(&marker, "modified in container").expect("should write marker in baked fixtures");
    assert!(
        marker.exists(),
        "marker file should exist after modification inside container"
    );

    // Optionally read it back
    let content = fs::read_to_string(&marker).expect("read marker");
    assert!(content.contains("modified in container"));
}
