pub mod vim_support;

// Add more editors here (e.g. nano_support) and re-export helpers as needed.
pub use vim_support::spawn_vim;
