use assert_fs::prelude::*;
use fileZoom::fs_op::files::*;

#[cfg(unix)]
use std::os::unix::fs::{symlink, PermissionsExt};

#[test]
#[cfg(unix)]
fn symlink_file_and_dir_copy() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // file symlink
    let target = temp.child("t.txt");
    target.write_str("hello symlink")?;
    let link = temp.child("link.txt");
    #[cfg(unix)]
    symlink(target.path(), link.path())?;

    let dest = temp.child("out.txt");
    // copying via symlink should result in a copied target file
    copy_path(link.path(), dest.path())?;
    assert!(dest.path().exists());
    let s = std::fs::read_to_string(dest.path())?;
    assert_eq!(s, "hello symlink");

    // directory symlink
    let d = temp.child("d1");
    d.create_dir_all()?;
    d.child("one.txt").write_str("1")?;
    let dlink = temp.child("dlink");
    #[cfg(unix)]
    symlink(d.path(), dlink.path())?;

    let destdir = temp.child("dout");
    copy_path(dlink.path(), destdir.path())?;
    assert!(destdir.child("one.txt").exists());

    Ok(())
}

#[test]
fn move_path_fallback_on_rename_error() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let src = temp.child("s.txt");
    src.write_str("move me")?;

    // create a destination path whose parent doesn't exist to cause rename to fail
    let dest = temp.child("nonexistent_parent/subdir/target.txt");

    move_path(src.path(), dest.path())?;
    assert!(dest.path().exists());
    assert!(!src.path().exists());

    Ok(())
}

#[test]
#[cfg(unix)]
fn permission_denied_create_file() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    let temp = assert_fs::TempDir::new()?;
    let no_write = temp.child("nowrite");
    no_write.create_dir_all()?;

    // remove write permission
    let p = fs::metadata(no_write.path())?.permissions();
    let mut perms = p;
    perms.set_mode(0o500); // r-x only
    fs::set_permissions(no_write.path(), perms.clone())?;

    let target = no_write.child("a.txt");
    let res = create_file(target.path());
    assert!(
        res.is_err(),
        "expected create_file to fail due to permission denied"
    );

    // restore perms so cleanup can occur
    perms.set_mode(0o700);
    fs::set_permissions(no_write.path(), perms)?;

    Ok(())
}
