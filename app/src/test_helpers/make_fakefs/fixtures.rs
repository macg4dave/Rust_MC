use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
// filetime::FileTime was previously used here; advanced.rs handles filetime
// modifications now, so we no longer need this import.
use crate::advanced;
use rand::RngCore;

/// Create a temporary fixtures directory and populate it with many files used by tests.
pub fn generate_fixtures() -> PathBuf {
    let mut fixtures_dir = env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    fixtures_dir.push(format!(
        "filezoom_fixtures_{}_{}",
        std::process::id(),
        stamp
    ));

    if fixtures_dir.exists() {
        let _ = fs::remove_dir_all(&fixtures_dir);
    }
    fs::create_dir_all(&fixtures_dir).expect("failed to create fixtures dir");

    let manifest = fixtures_dir.join("fixtures_manifest.txt");
    let _ = std::fs::remove_file(&manifest);
    let mut manifest_file =
        std::fs::File::create(&manifest).expect("failed to create manifest file");

    let total: usize = 500;
    println!(
        "Generating {} fixtures under {}",
        total,
        fixtures_dir.display()
    );

    let mut emit = |p: &Path| {
        let rel = p.strip_prefix(&fixtures_dir).unwrap_or(p);
        let _ = writeln!(manifest_file, "{}", rel.to_string_lossy());
    };

    fs::create_dir_all(fixtures_dir.join("deep/level1/level2"))
        .expect("failed to create deep structure");
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
    let mut files: Vec<PathBuf> = vec![f1, f2, f3, f4];

    let create_file_of_size = |path: &Path, size: usize| {
        if let Some(dir) = path.parent() {
            // Try to create the parent directory tree. If a component along the
            // path exists as a file (NotADirectory), remove that file and retry
            // creating directories so generation can proceed.
            match fs::create_dir_all(dir) {
                Ok(_) => {}
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound
                        || e.kind() == std::io::ErrorKind::Other
                        || e.kind() == std::io::ErrorKind::NotADirectory
                    {
                        // Walk ancestors to find any path that exists and is a file,
                        // remove it, then retry.
                        for anc in dir.ancestors() {
                            if anc == Path::new("") {
                                continue;
                            }
                            if let Ok(md) = fs::metadata(anc) {
                                if md.is_file() {
                                    let _ = fs::remove_file(anc);
                                }
                            }
                        }
                        // Retry creating dirs; ignore error on second attempt to let
                        // the following file creation fail with a clearer message.
                        let _ = fs::create_dir_all(dir);
                    } else {
                        // Non-directory error; ignore here and let file creation fail
                        // with a clear panic message below.
                        let _ = fs::create_dir_all(dir);
                    }
                }
            }
        }
        if size == 0 {
            if let Err(e) = fs::File::create(path) {
                eprintln!(
                    "failed to create empty file {:?} parent={:?}: {}",
                    path,
                    path.parent(),
                    e
                );
                panic!("failed to create empty file: {}", e);
            }
            return;
        }
        let mut f = match fs::File::create(path) {
            Ok(h) => h,
            Err(e) => {
                eprintln!(
                    "failed to create file {:?} parent={:?}: {}",
                    path,
                    path.parent(),
                    e
                );
                panic!("failed to create file: {}", e);
            }
        };
        let block = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-+_";
        let mut written = 0usize;
        while written < size {
            let to_write = std::cmp::min(block.len(), size - written);
            f.write_all(&block[..to_write])
                .expect("failed to write block");
            written += to_write;
        }
        let _ = f.set_len(size as u64);
    };

    #[allow(deprecated)]
    let mut rng = rand::thread_rng();
    let mut i = 0usize;

    fn sanitize_name(name: &str) -> String {
        // Force ASCII-only filenames: allow ASCII alphanumerics, dot, dash, underscore.
        // Replace spaces and all non-ASCII or otherwise-problematic characters with '_'.
        let mut out = String::with_capacity(name.len());
        for c in name.chars() {
            if c.is_ascii() {
                match c {
                    '/' | '\0' => out.push('_'),
                    ' ' | '\t' | '\n' | '\r' => out.push('_'),
                    '\'' | '"' | '`' | '\\' | '*' | '?' | '<' | '>' | '|' | '&' | ';' | '$'
                    | '(' | ')' | '{' | '}' | '[' | ']' | ':' | ',' => out.push('_'),
                    '%' | '#' | '@' | '+' => out.push('_'),
                    c if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' => {
                        out.push(c)
                    }
                    _ => out.push('_'),
                }
            } else {
                // Non-ASCII -> replace with underscore to ensure ASCII-only names
                out.push('_');
            }
        }
        let trimmed = out.trim_matches('_').to_string();
        if trimmed.is_empty() {
            "file_invalid".to_string()
        } else {
            trimmed
        }
    }

    while count_created < total {
        // Build a directory path with a mix of ASCII and occasional
        // multilingual components. We keep both a sanitized (ASCII-only)
        // path and a native path so we can create both variants.
        let depth = rng.next_u64() as usize % 8;
        let mut dir_sanitized = PathBuf::new();
        let mut dir_native = PathBuf::new();
        let mut native_used = false;
        for d_idx in 0..depth {
            let n = rng.next_u64() as usize % 100;
            if (rng.next_u32() % 100) < 30 {
                // multilingual component
                let comp_raw = advanced::gen_name(i + d_idx, &mut rng);
                let comp_safe = sanitize_name(&comp_raw);
                dir_sanitized.push(format!("d__{}", comp_safe));
                dir_native.push(format!("d__{}", comp_raw));
                native_used = true;
            } else {
                let comp = format!("d__dir_{}", n);
                dir_sanitized.push(comp.clone());
                dir_native.push(comp.clone());
            }
        }

        let name = advanced::gen_name(i, &mut rng);
        let safe_name = sanitize_name(&name);

        let fullpath = fixtures_dir.join(&dir_sanitized).join(&safe_name);

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
        count_created += 1;

        // Optionally create a native-name variant in the native directory
        if native_used || (rng.next_u32() % 100) < 50 {
            let native_name: String = name
                .chars()
                .map(|c| if c == '/' || c == '\0' { '_' } else { c })
                .collect();
            let native_path = fixtures_dir.join(&dir_native).join(&native_name);
            if native_path != fullpath {
                create_file_of_size(&native_path, size);
                emit(&native_path);
                files.push(native_path);
                count_created += 1;
            }
        }

        // Create a nested subtree with variable size to exercise directory trees
        // of different shapes. Some iterations will create deeper trees with
        // many files; others will be shallow.
        if (rng.next_u32() % 100) < 40 {
            let tree_depth = 1 + (rng.next_u32() as usize % 5);
            let mut base = fixtures_dir.join(&dir_sanitized);
            for td in 0..tree_depth {
                let branch_count = 1 + (rng.next_u32() as usize % 6);
                for b in 0..branch_count {
                    let subdir_name = if (rng.next_u32() % 100) < 25 {
                        // multilingual directory under the subtree
                        let raw = advanced::gen_name(i + td + b, &mut rng);
                        format!("d__{}", sanitize_name(&raw))
                    } else {
                        let num = rng.next_u32() as usize % 1000;
                        format!("d__sub_{}", num)
                    };
                    base.push(&subdir_name);
                    // create a few files inside this subdir
                    let files_here = 1 + (rng.next_u32() as usize % 8);
                    for fh in 0..files_here {
                        let fname = advanced::gen_name(i + td + b + fh, &mut rng);
                        let f_safe = sanitize_name(&fname);
                        let p = base.join(&f_safe);
                        let sz = 1 + (rng.next_u64() as usize % 4096);
                        create_file_of_size(&p, sz);
                        emit(&p);
                        files.push(p);
                        count_created += 1;
                    }
                    // pop the subdir component to continue loops
                    base.pop();
                }
                // go one level deeper for next td
            }
        }

        i += 1;
    }

    // Apply advanced attributes across all generated files so symlinks, FIFOs,
    // ACLs and xattrs are created and added to the manifest.
    {
        let mut created_any: Vec<PathBuf> = Vec::new();
        for f in &files {
            let extra = advanced::apply_advanced_attrs(&mut rng, &files, f, &fixtures_dir);
            for c in &extra {
                emit(c);
                created_any.push(c.clone());
            }
        }
        files.extend(created_any);
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
            println!(
                "Permission target {} does not exist; skipping",
                file.display()
            );
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
