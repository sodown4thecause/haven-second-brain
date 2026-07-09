use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

/// Events emitted by the filesystem watcher.
#[derive(Debug, Clone)]
pub enum FsEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Removed(PathBuf),
}

/// A pending event held in the debounce buffer awaiting flush.
#[derive(Debug)]
struct PendingEvent {
    event: FsEvent,
    last_seen: Instant,
}

/// Shared debounce state: a map from path to the most recent pending event.
#[derive(Debug)]
struct DebounceState {
    pending: Mutex<HashMap<PathBuf, PendingEvent>>,
}

/// A debounced filesystem watcher that emits events over a broadcast channel.
///
/// Events for the same path within the `debounce` window are collapsed into a
/// single trailing-edge event. This prevents indexing storms when editors or
/// sync tools write to a file many times in quick succession.
pub struct FsWatcher {
    rx: broadcast::Receiver<FsEvent>,
    _watcher: notify::RecommendedWatcher,
    _flush_task: JoinHandle<()>,
    _shutdown: Arc<AtomicBool>,
}

impl FsWatcher {
    /// Start watching the given directory (recursively) with a default
    /// 300 ms debounce window.
    pub fn start(path: impl AsRef<Path>) -> Result<Self, notify::Error> {
        Self::start_with_debounce(path, Duration::from_millis(300))
    }

    /// Start watching with a custom debounce duration.
    ///
    /// Events for the same path are held until `debounce` has elapsed with no
    /// new activity for that path, then emitted as a single trailing-edge event.
    pub fn start_with_debounce(
        path: impl AsRef<Path>,
        debounce: Duration,
    ) -> Result<Self, notify::Error> {
        let (tx, rx) = broadcast::channel(256);
        let state = Arc::new(DebounceState {
            pending: Mutex::new(HashMap::new()),
        });
        let shutdown = Arc::new(AtomicBool::new(false));
        let path_buf = path.as_ref().to_path_buf();

        // notify callback — runs on the watcher's internal thread (synchronous).
        // It only updates the shared pending map; actual emission is deferred to
        // the flush task so that bursts collapse into one event per path.
        let state_cb = state.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let Some(path) = event.paths.first() else {
                    return;
                };
                let fs_event = match event.kind {
                    EventKind::Create(_) => FsEvent::Created(path.clone()),
                    EventKind::Modify(_) => FsEvent::Modified(path.clone()),
                    EventKind::Remove(_) => FsEvent::Removed(path.clone()),
                    _ => return,
                };
                if let Ok(mut pending) = state_cb.pending.lock() {
                    pending.insert(
                        path.clone(),
                        PendingEvent {
                            event: fs_event,
                            last_seen: Instant::now(),
                        },
                    );
                }
            }
        })?;

        watcher.watch(&path_buf, RecursiveMode::Recursive)?;

        // Background flush task — wakes periodically and emits events whose
        // debounce window has elapsed since the last activity for that path.
        let shutdown_flush = shutdown.clone();
        let tx_flush = tx;
        let flush_task = tokio::spawn(async move {
            let poll = std::cmp::max(debounce / 4, Duration::from_millis(10));
            loop {
                tokio::time::sleep(poll).await;
                if shutdown_flush.load(Ordering::SeqCst) {
                    break;
                }
                let now = Instant::now();
                let to_send: Vec<FsEvent> = {
                    let mut pending = match state.pending.lock() {
                        Ok(guard) => guard,
                        Err(_) => continue,
                    };
                    let expired: Vec<PathBuf> = pending
                        .iter()
                        .filter(|(_, pe)| now.duration_since(pe.last_seen) >= debounce)
                        .map(|(p, _)| p.clone())
                        .collect();
                    expired
                        .into_iter()
                        .filter_map(|p| pending.remove(&p))
                        .map(|pe| pe.event)
                        .collect()
                };
                for event in to_send {
                    let _ = tx_flush.send(event);
                }
            }
        });

        Ok(Self {
            rx,
            _watcher: watcher,
            _flush_task: flush_task,
            _shutdown: shutdown,
        })
    }

    /// Get a new receiver for filesystem events.
    pub fn receiver(&self) -> broadcast::Receiver<FsEvent> {
        self.rx.resubscribe()
    }
}

impl Drop for FsWatcher {
    fn drop(&mut self) {
        self._shutdown.store(true, Ordering::SeqCst);
        self._flush_task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[tokio::test]
    async fn watcher_detects_file_creation() {
        let dir = TempDir::new().unwrap();
        let watcher = FsWatcher::start(dir.path()).unwrap();
        let mut rx = watcher.receiver();

        let file_path = dir.path().join("test.md");
        std::fs::write(&file_path, "content").unwrap();

        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert!(matches!(event, FsEvent::Created(_) | FsEvent::Modified(_)));
    }

    #[tokio::test]
    async fn watcher_debounces_rapid_writes() {
        let dir = TempDir::new().unwrap();
        let watcher =
            FsWatcher::start_with_debounce(dir.path(), Duration::from_millis(200)).unwrap();
        let mut rx = watcher.receiver();

        let file_path = dir.path().join("debounce.md");
        // Five rapid writes with small gaps — must collapse into one event.
        for i in 0..5 {
            std::fs::write(&file_path, format!("content {i}")).unwrap();
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        // Within the debounce window, no event should arrive. With the old
        // pass-through implementation an event would fire almost immediately.
        let immediate = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(
            immediate.is_err(),
            "no event expected within debounce window"
        );

        // After the debounce window elapses, the collapsed event arrives.
        let event = tokio::time::timeout(Duration::from_millis(600), rx.recv())
            .await
            .expect("event expected after debounce window")
            .expect("channel should still be open");
        assert!(matches!(event, FsEvent::Created(_) | FsEvent::Modified(_)));
    }
}
