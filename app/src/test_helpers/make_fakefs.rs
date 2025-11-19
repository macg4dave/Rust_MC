use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Create a tiny fake fixtures directory under the OS temporary directory.
///
/// Returns the path to the newly-created fixtures directory. The caller is
/// responsible for removing it when finished. This helper is intentionally
/// small and deterministic so tests can rely on its behaviour.
pub fn generate_fixtures() -> std::io::Result<PathBuf> {
    let mut fixtures_dir = std::env::temp_dir();
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    fixtures_dir.push(format!("filezoom_fixtures_{}_{}", std::process::id(), stamp));

    if fixtures_dir.exists() {
        let _ = fs::remove_dir_all(&fixtures_dir);
    }
    fs::create_dir_all(&fixtures_dir)?;

    // A couple of small test artefacts used by various tests.
    fs::write(fixtures_dir.join("file1.txt"), b"test file\n")?;
    fs::create_dir_all(fixtures_dir.join("dirA"))?;

    Ok(fixtures_dir)
}

/// Apply deterministic permissions for tests. On non-Unix platforms this is
/// a no-op and returns Ok(()).
pub fn apply_permissions(fixtures_dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file = fixtures_dir.join("file1.txt");
        let perms = fs::Permissions::from_mode(0o644);
        fs::set_permissions(&file, perms)?;
    }
    // On non-unix platforms, permissions semantics differ; we keep this
    // function portable and silently succeed there so tests can still run.
    Ok(())
}
