Copilot / Agent Instructions for fileZoom (Improved)

This repository contains a Rust TUI file manager (fileZoom). These
instructions give AI agents precise, actionable guidance.

------------------------------------------------------------------------

🔥 Critical Rules (Copilot reads these first)

-   Always run cargo build and full tests before and after changes.
-   No warnings allowed: fix all Rust, Clippy, and test warnings/errors.
-   Make the smallest, safest patch that solves the task.
-   Preserve all public CLI flags, machine output, and documented
    behaviour unless explicitly authorised.
-   Any behavioural change MUST include updated or new tests.
-   Never introduce unsafe unless absolutely necessary and justified
    with comments + tests.
-   Do not remove modules or functions unless tests confirm they’re
    obsolete.
    - Do not remove crates from Cargo.toml unless confirmed unused.
    do not remove dependencies without checking for usage.
    do not remove tests unless they are obsolete or redundant.
-   No unwrap() except in trivial test scaffolding.
-   Keep code idiomatic, modular, and well-documented.
- do not remove crate uses without checking for usage.
-   use ratatui and crossterm idioms and patterns where applicable.
- use ratatui layout and widget systems properly.
- ratatui event handling and rendering patterns must be followed.
-ratatui styling and theming conventions must be used.
- crossterm terminal control and input handling must be correct.
- crossterm async and sync patterns must be followed.
- crossterm error handling must be robust.
- crossterm performance best practices must be observed.
- crossterm cross-platform compatibility must be ensured.
- ratatui and crossterm version compatibility must be maintained.
- ratatui and crossterm community conventions must be respected.
- ratatui and crossterm security best practices must be followed.
- ratatui and crossterm testing patterns must be used.
- ratatui and crossterm documentation standards must be upheld.
- ratatui and crossterm code organization must be logical.
- ratatui and crossterm dependency management must be careful.
- ratatui and crossterm resource management must be efficient.
- ratatui and crossterm concurrency patterns must be correct.
- ratatui and crossterm error propagation must be proper.
- ratatui and crossterm logging practices must be followed.
- ratatui and crossterm configuration management must be sound.
- ratatui and crossterm usability best practices must be observed.
- early version don't worry about Backwards compatibility
------------------------------------------------------------------------

📦 Project Overview

-   Rust-only TUI file manager using Ratatui and Crossterm.
-   Single binary in app/ crate.
-   Goal: small, dependency-light, highly usable cross-platform terminal
    file manager with mouse support.

------------------------------------------------------------------------

📁 Key Paths

Important paths:
- Crate root: `app/` (fileZoom binary crate).
- Core entrypoints: `app/src/lib.rs`, `app/src/app.rs`, `app/src/main.rs`.
- App internals: `app/src/app/` (contains `core/`, `types.rs`, `path.rs`, `settings/`).
- UI code: `app/src/UI/` (menu, modal, panels, dialogs, util).
- Runner/handlers: `app/src/runner/` (commands, event_loop_main, handlers/).
- Filesystem ops: `app/src/fs_op/` (copy, mv, stat, permissions, path helpers).
- Input handling: `app/src/input/` (keyboard, mouse).
- Virtual FS and network backends: `app/src/vfs/`.
- Errors and localization: `app/src/errors/`.
- Building helpers and scripts: `app/building/`, `app/building/make_fakefs/`, `app/scripts/`.
- Test helpers and fixtures: `app/test_helper/`, `app/src/test_helpers/`, and integration tests in `app/tests/` and top-level `tests/`.
- Docker and packaging: `app/docker/`.

------------------------------------------------------------------------

🛠 Build / Test / Run

Preferred environment: Linux, macOS, or Windows 11 via WSL2.

Run everything manually:

    cd app
    cargo build
    cargo test -p fileZoom -- --nocapture
    cargo run

------------------------------------------------------------------------

🧭 Coding Standards (Rust-specific)

-   Follow idiomatic Rust:
    -   snake_case identifiers
    -   strong typing
    -   use ? instead of unwrap
    -   minimise cloning
    -   prefer enums over booleans
-   Document intent, not the obvious.
-   Add Rustdoc for every public type, function, or module.
-   Keep functions focused and modular (single responsibility).
-   Avoid monolithic files; split logically when needed.
-   Maintain clarity over cleverness.
-   All .rs files must be less than 800 chars long.
------------------------------------------------------------------------

📚 Tests

-   All tests live in app/tests/.
-   Integration tests must use temp directories and fixtures to avoid
    touching the real filesystem.
-   Any new behaviour → add tests.
-   Any refactor that changes behaviour → update tests.
-   Include failure cases, edge cases, and correct-path cases.

------------------------------------------------------------------------

🤖 Agent Behaviour (What Copilot must do)

-   Make minimal, correct patches that keep the entire suite passing.
-   When asked for a task, list:
    -   what modules are affected
    -   what tests must be updated
    -   what behaviour must remain untouched
-   Before modifying code, inspect:
    app/src/lib.rs, app/src/app.rs, app/src/main.rs, app/src/ui/mod.rs,
    app/tests/integration_tests.rs, and helper scripts.

------------------------------------------------------------------------

🔒 Security & External References

-   Never assume private/internal resources.
-   If external info is needed, request it explicitly.

------------------------------------------------------------------------

📝 Documentation Requirements

-   Update README for user-facing changes.
-   Add CHANGELOG entries for every feature, fix, or behavioural change.
-   Provide examples in Rustdoc when helpful.

------------------------------------------------------------------------

🧩 Prompt Files

-   Path-specific instructions live in .github/instructions/.
-   Use these for contextual understanding of edits in that directory.

------------------------------------------------------------------------

🚫 Limitations

-   Copilot Code Review sees only ~4,000 chars of this file.
-   Keep critical rules (tests, minimal patching, public API stability)
    at the top.

------------------------------------------------------------------------

✔ Hard Constraints for Every PR / Patch

1.  Run full build + tests before and after changes.
2.  Zero warnings (Rustc + Clippy).
3.  Only the smallest required patch.
4.  Add/update tests for any behavioural modifications.
5.  No API-breaking changes unless explicitly authorised.
6.  No deprecated or dead code allowed.
7.  Keep the repository modular, documented, and clean.
