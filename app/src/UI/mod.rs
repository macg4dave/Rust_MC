pub mod ui_main;
pub mod ui_state;
pub mod themes;
pub mod menu;
pub mod menu_model;
pub mod colors;
pub mod command_line;
pub mod dialogs;
pub mod modal;
pub mod panels;
pub mod widgets {
    pub mod header;
    pub mod footer;
    pub mod main_menu;
    pub mod submenu;
    pub mod file_list;
    pub mod preview;
    pub mod progress_bar;
    pub mod panel;
}

pub use ui_main::{draw_frame, ui};
pub use ui_state::UIState;
pub use themes::Theme;
