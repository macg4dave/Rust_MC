---
name: fileZoom_api
description: "Prompt template for public API / library-level changes in fileZoom (lib exports, `fileZoom::` surface)."
---

Hard constraints
----------------
- Preserve backward-compatible public APIs unless a breaking change is explicitly requested.
-- If breaking changes are necessary, include a migration note and tests demonstrating the new behavior.
-- Run `cargo test -p fileZoom` and include full output.

Prompt template
---------------
Task:
"""
<Brief summary of API change>

Details:
- What to change: <public API additions/removals/behavior changes>
- Files: <list files>
- Tests/migration notes: <describe changes to tests or migration guidance>
"""

Assistant instructions
---------------------
1. Provide a concise plan (2 bullets).
2. Make minimal changes; prefer additive APIs over breaking ones.
3. Add tests that demonstrate the public contract (unit or integration).
4. If breaking, add a migration note in the changelog or README and include tests.
5. Run `cargo test -p fileZoom` and include output.
6. Prefer using well-maintained dependencies for common functionality (UI helpers, logging, parsing, etc.) instead of reimplementing them when appropriate — list suggestions in the PR description.

Example prompts
---------------
- "Export `Side` enum from `app::` root and update call sites. Files: `app/src/lib.rs`, `app/src/main.rs`. Add note in README." 
- "Change `App::new()` to accept an optional starting path; add tests and update docs." 
