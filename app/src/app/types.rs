use chrono::{DateTime, Local};
use std::fmt;
use std::path::PathBuf;

/// A directory entry displayed in a panel.
///
/// This is a lightweight representation used by the UI layer; it intentionally
/// stores a `PathBuf` and a precomputed `name` to avoid repeated allocations
/// while rendering.
///
/// This is a domain-only type. Presentation concerns (headers, parent
/// rows, and synthetic UI rows) are owned by the UI module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Display name for the entry (filename or `..` or the full path in the
    /// case of the header row).
    pub name: String,
    /// Full path to the entry.
    pub path: PathBuf,
    /// Whether the entry is a directory. Header rows are not directories.
    pub is_dir: bool,
    /// File size in bytes. Directories typically have `0` here.
    pub size: u64,
    /// Optional last-modified timestamp.
    pub modified: Option<DateTime<Local>>,
}

impl Entry {
    /// Construct a regular file entry.
    pub fn file(
        name: impl Into<String>,
        path: PathBuf,
        size: u64,
        modified: Option<DateTime<Local>>,
    ) -> Self {
        Entry {
            name: name.into(),
            path,
            is_dir: false,
            size,
            modified,
        }
    }

    /// Construct a regular directory entry.
    pub fn directory(
        name: impl Into<String>,
        path: PathBuf,
        modified: Option<DateTime<Local>>,
    ) -> Self {
        Entry {
            name: name.into(),
            path,
            is_dir: true,
            size: 0,
            modified,
        }
    }

    // Header/parent are UI concerns implemented in `ui::panels::UiEntry`.

    // NOTE: UI-only helpers like `is_header` and `is_parent` were intentionally
    // moved into the UI layer. This keeps `Entry` as a domain struct and
    // prevents the core data model from depending on presentation concerns.
}

/// Keys by which listings may be sorted.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum SortKey {
    #[default]
    Name,
    Size,
    Modified,
}

impl SortKey {
    /// Cycle to the next sorting key in the order Name -> Size -> Modified -> Name
    pub fn next(self) -> Self {
        match self {
            SortKey::Name => SortKey::Size,
            SortKey::Size => SortKey::Modified,
            SortKey::Modified => SortKey::Name,
        }
    }
}

// Default derived via `#[default]` on the `Name` variant.

/// Mode represents the global UI mode/state the application may be in.
///
/// - `Normal` is the default browsing mode.
/// - `Confirm` is used for yes/no prompts (for example, delete).
/// - `Message` displays an information dialog with buttons.
/// - `Input` requests textual input from the user.
#[derive(Clone, Debug, Default)]
pub enum Mode {
    #[default]
    Normal,
    Confirm {
        msg: String,
        on_yes: Action,
        selected: usize,
    },
    Message {
        title: String,
        content: String,
        buttons: Vec<String>,
        selected: usize,
    },
    Progress {
        title: String,
        processed: usize,
        total: usize,
        message: String,
        cancelled: bool,
    },
    Conflict {
        path: std::path::PathBuf,
        selected: usize,
        apply_all: bool,
    },
    Input {
        prompt: String,
        buffer: String,
        kind: InputKind,
    },
}

// Default for Mode is derived via `#[default]` on the `Normal` variant.

/// The kind of input requested from the user. This guides how the input buffer
/// is interpreted (e.g. a destination path vs a filename).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum InputKind {
    Copy,
    Move,
    Rename,
    NewFile,
    NewDir,
    ChangePath,
}

/// Actions represent high-level user requests executed by the runner.
///
/// These are intentionally simple enums so they can be passed around the UI
/// and executed by `runner::commands::perform_action`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    DeleteSelected,
    CopyTo(PathBuf),
    MoveTo(PathBuf),
    RenameTo(String),
    NewFile(String),
    NewDir(String),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::DeleteSelected => write!(f, "DeleteSelected"),
            Action::CopyTo(p) => write!(f, "CopyTo({})", p.display()),
            Action::MoveTo(p) => write!(f, "MoveTo({})", p.display()),
            Action::RenameTo(name) => write!(f, "RenameTo({})", name),
            Action::NewFile(name) => write!(f, "NewFile({})", name),
            Action::NewDir(name) => write!(f, "NewDir({})", name),
        }
    }
}

/// Which panel is active.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Side {
    Left,
    Right,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Left => write!(f, "Left"),
            Side::Right => write!(f, "Right"),
        }
    }
}

impl fmt::Display for SortKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortKey::Name => write!(f, "Name"),
            SortKey::Size => write!(f, "Size"),
            SortKey::Modified => write!(f, "Modified"),
        }
    }
}
