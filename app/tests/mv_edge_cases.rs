use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};

use tempfile::tempdir;

use fileZoom::fs_op::mv::{copy_path, move_path};

// Ensure that when the source is a symlink to a directory, `copy_path` copies
// the contents of the target directory into the destination.
#[test]
fn symlink_to_dir_copy_copies_target_contents() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let target = tmp.path().join("target");
    fs::create_dir_all(&target)?;

    let inner = target.join("inner.txt");
    fs::write(&inner, b"hello")?;

    let link = tmp.path().join("link_to_target");
    symlink(&target, &link)?;

    let dest_tmp = tempdir()?;
    let dest = dest_tmp.path().join("out");

    copy_path(&link, &dest)?;

    let copied = dest.join("inner.txt");
    assert!(copied.exists(), "expected copied file exists at {:?}", copied);
    assert_eq!(fs::read_to_string(copied)?, "hello");

    Ok(())
}

// Ensure `move_path` returns an error when the destination is unwritable and
// that the source remains intact after the failed move.
#[test]
fn move_path_returns_error_on_unwritable_dest_and_leaves_source_intact() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let src = tmp.path().join("sourcedir");
    fs::create_dir_all(&src)?;
    let f = src.join("file.txt");
    fs::write(&f, b"data")?;

    let dest_parent = tempdir()?;
    let dest = dest_parent.path().join("dest");

    // Make the destination parent directory read-only so creating files fails.
    let readonly = fs::Permissions::from_mode(0o555);
    fs::set_permissions(dest_parent.path(), readonly)?;

    let res = move_path(&src, &dest);
    assert!(res.is_err(), "expected move_path to error when dest is unwritable");

    // Source should still exist after the failed move.
    assert!(src.exists(), "source dir should still exist after failed move");

    Ok(())
}
