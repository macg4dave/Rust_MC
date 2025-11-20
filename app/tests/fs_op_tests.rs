use assert_fs::prelude::*;
use fileZoom::fs_op::files::*;

#[test]
fn basic_copy_move_rename_remove() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // create src file
    let src = temp.child("a.txt");
    src.write_str("hello")?;

    // copy to new file path
    let dest = temp.child("b.txt");
    copy_path(src.path(), dest.path())?;
    assert!(dest.path().exists());

    // rename dest within same dir
    rename_path(dest.path(), "b2.txt")?;
    assert!(temp.child("b2.txt").exists());

    // move file to subdir
    let sub = temp.child("sub");
    sub.create_dir_all()?;
    move_path(temp.child("b2.txt").path(), sub.path())?;
    assert!(sub.child("b2.txt").exists());

    // remove file
    remove_path(sub.child("b2.txt").path())?;
    assert!(!sub.child("b2.txt").exists());

    Ok(())
}

#[test]
fn copy_dir_recursive() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let d = temp.child("dir1");
    d.create_dir_all()?;
    d.child("one.txt").write_str("1")?;
    d.child("sub").create_dir_all()?;
    d.child("sub/two.txt").write_str("2")?;

    let dest = temp.child("dir2");
    copy_path(d.path(), dest.path())?;
    assert!(dest.child("one.txt").exists());
    assert!(dest.child("sub/two.txt").exists());

    Ok(())
}
