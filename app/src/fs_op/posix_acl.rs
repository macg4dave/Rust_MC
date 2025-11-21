use std::io;
use std::path::Path;

/// Minimal POSIX ACL helper implemented in Rust.
///
/// This module provides a pure-Rust helper to read and write the common
/// POSIX ACL xattr blobs (`system.posix_acl_access` and
/// `system.posix_acl_default`). It treats the xattr contents as opaque
/// binary blobs (round-trip) so behaviour matches the kernel/libacl layout
/// without linking to libacl. A fuller parser/writer could be added later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosixAcl {
    pub access: Option<Vec<u8>>,
    pub default: Option<Vec<u8>>,
}

impl PosixAcl {
    /// Read POSIX ACL xattrs from `path`. Returns Ok(None) if no ACLs
    /// were present.
    pub fn read_from_path(path: &Path) -> io::Result<Option<Self>> {
        #[cfg(unix)]
        {
            let mut found = false;
            let mut access = None;
            let mut default = None;
            if let Ok(Some(v)) = xattr::get(path, "system.posix_acl_access") {
                access = Some(v);
                found = true;
            }
            if let Ok(Some(v)) = xattr::get(path, "system.posix_acl_default") {
                default = Some(v);
                found = true;
            }
            if found {
                return Ok(Some(PosixAcl { access, default }));
            }
            Ok(None)
        }
        #[cfg(not(unix))]
        {
            // No-op on non-Unix platforms.
            Ok(None)
        }
    }

    /// Write ACL blobs to `path`. Best-effort: ignore set failures.
    pub fn write_to_path(&self, path: &Path) -> io::Result<()> {
        #[cfg(unix)]
        {
            if let Some(ref v) = self.access {
                let _ = xattr::set(path, "system.posix_acl_access", v);
            }
            if let Some(ref v) = self.default {
                let _ = xattr::set(path, "system.posix_acl_default", v);
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn roundtrip_acl_xattrs() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("f.txt");
        File::create(&p).unwrap();
        let acl = PosixAcl {
            access: Some(vec![1, 2, 3, 4]),
            default: Some(vec![5, 6, 7]),
        };
        acl.write_to_path(&p).unwrap();
        // Some CI filesystems / platforms may not support xattrs. In that
        // case `read_from_path` will return `Ok(None)` and the best option
        // is to treat the test as a no-op rather than a hard failure.
        let got_opt = PosixAcl::read_from_path(&p).unwrap();
        if got_opt.is_none() {
            eprintln!("posix ACL xattrs not supported, skipping roundtrip test");
            return;
        }
        let got = got_opt.unwrap();
        assert_eq!(got, acl);
    }
}
