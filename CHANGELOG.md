# Changelog

## Unreleased

- Add Command Line and Menu Bar integration
  - New `CommandLineState` in `App` and `ui::command_line` drawing/handler.
  - Simple command registry (`app::runner::commands::execute_command`) supporting `toggle-preview`, `menu-next`, `menu-prev`, and `menu-activate`.
  - Menu bar now activates on left-click (mouse handler updated).
  - Integration tests added: `ui_menu_interaction.rs` and `menu_commandline_feature.rs`.
  - Minor wiring in `ui::mod` and `runner` handlers to route keys and mouse events.

- Replace manual recursive filesystem walks with `walkdir` where appropriate
  (improves robustness and reduces manual recursion code). Files updated:
  - `app/src/fs_op/copy.rs`
  - `app/src/fs_op/mv.rs`
  - `app/src/building/make_fakefs_lib.rs`

### Notes

- Tests run locally and currently pass.
