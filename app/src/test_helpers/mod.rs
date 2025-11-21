// Test helpers for unit/integration tests.
// This module is compiled only for tests or when the `test-helpers` feature
// is explicitly enabled.

#[cfg(test)]
pub use _test_only::{set_up_temp_home, set_up_temp_xdg_config};

#[cfg(test)]
mod _test_only {
	use tempfile::TempDir;

	/// Create a temporary directory and set common environment variables so
	/// tests do not touch the real user environment.
	///
	/// Returns the `TempDir` which the caller should keep alive for the
	/// duration of the test.
	pub fn set_up_temp_home() -> TempDir {
		let td = tempfile::tempdir().expect("failed to create tempdir");
		std::env::set_var("HOME", td.path());
		std::env::set_var("XDG_CONFIG_HOME", td.path());
		std::env::set_var("XDG_DATA_HOME", td.path());
		td
	}

	/// Convenience helper that sets only XDG config to a new tempdir and
	/// returns it.
	pub fn set_up_temp_xdg_config() -> TempDir {
		let td = tempfile::tempdir().expect("failed to create tempdir");
		std::env::set_var("XDG_CONFIG_HOME", td.path());
		td
	}
}
