pub mod write_settings;
pub mod read_settings;

// Re-export commonly used types/functions for convenience
pub use write_settings::Settings;
pub use write_settings::save_settings;
pub use read_settings::load_settings;
