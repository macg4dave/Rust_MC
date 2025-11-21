//! Small utilities to preserve file metadata (permissions, timestamps, xattrs).
//!
//! These helpers are intentionally conservative: ownership (UID/GID) is only
//! applied on Unix and treated as best-effort. Metadata copying is designed to
//! be non-fatal for bulk operations (we prefer best-effort behavior for
//! extended attributes and ACLs), but for single-file operations core
//! failures (permissions/timestamps) are propagated.

use std::fs::{self, Permissions};
use std::io;
use std::path::{Path, PathBuf};

use filetime::{set_file_times, FileTime};
use walkdir::WalkDir;

use rayon::prelude::*;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(unix)]
mod unix_extra {
    use super::*;
    use nix::unistd::{chown, Gid, Uid};

    /// Best-effort: copy ownership (UID/GID), xattrs, and POSIX ACLs on Unix.
    /// Failures are ignored because these operations are non-portable and
    /// often require elevated privileges.
    pub(crate) fn copy_unix_extras(src: &Path, dst: &Path) {
        // Ownership
        if let Ok(meta) = fs::metadata(src) {
            let uid = meta.uid();
            let gid = meta.gid();
            let _ = chown(dst, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid)));
        }

        // Extended attributes (xattr)
        if let Ok(names) = xattr::list(src) {
            for name in names {
                let name_s = name.to_string_lossy();
                if let Ok(Some(val)) = xattr::get(src, &*name_s) {
                    let _ = xattr::set(dst, &*name_s, &val);
                }
            }
        }

        // POSIX ACL (project-local helper) best-effort
        if let Ok(Some(acl)) = crate::fs_op::posix_acl::PosixAcl::read_from_path(src) {
            let _ = acl.write_to_path(dst);
        }
    }
}

/// Copy permission bits from `src` to `dst`.
///
/// This propagates IO errors (it's considered a core operation).
pub(crate) fn copy_permissions(src: &Path, dst: &Path) -> io::Result<()> {
    let perms: Permissions = fs::metadata(src)?.permissions();
    fs::set_permissions(dst, perms)
}

/// Preserve metadata from `src` to `dst`.
///
/// If `src` is a file, permissions and timestamps are applied and errors from
/// those operations are propagated. Extra Unix-specific attributes (ownership,
/// xattrs, ACLs) are attempted but treated as best-effort. If `src` is a
/// directory, the tree is walked and metadata is applied to any existing
/// corresponding path under `dst`; missing targets and non-critical failures
/// are ignored to keep bulk copy operations resilient.
pub(crate) fn preserve_all_metadata(src: &Path, dst: &Path) -> io::Result<()> {
    if !src.exists() || !dst.exists() {
        return Ok(());
    }

    if src.is_file() {
        // permissions (critical)
        copy_permissions(src, dst)?;

        // timestamps (critical)
        let meta = fs::metadata(src)?;
        let m = meta.modified()?;
        let a = meta.accessed()?;
        let m_ft = FileTime::from_system_time(m);
        let a_ft = FileTime::from_system_time(a);
        set_file_times(dst, a_ft, m_ft)?;

        // Best-effort extras on Unix
        #[cfg(unix)]
        unix_extra::copy_unix_extras(src, dst);

        return Ok(());
    }

    // Directory: walk and apply metadata to corresponding targets under dst.
    // We treat this as best-effort: ignore missing targets and continue on
    // individual failures.
    let entries: Vec<PathBuf> = WalkDir::new(src)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.into_path())
        .collect();

    // Parallelize the per-entry application to improve throughput.
    entries.into_par_iter().for_each(|p| apply_metadata_to_target(&p, src, dst));

    Ok(())
}

fn apply_metadata_to_target(path: &Path, src_root: &Path, dst_root: &Path) {
    let rel = match path.strip_prefix(src_root) {
        Ok(r) => r,
        Err(_) => return,
    };
    let target = dst_root.join(rel);
    if !target.exists() {
        return;
    }

    // copy permissions (best-effort)
    let _ = copy_permissions(path, &target);

    // copy timestamps (best-effort)
    if let Ok(meta) = fs::metadata(path) {
        if let (Ok(m), Ok(a)) = (meta.modified(), meta.accessed()) {
            let m_ft = FileTime::from_system_time(m);
            let a_ft = FileTime::from_system_time(a);
            let _ = set_file_times(&target, a_ft, m_ft);
        }

        #[cfg(unix)]
        unix_extra::copy_unix_extras(path, &target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;
    use std::time::{Duration, SystemTime};

    #[test]
    fn copy_permissions_and_timestamps_file() -> io::Result<()> {
        let dir = tempdir()?;
        let src = dir.path().join("src.txt");
        let dst = dir.path().join("dst.txt");

        // create src
        {
            let mut f = fs::File::create(&src)?;
            writeln!(f, "hello")?;
        }

        // create dst
        {
            let mut f = fs::File::create(&dst)?;
            writeln!(f, "world")?;
        }

        // set a specific permission on src
        let mut perms = fs::metadata(&src)?.permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o644);
        }
        fs::set_permissions(&src, perms.clone())?;

        // change src timestamps to an older time
        let past = SystemTime::now() - Duration::from_secs(24 * 3600);
        let ft = FileTime::from_system_time(past);
        set_file_times(&src, ft, ft)?;

        // run preserve
        preserve_all_metadata(&src, &dst)?;

        // verify permissions copied
        let dst_meta = fs::metadata(&dst)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode_src = fs::metadata(&src)?.permissions().mode();
            let mode_dst = dst_meta.permissions().mode();
            assert_eq!(mode_src & 0o777, mode_dst & 0o777);
        }

        // verify timestamps roughly match (within a second)
        let dst_m = dst_meta.modified()?;
        let src_m = fs::metadata(&src)?.modified()?;
        let diff = dst_m.duration_since(src_m).unwrap_or_else(|e| e.duration());
        assert!(diff.as_secs() < 2, "timestamps differ too much");

        Ok(())
    }
}
