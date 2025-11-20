use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Copy a directory recursively from `src` to `dst`.
pub fn copy_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "source not found",
        ));
    }
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_recursive(&from, &to)?;
        } else if file_type.is_file() {
            fs::copy(&from, &to)?;
        } else if file_type.is_symlink() {
            if let Ok(target) = fs::read_link(&from) {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(&target, &to)?;
                }
                #[cfg(not(unix))]
                {
                    if target.is_file() {
                        fs::copy(&target, &to)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Build a Docker image. If `fixtures` is Some, this creates a temporary
/// build context, copies the current directory into it, copies fixtures into
/// `tests/fixtures` inside the context, and runs `docker build` using that
/// context. This function invokes external commands directly (cargo, docker).
pub fn build_image_with_fixtures(fixtures: Option<&Path>, current_dir: &Path) -> Result<()> {
    if let Some(fixtures_dir) = fixtures {
        let mut build_ctx = std::env::temp_dir();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        build_ctx.push(format!(
            "filezoom_build_ctx_{}_{}",
            std::process::id(),
            stamp
        ));

        if build_ctx.exists() {
            let _ = fs::remove_dir_all(&build_ctx);
        }
        fs::create_dir_all(&build_ctx)?;

        // Build release binary in current_dir so artifacts exist in the build
        // context if required.
        let status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(current_dir)
            .status()
            .context("Failed to run cargo build --release")?;
        if !status.success() {
            return Err(anyhow::anyhow!("cargo build --release failed"));
        }

        // Copy the repository root into the build context so paths inside the
        // image match expectations (the Dockerfile expects an `app/` folder).
        let repo_root = if current_dir.file_name().map(|s| s == "app").unwrap_or(false) {
            current_dir.parent().unwrap_or(current_dir)
        } else {
            current_dir
        };

        copy_recursive(repo_root, &build_ctx)
            .context("failed to copy repository into build context")?;

        let target_fixtures = build_ctx.join("tests").join("fixtures");
        fs::create_dir_all(&target_fixtures)?;
        copy_recursive(fixtures_dir, &target_fixtures)
            .context("failed to copy fixtures into build context")?;

        // Ensure the release binary exists in the copied build context at the
        // path the Dockerfile expects (app/target/release/fileZoom). If it's
        // missing, try to run `cargo build --release` in the crate directory
        // that contains `app/` so the artifact is produced. If that still
        // fails, return an explicit error so callers can act.
        let expected_bin = build_ctx
            .join("app")
            .join("target")
            .join("release")
            .join("fileZoom");
        if !expected_bin.exists() {
            // Determine where to run cargo build: prefer repo_root/app if it
            // exists, otherwise fall back to current_dir.
            let crate_dir = if build_ctx.join("app").exists() {
                build_ctx.join("app")
            } else {
                build_ctx.clone()
            };
            eprintln!("Release binary not found at {}. Running `cargo build --release` in {} to produce it...",
                expected_bin.display(), crate_dir.display());

            let status = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(&crate_dir)
                .status()
                .context("Failed to run cargo build --release")?;
            if !status.success() {
                return Err(anyhow::anyhow!(
                    "cargo build --release failed while producing release binary at {}",
                    expected_bin.display()
                ));
            }

            // Sometimes cargo writes workspace-level targets to `target/release`
            // at the build context root rather than under `app/target`.
            // If that happened, move the built binary into the path the Dockerfile
            // expects (`app/target/release/fileZoom`).
            if !expected_bin.exists() {
                let alt = build_ctx.join("target").join("release").join("fileZoom");
                if alt.exists() {
                    if let Some(parent) = expected_bin.parent() {
                        fs::create_dir_all(parent)
                            .context("creating parent dir for expected binary")?;
                    }
                    fs::copy(&alt, &expected_bin).context(
                        "copying workspace-level built binary into expected app/target path",
                    )?;
                }
            }

            if !expected_bin.exists() {
                return Err(anyhow::anyhow!(
                    "Release binary still missing after build; expected at {}",
                    expected_bin.display()
                ));
            }
        }

        // Choose the Dockerfile path relative to the copied build context.
        let default_dockerfile = if build_ctx.join("docker").join("Dockerfile").exists() {
            "docker/Dockerfile".to_string()
        } else if build_ctx
            .join("app")
            .join("docker")
            .join("Dockerfile")
            .exists()
        {
            "app/docker/Dockerfile".to_string()
        } else {
            // Fallback to docker/Dockerfile; Docker will error if missing.
            "docker/Dockerfile".to_string()
        };

        // If the expected binary is present and looks like an ELF (Linux) binary,
        // use the existing Dockerfile. Otherwise, generate a temporary
        // multi-stage Dockerfile in the build context that builds the release
        // binary inside the builder image so the runtime image contains a
        // compatible Linux executable. This leaves the repo's Dockerfile
        // unchanged.
        let expected_bin = build_ctx
            .join("app")
            .join("target")
            .join("release")
            .join("fileZoom");
        let use_dockerfile = if expected_bin.exists() {
            // quick ELF magic check
            match std::fs::File::open(&expected_bin) {
                Ok(mut f) => {
                    let mut magic = [0u8; 4];
                    if f.read_exact(&mut magic).is_ok() {
                        magic == [0x7f, b'E', b'L', b'F']
                    } else {
                        false
                    }
                }
                Err(_) => false,
            }
        } else {
            false
        };

        let dockerfile_rel = if use_dockerfile {
            default_dockerfile
        } else {
            // Create a temporary multi-stage Dockerfile inside build_ctx
            let temp_path = build_ctx.join("Dockerfile.multistage");
            let mut file =
                fs::File::create(&temp_path).context("failed to create temp Dockerfile")?;
            let content = r#"FROM rust:1 AS builder
WORKDIR /work
COPY . /work
WORKDIR /work/app
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM debian:stable-slim AS runtime
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates attr file tzdata \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /work
COPY --from=builder /work /work
RUN if [ -x /work/app/scripts/apply_permissions_in_image.sh ]; then /bin/bash /work/app/scripts/apply_permissions_in_image.sh; fi
WORKDIR /work/app
RUN chmod +x scripts/*.sh || true
CMD ["/work/target/release/fileZoom"]
"#;
            file.write_all(content.as_bytes())?;
            // Use the temporary Dockerfile path relative to the build context
            "Dockerfile.multistage".to_string()
        };

        let status = Command::new("docker")
            .current_dir(&build_ctx)
            .args(["build", "-f", &dockerfile_rel, "-t", "filezoom-fakefs", "."])
            .status()
            .context("Failed to run docker build")?;

        // If we created a temporary Dockerfile in the build context, remove it
        // now that the build finished to avoid leaving artifacts behind.
        if !use_dockerfile {
            let tmp = build_ctx.join("Dockerfile.multistage");
            let _ = fs::remove_file(&tmp);
        }

        let _ = fs::remove_dir_all(&build_ctx);

        if !status.success() {
            return Err(anyhow::anyhow!("Docker build failed"));
        }
        Ok(())
    } else {
        // Build using the Dockerfile located at `docker/Dockerfile` inside the
        // crate so running from the `app/` directory works consistently.
        // If the Dockerfile is under `app/docker` (common in this repo), use
        // that path; otherwise try `docker/Dockerfile`.
        // Before building, ensure the release binary exists at one of the
        // expected locations. If missing, attempt to run `cargo build
        // --release` in a reasonable crate directory to produce it, and
        // fail with a clear error if that still doesn't produce the binary.
        let candidate1 = current_dir.join("target").join("release").join("fileZoom");
        let candidate2 = current_dir
            .join("app")
            .join("target")
            .join("release")
            .join("fileZoom");
        if !candidate1.exists() && !candidate2.exists() {
            // Decide where to run cargo build: prefer current_dir if it has a Cargo.toml,
            // otherwise try current_dir/app if present.
            let build_dir = if current_dir.join("Cargo.toml").exists() {
                current_dir.to_path_buf()
            } else if current_dir.join("app").join("Cargo.toml").exists() {
                current_dir.join("app")
            } else {
                current_dir.to_path_buf()
            };
            eprintln!("Release binary not found in expected locations. Running `cargo build --release` in {}...", build_dir.display());
            let status = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(&build_dir)
                .status()
                .context("Failed to run cargo build --release")?;
            if !status.success() {
                return Err(anyhow::anyhow!(
                    "cargo build --release failed while producing release binary"
                ));
            }
            // Re-check
            if !candidate1.exists() && !candidate2.exists() {
                return Err(anyhow::anyhow!(
                    "Release binary not found after build; expected at {} or {}",
                    candidate1.display(),
                    candidate2.display()
                ));
            }
        }

        let default_dockerfile = if current_dir.join("docker").join("Dockerfile").exists() {
            "docker/Dockerfile".to_string()
        } else if current_dir
            .join("app")
            .join("docker")
            .join("Dockerfile")
            .exists()
        {
            "app/docker/Dockerfile".to_string()
        } else if current_dir.join("../app/docker/Dockerfile").exists() {
            "../app/docker/Dockerfile".to_string()
        } else {
            "docker/Dockerfile".to_string()
        };

        // Check candidate release binaries for a Linux ELF. Prefer workspace-level
        // target first, then crate-level.
        let candidate1 = current_dir.join("target").join("release").join("fileZoom");
        let candidate2 = current_dir
            .join("app")
            .join("target")
            .join("release")
            .join("fileZoom");
        let mut use_dockerfile = false;
        let mut expected_bin = None;
        if candidate1.exists() {
            expected_bin = Some(candidate1);
        } else if candidate2.exists() {
            expected_bin = Some(candidate2);
        }
        if let Some(p) = &expected_bin {
            if let Ok(mut f) = fs::File::open(p) {
                let mut magic = [0u8; 4];
                if f.read_exact(&mut magic).is_ok() && magic == [0x7f, b'E', b'L', b'F'] {
                    use_dockerfile = true;
                }
            }
        }

        let dockerfile_to_use = if use_dockerfile {
            default_dockerfile
        } else {
            // Write a temporary multi-stage Dockerfile in `current_dir` and use it.
            let temp_path = current_dir.join("Dockerfile.multistage");
            let mut file =
                fs::File::create(&temp_path).context("failed to create temp Dockerfile")?;
            let content = r#"FROM rust:1 AS builder
WORKDIR /work
COPY . /work
WORKDIR /work/app
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM debian:stable-slim AS runtime
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates attr file tzdata \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /work
COPY --from=builder /work /work
RUN if [ -x /work/app/scripts/apply_permissions_in_image.sh ]; then /bin/bash /work/app/scripts/apply_permissions_in_image.sh; fi
WORKDIR /work/app
RUN chmod +x scripts/*.sh || true
CMD ["/work/target/release/fileZoom"]
"#;
            file.write_all(content.as_bytes())?;
            "Dockerfile.multistage".to_string()
        };

        let status = Command::new("docker")
            .current_dir(current_dir)
            .args([
                "build",
                "-f",
                &dockerfile_to_use,
                "-t",
                "filezoom-fakefs",
                ".",
            ])
            .status()
            .context("Failed to run docker build")?;

        // Remove the temporary Dockerfile if we wrote one into `current_dir`.
        if !use_dockerfile {
            let _ = fs::remove_file(current_dir.join("Dockerfile.multistage"));
        }
        if !status.success() {
            return Err(anyhow::anyhow!("Docker build failed"));
        }
        Ok(())
    }
}
