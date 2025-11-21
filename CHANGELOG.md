# Changelog

## Unreleased

- Add Command Line and Menu Bar integration
  - New `CommandLineState` in `App` and `ui::command_line` drawing/handler.
  - Simple command registry (`app::runner::commands::execute_command`) supporting `toggle-preview`, `menu-next`, `menu-prev`, and `menu-activate`.
  - Menu bar now activates on left-click (mouse handler updated).
  - Integration tests added: `ui_menu_interaction.rs` and `menu_commandline_feature.rs`.
  - Minor wiring in `ui::mod` and `runner` handlers to route keys and mouse events.

- Replace manual recursive filesystem walks with `walkdir` where appropriate
  (improves robustness and reduces manual recursion code). Files updated:
  - `app/src/fs_op/copy.rs`
  - `app/src/fs_op/mv.rs`
  - `app/src/building/make_fakefs_lib.rs`

- Adopt `fs_extra` for file/directory copy operations and add metadata
  preservation hooks:
  - Use `fs_extra` for recursive and batch copies where appropriate to
    improve throughput and simplify implementation.
  - Preserve permissions and timestamps (best-effort) after copies to
    better retain source metadata; atomic single-file copies still use
    the project's `atomic_copy_file` helper to avoid exposing partially
    written files.
  - `CopyOptions` tuned for the project: 64 KiB buffer and `overwrite = false`.

- Filesystem watching (optional):
  - Add an optional feature `fs-watch` (gated behind Cargo features) which
    enables filesystem watching via the `notify` crate.
  - The watcher runs in a background thread and sends `FsEvent` messages
    into the runner; the event loop now maps events to affected panel(s)
    and performs a per-panel refresh rather than always refreshing both
    panels. This reduces unnecessary work and improves responsiveness.
  - Watcher behavior is recursive by default (subdirectories are observed).
  - Implementation: `app/src/fs_op/watcher.rs`, runner wiring in
    `app/src/runner/event_loop_main.rs`, and an integration test
    `app/tests/fs_watch.rs`.

  - Refactor: `app::core` module cleanup
    - Consolidated and documented `app/src/app/core/mod.rs`:
      - Introduced clear type aliases for background operation channels and
        cancel flags (`OpProgressReceiver`, `OpCancelFlag`, `OpDecisionSender`).
      - Consolidated duplicate preview-size constants into a single
        canonical `MAX_PREVIEW_BYTES` (100 KiB) while preserving the
        legacy `App::MAX_PREVIEW_BYTES` associated constant for
        compatibility.
      - Improved comments and Rustdoc for `App` and its helper accessors,
        simplified `selected_index` logic, and removed dead/duplicate code.
      - Adjusted helper visibility and kept compatibility with existing
        tests and modules.
    - Tests: added a small unit check for `MAX_PREVIEW_BYTES`; full
      behaviour remains covered by existing integration tests.

  - Refactor: navigation helpers and API rename
    - Refactored `app/src/app/core/navigation.rs` to centralise post-navigation
      behaviour into a private helper (`apply_navigation`) and reduce code
      duplication when updating panel selection and preview state.
    - Renamed public `App` navigation methods to clearer identifiers:
      - `next` -> `select_next`
      - `previous` -> `select_prev`
      - `page_down` -> `select_page_down`
      - `page_up` -> `select_page_up`
    - Updated internal call sites (runner handlers and tests) to use the
      new names. All repository tests pass after these changes.
    - Consider adding deprecated shims for the old names if external users
      require backwards compatibility.

- Tidy: Refactor preview helpers and small core modules
  - Reworked preview helpers in `app/src/app/core/preview.rs`:
    - Implemented clearer `is_binary` heuristic (NUL detection, UTF-8 checks,
      and a non-printable character ratio threshold).
    - `build_file_preview` now reads a bounded sample, strips UTF-8 BOM,
      and reports truncated previews when sampling smaller than file size.
    - `build_directory_preview` uses `std::fs::read_dir` for a shallow
      one-level listing to reduce allocations and avoid unnecessary recursion.
  - Added unit tests covering binary detection, file preview truncation,
    and directory preview listing to ensure behaviour remains stable.
  - Conservative visibility tidy: attempted to tighten helper visibility,
    but kept public re-exports intact (compat shim `preview_helpers.rs`) to
    avoid breaking downstream callers.
  - Small hygiene changes in `app/src/app/core/methods.rs` and
    `app/src/app/core/init.rs`: replaced `use super::*` globs with explicit
    imports and added brief module docs.

- Switch POSIX ACL handling to Rust-only xattr round-trip
  - Add `app/src/fs_op/posix_acl.rs` which preserves POSIX ACLs by
    reading and writing the `system.posix_acl_access` and
    `system.posix_acl_default` xattrs as opaque binary blobs (round-trip).
  - Remove reliance on native `libacl` bindings and external `getfacl`/
    `setfacl` invocations; `app/Cargo.toml` cleaned of the optional
    `posix-acl` binding, the `acl-native` feature and the `which` helper.
  - Behaviour is best-effort: ACL xattr read/write failures are handled
    gracefully and the unit test `fs_op::posix_acl::tests::roundtrip_acl_xattrs`
    will skip when xattrs are not supported on the host filesystem.
  - Rationale: keep the `app` crate self-contained in Rust and avoid
    link-time/runtime failures on systems without system ACL libraries.

### Notes

