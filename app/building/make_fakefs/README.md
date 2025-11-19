# make_fakefs.rs: Rust CLI for building Docker image and managing fake filesystem for fileZoom tests

This tool replaces all previous shell scripts in this directory. Usage:

```
cargo run --bin make_fakefs -- <command>
```

Commands:
- `build` : Build the Docker image for the fake filesystem (tag: filezoom-fakefs)
- `generate-fixtures` : Generate test fixtures in ../tests/fixtures
- `apply-permissions` : Set permissions on fixtures (Unix only)

All shell scripts have been removed. Use this Rust CLI for all related tasks.
