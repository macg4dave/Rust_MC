// Deprecated compatibility re-export. `core::panel::Panel` is the canonical
// type; re-export it here for code that referenced `crate::app::panel`.
#[deprecated(since = "0.1.0", note = "`app::panel` is deprecated; use `app::core::panel` or `app::Panel` instead")]
pub use crate::app::core::panel::Panel;
