# Contributing

Thanks for wanting to contribute to fileZoom! Below are a few notes to help
you get started and to point you at the repository layout.

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

Getting started
---------------

- Build and test locally:

```bash
cd app
cargo build
cargo test -p fileZoom -- --nocapture
```

- Follow repository rules in `.github/instructions/rust_mc.prompt.md` when
  producing patches or PRs (this file documents repo-specific guidelines).

If you want, I can also add a short checklist for creating PRs (formatting,
running clippy, adding tests, etc.).