// Thin compatibility shim: re-export the preview helpers from the
// `preview` module so code that referenced
// `crate::app::core::preview_helpers::...` continues to work.
pub use crate::app::core::preview::{
    build_directory_preview, build_file_preview, is_binary, PreviewError,
    MAX_DIR_PREVIEW_ENTRIES,
};
