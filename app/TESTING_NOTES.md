Running the `start_options` test
--------------------------------

The repository includes a small integration test `start_options` which verifies
that CLI-style startup options are applied into the `App` initial state. Run it
from the repository root (or from the `app/` directory):

```bash
# from repo root
cargo test -p fileZoom start_options -- --nocapture

# or from the `app/` directory
cd app
cargo test start_options -- --nocapture
```

The test is quick and non-interactive; it exercises `App::with_options` and
the UI theme setter without launching the full TUI.
