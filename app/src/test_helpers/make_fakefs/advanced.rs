use std::path::{Path, PathBuf};
use rand::RngCore;
use std::process::Command;
use filetime::FileTime;
use std::fs;

/// Generate a test filename using the same heuristics previously embedded in the big function.
pub fn gen_name(i: usize, rng: &mut impl RngCore) -> String {
    match i % 12 {
        0 => format!("file.with.many.dots.{:03}.txt", i),
        1 => format!("name#special&chars@{}.log", 1000 + i),
        2 => format!("unicode-漢字-{}.bin", i),
        3 => format!("emoji-🙂-{:03}", i),
        4 => format!("complex;name;semi;{}.txt", i),
        5 => format!("space name {}.txt", i),
        6 => format!(".leading.dot.{}", i),
        7 => format!("trailing-space-{} ", i),
        8 => {
            const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            let mut short = String::with_capacity(24);
            for _ in 0..24 {
                let idx = (rng.next_u64() as usize) % CHARS.len();
                short.push(CHARS[idx] as char);
            }
            format!("very-long-name-{:010}-{}", i, short)
        }
        9 => format!("reserved%20name{}", i),
        10 => format!("combining-á-{}", i),
        _ => format!("file_{:04}.txt", i),
    }
}

/// Apply a variety of optional, OS-dependent metadata and features to a generated file.
/// This keeps that logic isolated and makes it easier to test/reuse.
pub fn apply_advanced_attrs(rng: &mut impl RngCore, files: &Vec<PathBuf>, fullpath: &Path, fixtures_dir: &Path) -> Vec<PathBuf> {
    let mut created: Vec<PathBuf> = Vec::new();
    #[cfg(unix)]
    {
        if rng.next_u32() % 100 < 30 {
            let xname = format!("user.random{}", rng.next_u64() % 100);
            let xval = format!("xattr-{}", rng.next_u32());
            let _ = Command::new("setfattr")
                .arg("-n")
                .arg(&xname)
                .arg("-v")
                .arg(&xval)
                .arg(&fullpath)
                .status();
        }

        if rng.next_u32() % 100 < 40 {
            let mode = match rng.next_u64() % 7 {
                0 => 0o644,
                1 => 0o600,
                2 => 0o666,
                3 => 0o755,
                4 => 0o700,
                5 => 0o444,
                _ => 0o664,
            };
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(mode);
            let _ = fs::set_permissions(&fullpath, perms);
        }

        if rng.next_u32() % 100 < 50 {
            let days = (rng.next_u64() as i64 % 365) as i64;
            let secs = (rng.next_u64() as i64 % 86400) as i64;
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
            let new_time = now - (days * 86400 + secs) as i64;
            let ft = FileTime::from_unix_time(new_time, 0);
            let _ = filetime::set_file_mtime(&fullpath, ft);
        }

        if rng.next_u32() % 100 < 10 {
            if Command::new("setfacl").arg("-h").status().is_ok() {
                let user = if let Ok(out) = Command::new("id").arg("-un").output() {
                    String::from_utf8_lossy(&out.stdout).trim().to_string()
                } else { String::from("root") };
                let _ = Command::new("setfacl")
                    .arg("-m")
                    .arg(format!("u:{}:r--", user))
                    .arg(&fullpath)
                    .status();
            }
        }

        // occasionally create a symlink pointing to an existing file
        if files.len() > 1 && rng.next_u32() % 100 < 8 {
            let pick = rng.next_u64() as usize % (files.len() - 1);
            let tgt = files[pick].clone();
            if tgt != fullpath {
                let link = fullpath.with_extension("link");
                let _ = std::os::unix::fs::symlink(&tgt, &link);
                created.push(link);
            }
        }

        // create FIFO occasionally
        if rng.next_u64() % 1000 < 8 {
            let dir_for_fifo = fullpath.parent().unwrap_or(&fixtures_dir).to_path_buf();
            let name = format!("fifo_{}_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(), rng.next_u32());
            let p = dir_for_fifo.join(name);
            let _ = Command::new("mkfifo").arg(&p).status();
            created.push(p);
        }
    }

    created
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn gen_name_varies_by_index() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0x1234);
        let n0 = gen_name(0, &mut rng);
        assert!(n0.starts_with("file.with.many.dots."));
        let n8 = gen_name(8, &mut rng);
        assert!(n8.starts_with("very-long-name-"));
    }
}
