PR: Add TerminalGuard RAII + use queue! for init/restore

Summary

- Add `TerminalGuard` RAII wrapper to ensure terminal state is restored on drop.
- Switch `init_terminal`/restore paths to use `crossterm::queue!` + `flush()` to batch commands.
- Update mouse-capture helpers to use queued commands and flush.
- Add testing notes about terminal safety to `app/TESTING.md`.

Checklist

- [x] Add `TerminalGuard` type with `Drop` to restore terminal state.
- [x] `init_terminal()` returns `TerminalGuard`.
- [x] Replace direct `execute!` calls in init/restore with `queue!` + `flush()`.
- [x] Update `enable_mouse_capture_on_terminal`/`disable_mouse_capture_on_terminal` to queue commands and flush.
- [x] Add a short section in `app/TESTING.md` explaining terminal safety for tests.
- [ ] Run `cargo build` and `cargo test` and resolve any issues (please run locally).

Notes for reviewer

- The RAII guard lets existing code continue using the terminal via `Deref`/`DerefMut`.
- Explicit `restore_terminal(guard)` is provided for deterministic cleanup (and for returning errors).
- The change is intentionally small and focused on safety and batching terminal commands.

Suggested follow-ups (not in this PR)

- Consider adding integration tests that exercise panic/unwind paths to ensure Drop path restores the terminal.
- Consider migrating event loop code to use `crossterm::poll`/`EventStream` if async is adopted.
