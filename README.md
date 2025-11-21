**Overview**

This repository contains `fileZoom`, a minimal Rust TUI file manager used for development
and integration testing. It includes a small helper binary, `make_fakefs`, which generates
large, varied filesystem fixtures and can build/run a Docker image that mounts those
fixtures into a container for isolated, repeatable tests.

Important paths
---------------

- Crate root: `app/` (fileZoom binary crate).
- Core entrypoints: `app/src/lib.rs`, `app/src/app.rs`, `app/src/main.rs`.
- App internals: `app/src/app/` (contains `core/`, `types.rs`, `path.rs`, `settings/`).
- UI code: `app/src/ui/` (menu, modal, panels, dialogs, util).
- Runner/handlers: `app/src/runner/` (commands, `event_loop_main.rs`, `handlers/`).
- Filesystem ops: `app/src/fs_op/` (copy, mv, stat, permissions, path helpers).
- Input handling: `app/src/input/` (keyboard, mouse).
- Virtual FS and network backends: `app/src/vfs/`.
- Errors and localization: `app/src/errors/`.
- Building helpers and scripts: `app/building/`, `app/building/make_fakefs/`, `app/scripts/`.
- Test helpers and fixtures: `app/test_helper/`, `app/src/test_helpers/`, and integration tests in `app/tests/` and top-level `tests/`.
- Docker and packaging: `app/docker/`.

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

Filesystem copy behavior
------------------------

- The project uses the `fs_extra` crate for recursive and batch copy
  operations where appropriate to improve performance and simplify the
  implementation. Single-file copies use an atomic temporary-file+rename
  helper so other processes never observe partially-written files.
- After copy operations `fileZoom` attempts to preserve permission bits and
  file timestamps (best-effort). By default files are not overwritten when
  copying into destinations that already exist (`overwrite = false`).
- If you need exact platform-specific ownership preservation (UID/GID), the
  code intentionally does not modify ownership to avoid portability issues.

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

CLI Usage
---------

You can start `fileZoom` with a few helpful CLI flags that override persisted
settings for the current run. CLI-provided values take precedence over saved
settings.

Examples:
````markdown
**Overview**

This repository contains `fileZoom`, a minimal Rust TUI file manager used for development
and integration testing. It includes a small helper binary, `make_fakefs`, which generates
large, varied filesystem fixtures and can build/run a Docker image that mounts those
fixtures into a container for isolated, repeatable tests.

Important paths
---------------

- Crate root: `app/` (fileZoom binary crate).
- Core entrypoints: `app/src/lib.rs`, `app/src/app.rs`, `app/src/main.rs`.
- App internals: `app/src/app/` (contains `core/`, `types.rs`, `path.rs`, `settings/`).
- UI code: `app/src/ui/` (menu, modal, panels, dialogs, util).
- Runner/handlers: `app/src/runner/` (commands, `event_loop_main.rs`, `handlers/`).
- Filesystem ops: `app/src/fs_op/` (copy, mv, stat, permissions, path helpers).
- Input handling: `app/src/input/` (keyboard, mouse).
- Virtual FS and network backends: `app/src/vfs/`.
- Errors and localization: `app/src/errors/`.
- Building helpers and scripts: `app/building/`, `app/building/make_fakefs/`, `app/scripts/`.
- Test helpers and fixtures: `app/test_helper/`, `app/src/test_helpers/`, and integration tests in `app/tests/` and top-level `tests/`.
- Docker and packaging: `app/docker/`.

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

CLI Usage
---------

You can start `fileZoom` with a few helpful CLI flags that override persisted
settings for the current run. CLI-provided values take precedence over saved
settings.

Examples:

- Start in `/tmp` and show hidden files:

```bash
cd app
cargo run -- --dir /tmp --show-hidden
```

- Start with the dark theme and increased verbosity:

```bash
cd app
cargo run -- --theme dark -vv
```

- Disable mouse capture at startup (you can toggle it later from Settings):

```bash
cd app
cargo run -- --no-mouse
```

Notes:

- `--theme` accepts `default` or `dark` (case-sensitive). If omitted the
  persisted theme or the built-in default is used.
- The `-v`/`--verbose` flag can be passed multiple times to increase logging
verbosity: `-v` (info), `-vv` (debug), `-vvv` (trace).
- CLI flags only affect the current run; use the in-app Settings menu to
  persist changes to disk.

Top menu usage
--------------

The top menu (`File`, `Copy`, `Move`, `New`, `Sort`, `Help`) is interactive. Press
`F1` to focus the menu, then use Left/Right arrow keys to select a menu item and
`Enter` to activate it (currently the activation opens a simple `Message` box).

Keybindings
-----------

`fileZoom` supports user-remappable keybindings via a simple `keybinds.xml` file.

- **Placement**: create `keybinds.xml` at either:
  - `$XDG_CONFIG_HOME/fileZoom/keybinds.xml` (recommended)
  - `./keybinds.xml` in the current working directory (fallback)

- **Examples shipped**:
  - `doc/keybinds_example.xml` — extended example and usage notes
  - `app/keybinds.xml.example` — minimal example placed next to the app

- **Format**: a very small XML with `bind` entries:

```
<keybinds>
  <bind action="enter">Enter</bind>
  <bind action="quit">q</bind>
</keybinds>
```

- **KEY tokens**: single characters (`q`, `.`), named keys (`Enter`, `Esc`, `Up`, `Down`, `Tab`, `Space`), and modifiers like `Ctrl+q`.

Keybindings are loaded at startup; edit the file and restart `fileZoom` to apply changes.
