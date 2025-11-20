**Overview**

This repository contains `fileZoom`, a minimal Rust TUI file manager used for development
and integration testing. It includes a small helper binary, `make_fakefs`, which generates
large, varied filesystem fixtures and can build/run a Docker image that mounts those
fixtures into a container for isolated, repeatable tests.

**Quick Usage: `make_fakefs`**

- **Build & run the helper (foreground)**: from the repository root run:

# fileZoom — Quick README

Overview

This repository contains `fileZoom`, a compact Rust TUI file manager used for
development and integration testing. It includes a helper binary, `make_fakefs`,
that generates large, diverse filesystem fixtures and can build/run a Docker image
which mounts those fixtures into a container for isolated tests.

Quick usage (helper)

- Build & run the helper in the current terminal (foreground):

```bash
cd app
cargo run -p fileZoom --bin make_fakefs -- run --foreground
```

- What `make_fakefs` does by default:
  - Generates many fixture files under a temporary directory (the path is printed).
  - Builds a Docker image named `filezoom-fakefs` (a multi-stage build is used when
    a host-compatible release binary is not available).
  - Creates a Docker volume populated from the image and mounts it into the
    container under `/work/tests` so the container only sees the fixtures.

- To open the app in a new host terminal window (macOS `osascript` / common
  Linux terminals), run without `--foreground` and set `ATTACH_TERMINAL=1` or use
  `--terminal NAME` to pick a terminal program.

Opt-in integration test

There is an ignored integration test `docker_fakefs_run` that builds the image and
runs `fileZoom` inside a container attached to an isolated fixtures volume. The
test is intentionally opt-in so it only runs when you explicitly enable it:

```bash
cd app
ATTACH_TERMINAL=1 RUN_DOCKER_FAKEFS=1 cargo test -p fileZoom docker_fakefs_run -- --nocapture --ignored
```

- `RUN_DOCKER_FAKEFS=1` enables the test. `ATTACH_TERMINAL=1` asks the helper to
  open the app in a new host terminal window; omit `ATTACH_TERMINAL` to run in the
  current terminal (use `--foreground`).

Prerequisites & notes

- Docker daemon is required for builds and runs.
- On macOS the helper uses AppleScript (`osascript`) to open Terminal/iTerm; this
  behavior may vary. If `osascript` is unavailable, the helper tries common Linux
  terminals or runs in the current terminal.
- If the host release binary is incompatible with the container (e.g., macOS vs
  Linux), the helper generates a temporary multi-stage Dockerfile and builds a
  Linux release binary inside the image.

If you want different fixture defaults (counts, multilingual probability, tree
depth/branching), tell me the desired values and I will update the generator.

UI notes
-------

The TUI is implemented using Ratatui widgets (List, Paragraph, Scrollbar, etc.).
To visually verify the scrollbars, open `fileZoom` in a directory with many
files or run the included `make_fakefs` helper to generate a fixture directory
and then launch `fileZoom`:

```bash
cd app
cargo run --release
```

Use the arrow keys or page keys to scroll and confirm the vertical scrollbars
appear at the right hand side of panels and in the preview.

Top menu usage
--------------

The top menu (`File`, `Copy`, `Move`, `New`, `Sort`, `Help`) is interactive. Press
`F1` to focus the menu, then use Left/Right arrow keys to select a menu item and
`Enter` to activate it (currently the activation opens a simple `Message` box).
