// Test helpers used only by the test-suite. This module is gated behind the
// `test-helpers` cargo feature so it is not included in production builds.
//
// Purpose: provide small, well-tested helpers for creating fake filesystem
// fixtures and applying permissions so integration tests don't need to shell
// out to the make_fakefs binary or duplicate logic.

pub mod make_fakefs;
