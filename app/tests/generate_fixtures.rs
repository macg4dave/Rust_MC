use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::PathBuf;

#[test]
fn rust_generate_fixtures_creates_manifest_and_files() {
    // Run the make_fakefs binary with the generate-fixtures command
    let mut cmd = cargo_bin_cmd!("make_fakefs");
    let output = cmd
        .arg("generate-fixtures")
        .assert()
        .get_output()
        .stdout
        .clone();
    let out = String::from_utf8_lossy(&output);

    // Expect printed output with manifest path (new Rust generator prints "Wrote N entries to <manifest>")
    assert!(
        out.contains("Wrote"),
        "output did not indicate manifest: {}",
        out
    );
    let start = out.find("Wrote").expect("Wrote marker");
    let manifest_part = out[start..]
        .split_whitespace()
        .last()
        .expect("manifest path");
    let manifest = PathBuf::from(manifest_part);

    // manifest should exist and contain at least the four deterministic entries
    assert!(manifest.exists());
    let content = fs::read_to_string(&manifest).expect("read manifest");
    assert!(content.contains("emoji-"));

    // Clean up the generated fixtures directory
    let fixtures_dir = manifest.parent().unwrap();
    let _ = fs::remove_dir_all(fixtures_dir);
}
