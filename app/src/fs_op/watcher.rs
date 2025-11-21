#![cfg(feature = "fs-watch")]

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

/// Filesystem event detailed enough for the app to decide what to refresh.
///
/// This type represents a minimal set of events the application cares about
/// coming from the filesystem watcher. All paths are owned (`PathBuf`) since
/// the watcher runs on a dedicated thread and must send owned data across
/// channels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsEvent {
    /// A file or directory was created.
    Create(PathBuf),
    /// A file or directory was modified.
    Modify(PathBuf),
    /// A file or directory was removed.
    Remove(PathBuf),
    /// A rename with (from, to) paths.
    Rename(PathBuf, PathBuf),
    /// Any other event we don't map explicitly.
    Other,
}

/// Convert a `notify::Event` into our crate-local `FsEvent`.
///
/// The conversion prefers detecting renames when two or more paths are
/// present on the event. Otherwise it maps the first path and a known
/// `EventKind` to a corresponding `FsEvent`.
fn map_notify_event(event: &Event) -> FsEvent {
    match event.paths.as_slice() {
        [from, to, ..] => FsEvent::Rename(from.clone(), to.clone()),
        [p, ..] => match &event.kind {
            EventKind::Create(_) => FsEvent::Create(p.clone()),
            EventKind::Modify(_) => FsEvent::Modify(p.clone()),
            EventKind::Remove(_) => FsEvent::Remove(p.clone()),
            _ => FsEvent::Other,
        },
        _ => FsEvent::Other,
    }
}

/// Spawn a background thread that watches `path` and sends mapped `FsEvent`
/// values into `tx` for events of interest. The returned `JoinHandle` owns
/// the watcher; the thread lives until `stop_rx` is signalled or the
/// process exits.
///
/// This function keeps a small, well-defined responsibility: create a
/// `notify::RecommendedWatcher`, register `path` (recursively), and forward
/// events to the provided channel. Errors are logged via `tracing` rather
/// than propagated because the watcher runs inside its own thread.
pub fn spawn_watcher(path: PathBuf, tx: Sender<FsEvent>, stop_rx: Receiver<()>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        // Create watcher with a callback that forwards mapped events to `tx`.
        let res: notify::Result<RecommendedWatcher> = RecommendedWatcher::new(
            move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    let ev = map_notify_event(&event);
                    if let Err(e) = tx.send(ev) {
                        tracing::error!("failed to send fs event: {:#?}", e);
                    }
                }
                Err(e) => tracing::error!("file watcher error: {:#?}", e),
            },
            Config::default(),
        );

        match res {
            Ok(mut watcher) => {
                // Use recursive watching so changes in subdirectories are observed.
                if let Err(e) = watcher.watch(&path, RecursiveMode::Recursive) {
                    tracing::error!("failed to watch {}: {:#?}", path.display(), e);
                    return;
                }

                // Block until stop signal is received; when the sender side is
                // closed `recv` returns an error and we simply exit the thread.
                if stop_rx.recv().is_err() {
                    tracing::debug!("stop signal receiver closed, exiting watcher for {}", path.display());
                }
            }
            Err(e) => tracing::error!("failed to create watcher for {}: {:#?}", path.display(), e),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, ModifyKind, RemoveKind};

    fn make_event(kind: EventKind, paths: Vec<PathBuf>) -> Event {
        Event { kind, paths, attrs: Default::default() }
    }

    #[test]
    fn map_create() {
        let ev = make_event(EventKind::Create(CreateKind::Any), vec![PathBuf::from("/tmp/a.txt")]);
        assert_eq!(map_notify_event(&ev), FsEvent::Create(PathBuf::from("/tmp/a.txt")));
    }

    #[test]
    fn map_modify() {
        let ev = make_event(EventKind::Modify(ModifyKind::Any), vec![PathBuf::from("/tmp/b.txt")]);
        assert_eq!(map_notify_event(&ev), FsEvent::Modify(PathBuf::from("/tmp/b.txt")));
    }

    #[test]
    fn map_remove() {
        let ev = make_event(EventKind::Remove(RemoveKind::Any), vec![PathBuf::from("/tmp/c.txt")]);
        assert_eq!(map_notify_event(&ev), FsEvent::Remove(PathBuf::from("/tmp/c.txt")));
    }

    #[test]
    fn map_rename() {
        // Use a simple Modify::Any kind; the mapping prefers the presence
        // of two paths to detect a rename regardless of the kind.
        let ev = make_event(EventKind::Modify(ModifyKind::Any), vec![PathBuf::from("/tmp/from"), PathBuf::from("/tmp/to")]);
        assert_eq!(map_notify_event(&ev), FsEvent::Rename(PathBuf::from("/tmp/from"), PathBuf::from("/tmp/to")));
    }
}
