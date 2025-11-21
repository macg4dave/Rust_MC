Dual Panels: Two side-by-side panels with independent directories, quick copy/move between them.
Quick View: F3-style viewer/preview pane showing file contents without opening editor.
VFS (Virtual File System): Transparent access to archives, FTP, SFTP, SMB, etc., via a VFS abstraction.
Command Line + Menu Bar: Integrated command line and context-sensitive menu bar (keys + mouse).
Operation Queue & Progress: Modal/queue for long ops with progress and ability to suspend/cancel.
Batch Operations (Rename/Filter): Multi-file rename, regex filters, and selection patterns.
Directory Hotlist / Bookmarks: Quickly jump to frequently used directories.
Context Menus & File Actions: Right-click or hotkey-driven context actions (permissions, edit, view).
Search & Filter: Fast file search with regex and attribute filtering.
Color Themes & Syntax Highlighting: Color for file types, plus highlighting in viewer.
Mouse Support: Full mouse-driven UI (click-to-select, menus).
Robust Keybindings: Intuitive, discoverable keymaps (F-keys) and hints on-screen.
Safe Deletes / Trash: Option to move to trash or require confirmation for destructive ops.
Localization & Accessibility: Multi-language support and clear keyboard navigation.
Quick View / Preview Pane: High impact, small-to-medium effort. Improves navigation speed.
Operation Progress Modal/Queue: UX improvement for long copies/moves; small-medium effort.
Bookmarks / Hotlist: Lightweight feature that greatly improves workflow; small effort.
Batch Rename UI: Powerful user-facing feature; small-medium effort with new modal.
VFS Abstraction (design, incremental): Long-term investment enabling SSH/FTP/archives; larger effort but high payoff.
Configurable Keybindings & Hints: Improves discoverability and user comfort; small effort.
Coloring & File Type Detection: Improve readability and UX; small effort.
Store bookmarks, keybindings, last layout in settings.
Add integration tests for VFS behaviors, preview, batch rename, and permission handling.
