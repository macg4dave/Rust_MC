use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Write;
use filetime::FileTime;
use rand::RngCore;
use crate::advanced;

/// Create a temporary fixtures directory and populate it with many files used by tests.
pub fn generate_fixtures() -> PathBuf {
    let mut fixtures_dir = env::temp_dir();
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    fixtures_dir.push(format!("filezoom_fixtures_{}_{}", std::process::id(), stamp));

    if fixtures_dir.exists() {
        let _ = fs::remove_dir_all(&fixtures_dir);
    }
    fs::create_dir_all(&fixtures_dir).expect("failed to create fixtures dir");

    let manifest = fixtures_dir.join("fixtures_manifest.txt");
    let _ = std::fs::remove_file(&manifest);
    let mut manifest_file = std::fs::File::create(&manifest).expect("failed to create manifest file");

    let total: usize = 500;
    println!("Generating {} fixtures under {}", total, fixtures_dir.display());

    let mut emit = |p: &Path| {
        let rel = p.strip_prefix(&fixtures_dir).unwrap_or(p);
        let _ = writeln!(manifest_file, "{}", rel.to_string_lossy());
    };

    fs::create_dir_all(fixtures_dir.join("deep/level1/level2")).expect("failed to create deep structure");
    let f1 = fixtures_dir.join("emoji-😊");
    fs::write(&f1, "emoji content").expect("failed to write emoji file");
    emit(&f1);

    let f2 = fixtures_dir.join("COMPLEX.name.with.many.dots.log");
    fs::write(&f2, "complex log").expect("failed to write complex file");
    emit(&f2);

    let f3 = fixtures_dir.join("spaces and tabs.txt");
    fs::write(&f3, "contains spaces and\ttabs").expect("failed to write spaces file");
    emit(&f3);

    let f4 = fixtures_dir.join("deep/level1/level2/nested_file.txt");
    fs::write(&f4, "nested content").expect("failed to write nested file");
    emit(&f4);

    let mut count_created: usize = 4;
    let mut files: Vec<PathBuf> = Vec::new();
    files.push(f1);
    files.push(f2);
    files.push(f3);
    files.push(f4);

    let create_file_of_size = |path: &Path, size: usize| {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if size == 0 {
            fs::File::create(path).expect("failed to create empty file");
            return;
        }
        let mut f = fs::File::create(path).expect("failed to create file");
        let block = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-+_";
        let mut written = 0usize;
        while written < size {
            let to_write = std::cmp::min(block.len(), size - written);
            f.write_all(&block[..to_write]).expect("failed to write block");
            written += to_write;
        }
        let _ = f.set_len(size as u64);
    };

    #[allow(deprecated)]
    let mut rng = rand::thread_rng();
    let mut i = 0usize;
    while count_created < total {
        let depth = rng.next_u64() as usize % 6;
        let mut dir = PathBuf::new();
        for _ in 0..depth {
            let n = rng.next_u64() as usize % 100;
            dir.push(format!("dir_{}/", n));
        }

    let name = advanced::gen_name(i, &mut rng);

        let trimmed_leading = name.trim_start().to_string();
        let safe_name = if trimmed_leading != name {
            let t = trimmed_leading;
            t
        } else {
            if name.chars().last().map(|c| c.is_whitespace()).unwrap_or(false) {
                format!("{}_", name.trim_end())
            } else {
                name
            }
        };

        let fullpath = fixtures_dir.join(&dir).join(&safe_name);

        let r = rng.next_u64() as usize % 10;
        let size = if r <= 1 {
            0usize
        } else if r <= 5 {
            10 + rng.next_u64() as usize % 200
        } else if r <= 8 {
            500 + rng.next_u64() as usize % 2000
        } else {
            10000 + rng.next_u64() as usize % 50000
        };

        create_file_of_size(&fullpath, size);
        emit(&fullpath);

        files.push(fullpath.clone());

        #[cfg(unix)]
        {
            use std::process::Command;
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
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
                let new_time = now - (days * 86400 + secs) as i64;
                let ft = FileTime::from_unix_time(new_time, 0);
                let _ = filetime::set_file_mtime(&fullpath, ft);
            }

            if rng.next_u32() % 100 < 10 {
                if std::process::Command::new("setfacl").arg("-h").status().is_ok() {
                    let user = if let Ok(out) = std::process::Command::new("id").arg("-un").output() {
                        String::from_utf8_lossy(&out.stdout).trim().to_string()
                    } else { String::from("root") };
                    let _ = std::process::Command::new("setfacl")
                        .arg("-m")
                        .arg(format!("u:{}:r--", user))
                        .arg(&fullpath)
                        .status();
                }
            }

            // centralize advanced attributes & randomization in one module so it can be tested
            // individually and reused elsewhere. The function returns any created paths
            // which should be written to the manifest.
            let created = advanced::apply_advanced_attrs(&mut rng, &files, &fullpath, &fixtures_dir);
            for c in &created {
                emit(c);
            }
        }

        count_created += 1;
        i += 1;
    }

    println!("Wrote {} entries to {}", count_created, manifest.display());
    fixtures_dir
}

pub fn apply_permissions(fixtures_dir: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file = fixtures_dir.join("file1.txt");
        if !file.exists() {
            println!("Permission target {} does not exist; skipping", file.display());
            return;
        }
        let perms = fs::Permissions::from_mode(0o644);
        if let Err(e) = fs::set_permissions(&file, perms) {
            eprintln!("Failed to set permissions for {}: {}", file.display(), e);
        } else {
            println!("Permissions set for {}", file.display());
        }
    }
    #[cfg(not(unix))]
    {
        println!("Permission setting is only supported on Unix");
    }
}
