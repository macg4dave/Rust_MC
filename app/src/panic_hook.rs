//! Panic hook helper: installs a panic hook that attempts to restore the
//! terminal state (leave alternate screen, disable raw mode) before allowing
//! the normal panic output to be emitted. It also captures a short crash
//! report (timestamp, thread, location, payload, backtrace) and writes it to
//! a `crash_reports` directory under the platform-specific data dir. This
//! helps with post-mortem debugging when users report crashes.

use std::panic::{self};
use std::io::Write;

/// Install the panic hook which will force-restore the terminal, write a
/// crash report to disk (best-effort), then delegate to the previously-
/// registered hook (to print the usual panic message/backtrace).
pub fn install_panic_hook() {
    // Take the existing hook so we can call it after we restore the terminal.
    let prev = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        // Best-effort restore of terminal state. Safe to call even if the
        // terminal wasn't initialized.
        crate::runner::terminal::force_restore();

        // Try to write a crash report; do not panic if any step fails.
        let _ = (|| {
            // Collect basic info about the panic.
            let thread = std::thread::current();
            let thread_name = thread.name().unwrap_or("<unnamed>");

            let location = if let Some(loc) = info.location() {
                format!("{}:{}", loc.file(), loc.line())
            } else {
                "<unknown>".to_string()
            };

            let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "<non-string-payload>".to_string()
            };

            // Capture a backtrace if available.
            let backtrace = std::backtrace::Backtrace::capture();

            // Determine a directory to write crash reports.
            let base_dir = directories_next::ProjectDirs::from("net", "macg4dave", "fileZoom")
                .map(|p| p.data_local_dir().to_path_buf())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
            let crash_dir = base_dir.join("crash_reports");
            let _ = std::fs::create_dir_all(&crash_dir);

            // Build a filename with timestamp and pid.
            let ts = match chrono::Utc::now().format("%Y%m%dT%H%M%S").to_string() {
                s => s,
            };
            let pid = std::process::id();
            let filename = format!("panic-{}-{}.log", ts, pid);
            let path = crash_dir.join(filename);

            // Open and append the report.
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;

            writeln!(f, "fileZoom panic report")?;
            writeln!(f, "timestamp: {}", ts)?;
            writeln!(f, "pid: {}", pid)?;
            writeln!(f, "thread: {}", thread_name)?;
            writeln!(f, "location: {}", location)?;
            writeln!(f, "payload: {}", payload)?;
            writeln!(f, "--- backtrace ---")?;
            writeln!(f, "{:?}", backtrace)?;
            writeln!(f, "--- env ---")?;
            if let Ok(env) = std::env::var("RUST_LOG") {
                writeln!(f, "RUST_LOG={}", env)?;
            }

            // Flush to make best-effort persistence before exit.
            let _ = f.flush();
            Ok::<(), std::io::Error>(())
        })();

        // Small user-facing message.
        eprintln!("\n\nfileZoom: an unexpected error occurred — the program will exit. A crash report may have been written.\n");

        // Delegate to the previous hook which prints the detailed panic info
        // (including location and backtrace when enabled).
        prev(info);
    }));
}
