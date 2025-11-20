use std::env;
use tempfile::tempdir;
use fileZoom::app::settings::write_settings::save_settings;
use fileZoom::app::settings::read_settings::load_settings;
use fileZoom::app::settings::write_settings::Settings;

#[test]
fn save_and_load_settings_roundtrip() {
    let tmp = tempdir().expect("tempdir");
    // set XDG_CONFIG_HOME to tmp so we don't touch real home
    env::set_var("XDG_CONFIG_HOME", tmp.path());

    let s = Settings {
        theme: "solarized".into(),
        show_hidden: true,
        left_panel_width: 30,
        right_panel_width: 50,
    };

    save_settings(&s).expect("save should succeed");
    let loaded = load_settings().expect("load should succeed");
    assert_eq!(loaded, s);
}