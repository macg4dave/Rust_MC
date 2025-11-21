use assert_fs::TempDir;
use fileZoom::app::App;
use fileZoom::app::Mode;
use fileZoom::runner::commands::execute_command;
use std::env;
use std::sync::Mutex;

static TEST_CWD_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn execute_command_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    // Acquire global cwd lock to avoid races when tests set the process cwd
    let _guard = TEST_CWD_LOCK.lock().unwrap();

    let temp = TempDir::new()?;
    let orig = env::current_dir()?;
    env::set_current_dir(temp.path())?;

    let mut app = App::new()?;

    // Initially preview should be hidden
    assert!(!app.preview_visible);

    // toggle-preview should flip the flag
    let matched = execute_command(&mut app, "toggle-preview")?;
    assert!(matched);
    assert!(app.preview_visible);

    // menu-next should advance the menu index
    let n = fileZoom::ui::menu::menu_labels().len();
    let before = app.menu_index;
    let matched = execute_command(&mut app, "menu-next")?;
    assert!(matched);
    assert_eq!(app.menu_index, (before + 1) % n);

    // menu-prev should move back
    let matched = execute_command(&mut app, "menu-prev")?;
    assert!(matched);
    assert_eq!(app.menu_index, before);

    // menu-activate should set a Mode (Message or Settings)
    let matched = execute_command(&mut app, "menu-activate")?;
    assert!(matched);
    match &app.mode {
        Mode::Message { title, .. } => {
            // When menu_index == 0 the label is "File"
            assert_eq!(title, "File");
        }
        Mode::Settings { .. } => {
            // If the labels change this is still acceptable
        }
        other => panic!("unexpected mode after menu-activate: {:?}", other),
    }

    // restore original cwd
    env::set_current_dir(orig)?;
    drop(_guard);
    Ok(())
}
