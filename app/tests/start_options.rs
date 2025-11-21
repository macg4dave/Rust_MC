use anyhow::Result;

/// Verify that `App::with_options` applies CLI-style start options into
/// the constructed `App` instance (mouse, show_hidden, theme).
#[test]
fn app_with_options_applies_settings() -> Result<()> {
    // Capture the current UI theme debug output so we can detect a change
    // after applying a theme via StartOptions.
    let before = format!("{:?}", fileZoom::ui::colors::current());

    let opts = fileZoom::app::StartOptions {
        start_dir: None,
        mouse_enabled: Some(false),
        theme: Some("dark".to_string()),
        show_hidden: Some(true),
        verbosity: Some(2),
    };

    let app = fileZoom::app::App::with_options(&opts)?;

    // Assert the settings were applied into the App instance.
    assert_eq!(app.settings.mouse_enabled, false);
    assert_eq!(app.settings.show_hidden, true);
    assert_eq!(app.settings.theme, "dark");

    // After applying a theme we expect the UI theme state to be different
    // than it was before (dark vs default). Compare the debug strings as a
    // lightweight way to observe the change.
    let after = format!("{:?}", fileZoom::ui::colors::current());
    assert_ne!(before, after, "UI theme did not change after StartOptions.theme");

    Ok(())
}
