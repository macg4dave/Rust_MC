Rust MC Prompt (Improved)

name: “fileZoom Assistant” scope: “repository” description:
“High‑precision, repository-aware Copilot prompt for Rust code changes,
refactors, tests, and PR-ready patches.”

------------------------------------------------------------------------

🔥 Hard Constraints (Always Enforced)

-   Run full test suite (cargo test -p fileZoom) before and after
    changes.
-   Make the smallest correct patch.
-   Add/update tests for any behavioural change.
-   Preserve public CLI interfaces and machine-facing output unless
    explicitly permitted.
-   Never remove features/tests unless tests confirm they’re obsolete.
-   No unwrap() except in trivial test scaffolding.
-   No unsafe unless absolutely necessary and validated with tests +
    rationale.

------------------------------------------------------------------------

🧱 Repository Context

-   Project: fileZoom, a pure Rust TUI file manager using Ratatui +
    Crossterm.
-   Crate root: app/ (fileZoom).
-   Tests: Integration tests in app/tests/, using fixtures from
    app/tests/fixtures/.
-   Build tools: cargo build, cargo test, cargo run, rustfmt, clippy.

Important paths:
- Crate root: `app/` (fileZoom binary crate).
- Core entrypoints: `app/src/lib.rs`, `app/src/app.rs`, `app/src/main.rs`.
- App internals: `app/src/app/` (contains `core/`, `types.rs`, `path.rs`, `settings/`).
- UI code: `app/src/ui/` (menu, modal, panels, dialogs, util).
- Runner/handlers: `app/src/runner/` (commands, event_loop_main, handlers/).
- Filesystem ops: `app/src/fs_op/` (copy, mv, stat, permissions, path helpers).
- Input handling: `app/src/input/` (keyboard, mouse).
- Virtual FS and network backends: `app/src/vfs/`.
- Errors and localization: `app/src/errors/`.
- Building helpers and scripts: `app/building/`, `app/building/make_fakefs/`, `app/scripts/`.
- Test helpers and fixtures: `app/test_helper/`, `app/src/test_helpers/`, and integration tests in `app/tests/` and top-level `tests/`.
- Docker and packaging: `app/docker/`.

------------------------------------------------------------------------

🦀 Rust Conventions (Repo-Specific)

-   Idiomatic Rust only:
    -   snake_case, PascalCase for types
    -   error handling via Result + ?
    -   avoid clone-heavy or allocation-heavy patterns
    -   enums > booleans for state
-   Keep functions small and single‑purpose.
-   Add doc‑comments to all public APIs.
-   Provide examples in Rustdoc when beneficial.

------------------------------------------------------------------------

📐 Patch Workflow Template (Use This Structure)

Task Summary:
<One-sentence description of the requested change>

Details:
- Goal: <Behaviour change or refactor>
- Relevant Files: <path1.rs, path2.rs>
- Tests to Add/Update: <describe>
- Must Not Change: <API, CLI flags, modules>

------------------------------------------------------------------------

🧠 Assistant Instructions

When generating a patch:

1.  Provide a 2–3 bullet plan describing the intended modification.
2.  Apply the smallest code change consistent with correctness and repo
    style.
3.  Add/update tests validating all behaviour.
4.  Run cargo test -p fileZoom and paste full output.
5.  If failures occur, iterate up to 5 times with brief reasoning each
    time.
6.  Final output must include:
    -   Summary of changes
    -   Exact patch(es) in apply_patch V4A diff format
    -   Passing cargo test output
    -   Optional follow‑up recommendations

Constraints: - Do not modify CLI arguments or terminal-facing output
unless explicitly authorised.
- Do not silently alter semantics.
- All behaviour changes must be test‑driven and documented.

------------------------------------------------------------------------

🧾 Example Prompt Snippets

Bug fix

    Task: Prevent crash when opening empty directories.
    Details:
    - Fix out‑of‑bounds access in `App::enter`.
    - Files: app/src/app.rs
    - Tests: add integration test reproducing empty‑directory case.

Feature

    Task: Add arrow‑key navigation for top menu.
    Details:
    - Add menu state + rendering.
    - Files: app/src/ui/menu.rs, app/src/main.rs
    - Tests: menu state unit tests. Describe manual test steps for TUI.

Refactor

    Task: Extract panel rendering helpers to their own module.
    Details:
    - Move helpers from ui/mod.rs → ui/panels.rs.
    - Tests: add formatting helper tests.

------------------------------------------------------------------------

🔧 Usage for VS Code / Copilot

-   Place in .github/instructions/ or as a .prompt.md file for quick
    recall in Copilot.
-   When requesting changes, fill in the <Task> block, paste it, and
    Copilot will generate a patch.

------------------------------------------------------------------------

⚠ Important Notes

-   Copilot Code Review reads only the first ~4,000 chars — keep
    critical rules at the top.
-   Public API changes require a migration note + dedicated tests.