- Tests run locally and currently pass.

- Filesystem ops: richer errors and UI adapter
  - Introduce a richer filesystem error type `FsOpError` in
    `app/src/fs_op/error.rs` (uses `thiserror` for clear `Display` and
    `From<std::io::Error>` conversions).
  - Add `thiserror = "1.0"` to `app/Cargo.toml` and derive error impls.
  - Add `errors::render_fsop_error` adapter which maps `FsOpError` to the
    existing templated output (delegates to `render_io_error` for IO
    variants). This lets UI handlers display human-friendly messages
    without changing all call sites at once.
  - Refactor `app/src/fs_op/app_ops.rs` to return `Result<_, FsOpError>`
    for high-level App filesystem operations and tidy internal API.
  - Update runner handlers (`runner/handlers/*`) to use
    `render_fsop_error` where appropriate.
  - Tests updated/added for move/rename fallback behaviour (feature
    `test-helpers`). All tests pass locally after the change.

  - Refactor: move/copy helpers and richer move errors
    - Refactored `app/src/fs_op/mv.rs` to follow idiomatic Rust: simplified
      error propagation using `?`, deterministic directory creation, and
      parallel file copying where appropriate. The `move_path` API retains
      its stable signature but falls back from `rename` to a copy+remove
      strategy on platforms/filesystems where `rename` fails.
    - Replaced ad-hoc error plumbing with structured errors (using
      `thiserror`) and expanded the move error variant to include
      contextual fields (`src`, `dest`, `context`) to improve diagnostics.
    - Added integration tests `app/tests/mv_edge_cases.rs` that exercise
      symlink-to-directory copy behaviour and move fallback/error handling
      when the destination is unwritable.

- Test-hooks: move and feature-gate
  - Move test-only failure hooks used by filesystem operation tests into a
    dedicated module: `app/src/fs_op/test_helpers.rs`.
  - Expose a small, stable API under `crate::fs_op::test_helpers` used by
    unit tests to force rename/copy/write failure paths and to acquire a
    global test lock when mutating hooks.
  - The hooks are enabled when running with the Cargo feature
    `test-helpers`; when the feature is not enabled the module provides
    safe no-op fallbacks so production builds are unaffected.
  - Tidy: remove unused public re-exports from `crate::fs_op::test_helpers`
    and reference the private `inner` helpers directly in tests to
    eliminate dead-code / unused-import warnings during builds.
  - Enable the feature when running tests that rely on the hooks, e.g.:

    ```bash
    cd app
    cargo test -p fileZoom --features test-helpers -- --nocapture
    ```

- Refactor: `fs_op::path` path resolver
  - Reworked `app/src/fs_op/path.rs` to improve clarity and robustness:
    - Replace manual `Display`/`Error` impl with `thiserror`-derived
      `PathError` for clearer diagnostics and easier conversions.
    - Use `directories_next::UserDirs` (with an env-var fallback) for reliable
      cross-platform `~` expansion.
    - Simplified `resolve_path` logic, tightened types, and removed
      duplicated code paths.
    - Tightened visibility and removed unused imports.
  - Tests: moved module-level unit tests into the integration test
    `app/tests/fs_op_path.rs` to centralise fs-op behaviour checks.
  - Rationale: improves portability, error clarity, and aligns with
    idiomatic Rust; change is non-breaking for public API and tests pass.

- Refactor: `app/src/fs_op/stat.rs`
  - Consolidated simple path predicates into a small `PathType` enum and
    introduced `PathType::of` to classify path kinds (NotFound/Directory/File/Other).
  - Reimplemented `exists`, `is_dir`, and `is_file` to use the classifier
    (reduces duplicated filesystem checks) and added unit tests.
  - Behaviour and public helpers remain compatible; tests updated and pass.

- Refactor: conflict resolution handler
  - Simplified `runner/handlers/conflict.rs`: extracted a pure mapping helper
    and centralized the send+progress transition to remove duplicated
    branching logic and reduce clones/side-effects.
  - Added focused unit tests for the mapping logic (`map_selection_to_decision`) in
    `app/src/runner/handlers/conflict.rs` to cover overwrite/skip/cancel cases.
  - Behaviour preserved: UI still sends `OperationDecision` messages and
    transitions to `Mode::Progress`; internal code is clearer and easier to
    maintain.
  - Tests:
    - Add unit tests for context-menu behaviour:
      - `app/tests/context_menu_extra.rs` covers unknown/other labels (ensuring
        an informative `Mode::Message` is shown) and navigation boundary cases
        (selection does not underflow or overflow).
      - These tests exercise the context-menu handler and help protect the
        `ContextAction` parsing and navigation logic.

- Refactor: runner command parsing and execution
  - Refactored `app/src/runner/commands.rs` for clarity and idiomatic Rust:
    - Introduced a small `ParsedCommand` enum and the `parse_command` helper
      to separate textual command parsing from execution.
    - Added `ParsedCommand::execute` and a typed `execute_command` entry
      point that operates on `App` for clearer semantics and testability.
    - Kept `perform_action` available and restored public visibility where
      needed to preserve existing consumers and tests.
  - Added an integration test `app/tests/execute_command_integration.rs`
    that exercises `execute_command` end-to-end (menu navigation, preview
    toggle and activation) and validates `App` state transitions.
  - Result: improved maintainability, clearer parsing/dispatch separation
    and the test-suite passes after the change.
