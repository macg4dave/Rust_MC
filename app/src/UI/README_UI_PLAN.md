## UI Plan (Ratatui + Crossterm)

This directory will contain the new Ratatui-first UI implementation.

Structure
- `ui_main.rs` — primary entry and layout renderer
- `widgets/` — small, focused widgets (header, footer, file_list, preview)
- `tests/` — layout rendering tests using TestBackend

Design goals:
- Adaptive / responsive layout
- 100% ratatui + crossterm for terminal control
- Safe terminal lifecycles and extensive tests
