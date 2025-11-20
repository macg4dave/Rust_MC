# Testing and `make_fakefs` helper

This document explains how to use the `make_fakefs` helper and the opt-in Docker
integration test used to exercise `fileZoom` against large, varied filesystem
fixtures.

Prerequisites

- Rust toolchain (cargo)
- Docker daemon available and running

Generate fixtures and run locally

1. From the repository `app` directory, run the helper in the current terminal (foreground):

```bash
cd app
cargo run -p fileZoom --bin make_fakefs -- run --foreground
```

This will:

- Generate a temporary fixtures directory (printed to stdout) with many files and
  directories, including multilingual name variants and nested trees.
- Build a Docker image `filezoom-fakefs` if needed (the helper prefers a host
  release binary; if incompatible it builds one inside a temporary multi-stage
  Dockerfile).
- Create a Docker volume populated from the image, then run a container with the
  fixtures mounted so the container sees only the generated fixtures.

Run the helper and open a terminal window

To ask the helper to open the app in a new host terminal window (macOS/Linux
GUI), omit `--foreground` and set `ATTACH_TERMINAL=1` when invoking the helper or
when running the test. Example:

```bash
cd app
ATTACH_TERMINAL=1 cargo run -p fileZoom --bin make_fakefs -- run
```

Opt-in Docker integration test

The repository provides an ignored integration test `docker_fakefs_run` which
builds the image and runs `fileZoom` in a container backed by the generated
fixtures. This test is opt-in and runs only when enabled via environment
variables (to avoid accidental long-running Docker operations during normal
test runs):

```bash
cd app
ATTACH_TERMINAL=1 RUN_DOCKER_FAKEFS=1 cargo test -p fileZoom docker_fakefs_run -- --nocapture --ignored
```

- `RUN_DOCKER_FAKEFS=1` enables the test.
- `ATTACH_TERMINAL=1` requests opening a new host terminal window for the GUI
  run. If not set, the container will run in the current terminal when `--foreground` is used.

Inspecting generated fixtures

After running the helper it prints the temporary fixtures directory path and
creates a `fixtures_manifest.txt` file inside it listing the generated entries. To
inspect:

```bash
ls -la /path/to/filezoom_fixtures_<stamp>
less /path/to/filezoom_fixtures_<stamp>/fixtures_manifest.txt
```

Cleanup

- The helper creates and later removes the Docker volume it uses by default. If a
  volume remains (e.g., due to an interrupted run), remove it manually:

```bash
docker volume rm filezoom_fixtures_<stamp>
```

## Terminal safety during tests

- Many tests and helper runs spawn the TUI `fileZoom` binary. When running
  terminal-based UI code in tests, it's important to ensure the terminal is
  always restored (leave alternate screen, disable raw mode, disable mouse
  capture) even on panic or early exit. Tests that fail to restore the
  terminal can leave the developer shell in an unusable state.
- The codebase now provides a `TerminalGuard` RAII helper that ensures
  terminal restoration on drop; tests or helpers that initialize the
  terminal should prefer `init_terminal()` which returns this guard. Where
  explicit restoration is desired, call `restore_terminal(guard)` to report
  errors instead of relying solely on Drop.


Adjusting generator behavior

If you want to change fixture defaults (counts, multilingual variance, depth), I
can modify `app/src/test_helpers/make_fakefs/fixtures.rs` to use different defaults
or add CLI flags to the `make_fakefs` binary to parameterize generation.
