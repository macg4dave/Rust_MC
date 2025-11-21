use std::fs;
use std::io;
use std::path::Path;
use filetime::{FileTime, set_file_times};
use walkdir::WalkDir;
use rayon::prelude::*;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use nix::unistd::{chown, Gid, Uid};
#[cfg(unix)]
use xattr;
// No external shell tools or native libacl bindings: use xattr-based ACL copy

// Minimal metadata preservation helpers.
//
// These helpers provide small utilities to copy common metadata (permissions)
// from `src` to `dst`. They are intentionally conservative and focus on
// permissions because ownership/UID/GID aren't portable across systems.

/// Copy file permission bits from `src` to `dst` when possible.
#[allow(dead_code)]
pub(crate) fn copy_permissions(src: &Path, dst: &Path) -> io::Result<()> {
    let perms = fs::metadata(src)?.permissions();
    fs::set_permissions(dst, perms)
}

/// Placeholder for future metadata copying (timestamps, ownership).
/// Implement platform-specific behavior here when needed.
#[allow(dead_code)]
pub(crate) fn preserve_all_metadata(_src: &Path, _dst: &Path) -> io::Result<()> {
    // Default behavior: preserve permission bits and timestamps for the
    // single `src` -> `dst` path. If `src` is a directory this will copy
    // metadata recursively for entries under `src` to the corresponding
    // paths under `dst`.
    //
    // This is intentionally conservative: ownership (UID/GID) is not
    // touched because it is not generally portable across environments.
    if !_src.exists() || !_dst.exists() {
        return Ok(());
    }

    if _src.is_file() {
        // copy permissions
        let _ = copy_permissions(_src, _dst);
        // copy timestamps
        if let Ok(meta) = fs::metadata(_src) {
            if let (Ok(m), Ok(a)) = (meta.modified(), meta.accessed()) {
                let m_ft = FileTime::from_system_time(m);
                let a_ft = FileTime::from_system_time(a);
                let _ = set_file_times(_dst, a_ft, m_ft);
            }
            // Try to copy ownership (UID/GID) on Unix when possible.
            #[cfg(unix)]
            {
                let uid = meta.uid();
                let gid = meta.gid();
                // Best-effort: ignore errors (permissions may prevent chown).
                let _ = chown(_dst, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid)));
            }
            // Try to copy extended attributes (xattrs) when available.
            #[cfg(unix)]
            {
                if let Ok(names) = xattr::list(_src) {
                    for name in names {
                        let name_s = name.to_string_lossy();
                        if let Ok(Some(val)) = xattr::get(_src, &*name_s) {
                            let _ = xattr::set(_dst, &*name_s, &val);
                        }
                    }
                }

                // Best-effort: copy POSIX ACLs by copying the well-known binary xattr
                // names used on many Linux filesystems. This avoids invoking external
                // tools or linking against libacl and keeps the implementation fully
                // Rust-contained. The xattr names are filesystem-dependent; failures are
                // silently ignored as this is best-effort behaviour.
                #[cfg(unix)]
                {
                    // Use the Rust-only POSIX ACL helper to round-trip ACL blobs.
                    if let Ok(Some(acl)) = crate::fs_op::posix_acl::PosixAcl::read_from_path(_src) {
                        let _ = acl.write_to_path(_dst);
                    }
                }
            }
        }
        return Ok(());
    }

    // For directories walk the tree and apply permissions + timestamps
    // to each matching path under dst. We tolerate missing targets and
    // treat metadata copying as best-effort to avoid hard failures during
    // bulk copies.
    // Collect entries first then apply metadata in parallel. This improves
    // throughput on machines with multiple cores while keeping behaviour
    // best-effort (we ignore missing targets).
    let entries: Vec<_> = WalkDir::new(_src)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .collect();

    entries.into_par_iter().for_each(|e| {
        let rel = match e.path().strip_prefix(_src) {
            Ok(r) => r,
            Err(_) => return,
        };
        let target = _dst.join(rel);
        if !target.exists() {
            return;
        }
        // copy permissions
        let _ = copy_permissions(e.path(), &target);
        // copy timestamps
        if let Ok(meta) = fs::metadata(e.path()) {
            if let (Ok(m), Ok(a)) = (meta.modified(), meta.accessed()) {
                let m_ft = FileTime::from_system_time(m);
                let a_ft = FileTime::from_system_time(a);
                let _ = set_file_times(&target, a_ft, m_ft);
            }

            // Try to copy ownership (UID/GID) on Unix when possible.
            #[cfg(unix)]
            {
                let uid = meta.uid();
                let gid = meta.gid();
                let _ = chown(&target, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid)));
            }

            // Copy extended attributes when possible (best-effort).
            #[cfg(unix)]
            {
                if let Ok(names) = xattr::list(e.path()) {
                    for name in names {
                        let name_s = name.to_string_lossy();
                        if let Ok(Some(val)) = xattr::get(e.path(), &*name_s) {
                            let _ = xattr::set(&target, &*name_s, &val);
                        }
                    }
                }

                #[cfg(unix)]
                {
                    if let Ok(Some(acl)) = crate::fs_op::posix_acl::PosixAcl::read_from_path(e.path()) {
                        let _ = acl.write_to_path(&target);
                    }
                }
            }
        }
    });
    Ok(())
}
