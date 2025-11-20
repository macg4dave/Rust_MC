```markdown
# fileZoom — Copilot / Agent Instructions (concise)

Purpose: quick, actionable guidance to get an AI coding agent productive in this repo.

**Project Overview**
- **What**: a single binary Rust CLI file-manager (TUI) in the `app` crate.
- **Why**: small, dependency-light TUI focused on usability and mouse support for interactive demos and tests.

**Important Paths**
- `app/` : main crate (source, scripts, tests, fixtures).
- `app/src/lib.rs`, `app/src/app.rs` : core application logic and public types (`App`, `Action`, `Mode`, `Side`).
- `app/src/ui/` : UI rendering (see `menu.rs`, `modal.rs`, `panels.rs`).
- `app/tests/fixtures/` : packaged fixtures used by integration tests.
- `.github/instructions/` : path-specific prompt files (use these for context-aware tasks).
- `app/test_helper/ `: test utilities.

**Build / Test / Run (practical)**
- Preferred environment: Linux/macOS or WSL2 on Windows 11. Helper scripts check for WSL and will refuse to run elsewhere.
-- Run tests (inside WSL or Linux): `cd app && cargo test -p fileZoom -- --nocapture`
-- Run tests via helper (from WSL): `./app/scripts/run_tests.sh`
-- From Windows PowerShell run tests in WSL: `wsl -- cd /mnt/c/Users/<you>/github/fileZoom && ./app/scripts/run_tests.sh`
- Run the interactive demo: `cd app && ./scripts/user_test_wsl.sh prepare|build|run` (note: `run` starts the TUI and will take over your terminal; quit with `q`).

**Key Conventions & Patterns (repo-specific)**
- Tests and fixtures: integration tests copy/operate on `app/tests/fixtures/` into temp dirs. When adding tests, use `assert_fs`/`tempfile` dev-deps and avoid touching the real filesystem.

**Dependencies & Tooling**
- See `app/Cargo.toml` for runtime (`clap`, `crossterm`, `tui`, `anyhow`) and dev (`assert_fs`, `tempfile`) dependencies.
- Use `rust-analyzer`, `cargo fmt`, and `cargo clippy` for local checks.

**What agents should do (practical rules)**
- Prefer the smallest, focused patch that builds and keeps tests passing for the `fileZoom` crate.
- If behavior changes, add/update tests in `app/tests/` and reference fixtures in `app/tests/fixtures/` (crate: `fileZoom`).
 - Keep all tests in the `./app/tests/` folder; do not place tests elsewhere in the repository.
- Preserve public CLI flags and machine-facing outputs unless explicitly authorized; document any changes.
- Avoid `unsafe` unless necessary and accompanied by tests and rationale.

**Files to inspect for context before editing**
- `app/src/lib.rs`, `app/src/app.rs`, `app/src/main.rs`, `app/src/ui/mod.rs`, `app/tests/integration_tests.rs`, `app/TESTING.md`, `app/scripts/run_tests.sh`, `app/scripts/user_test_wsl.sh`.

**Quick examples**
-- Run tests with output: `cargo test -p fileZoom -- --nocapture`
- Run helper tests script (WSL): `./app/scripts/run_tests.sh`

**Notes**
- Keep critical rules near the top (Copilot Code Review reads only the first ~4k chars).
- This file is intentionally concise — consult `app/TESTING.md` and `.github/instructions/` for task-specific prompts.

``` # Repository custom instructions for GitHub Copilot

Purpose: Provide concise, repository-wide context and preferences for Copilot.

## Project overview
- CLI file manager written in Rust, focused on usability and simplicity, with mouse support.
- 100% RUST with no external dependencies beyond standard Rust crates.
- Supports MacOS, Debian and Fedora.

## Folder structure (important paths)
- /app/ — main crate
- `/README.md` — project README
- `/.github` — Copilot instructions and prompt files
- `/tests/` — integration tests and fixtures
- `/fs_op/` — filesystem operations module
- `/input/` — input handling module
- `/runner/` — event loop and application runner module
- `/ui/` — UI rendering module
- `/error_logs/` - error logging module

## Tools & environment
- Language: Rust.
- Build/test: `cargo build`, `cargo test`, `cargo run`.
- IDE: `rust-analyzer`, `cargo fmt`, `clippy` encouraged.

## Coding standards & conventions
- Follow `rustfmt` formatting and prefer idiomatic Rust (snake_case for identifiers, `Result` error handling, explicit `unwrap` only in small examples/tests).
- Prefer clarity over cleverness; add doc comments on public APIs.
- Preserve public CLI flags and their semantics.
- No unsafe or only with justification and added tests.
- Performance changes must not alter user-visible behavior.
- Patch that minimally changes code and explicitly calls out impacted modules.
- Make as modular code as possible; avoid large monolithic functions.
- Write modular functions with single responsibility.
- Write clear variable and function names. 
- Write comments for complex logic.
- Write doc comments for public APIs. 
- Write examples in doc comments where helpful.
- Write idiomatic Rust, leveraging iterators, pattern matching, and error handling best practices.
- Run the test suite locally and include full test output (`cargo test` or targeted test command).
- test output (`cargo test` or targeted test command). 
- tests must pass before submission.
- Build must pass before submission.
- fix any build errors and rerun build upto 5 times.
- fix any test errors and rerun test upto 5 times.
- do not allow dead_code
- remove any dead_code flags.
- build and test after code changes
- no error warnings allowed after build or test.
- no clippy warnings allowed after build or test.


## Testing and PRs
- Use `cargo test` for unit and integration tests. Include tests for any public behavior.
- Do not remove functions/modules without test updates passing.
- List tests it expects to pass and any additional tests it proposes.
- Write tests for any new behavior. 
- PRs should be small, focused, and include test results.
- Write/Update documentation for any public API changes.
- Write/Update tests for any behavioral.
-keep readme up to date with any changes.
 write to change log for any changes.


## Response preferences
- Keep answers concise and focused
- When giving code, prefer small patches that compile with `cargo test` or `cargo run`.

## Security & external references
- Do not fetch or assume private/internal resources. If a response requires external data, ask for the data or a public reference.

## Prompt files & path-specific instructions
- Use `.github/instructions/` for path-specific rules and `*.prompt.md` for reusable prompts in VS Code (public preview).

## Limitations and notes
- Copilot Code Review reads only the first 4,000 characters of any custom instruction file — keep critical rules near the top.
- Avoid overly prescriptive or fragile constraints (e.g., exact character limits or strict output formats) that may reduce usefulness.

## Hard constraints to include in every AI prompt
- Run the test suite locally and include full test output (`cargo test` or targeted test command).
- Make the smallest possible change needed to solve the request.
- Add or update tests for any behavioral change.
- Preserve public APIs and CLI machine-facing outputs unless explicitly allowed.
- Avoid removing features or tests. If you must remove or change behavior, add a migration note and tests demonstrating the new behavior.