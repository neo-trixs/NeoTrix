use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use notify::Watcher;

#[derive(Clone)]
pub struct FileChangeEntry {
    pub path: PathBuf,
    pub modified_at: SystemTime,
    pub is_deleted: bool,
}

pub struct FileWatcher {
    receiver: Receiver<notify::Result<notify::Event>>,
    _watcher: notify::RecommendedWatcher,
    changes: Arc<Mutex<Vec<FileChangeEntry>>>,
    file_ages: Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
}

impl FileWatcher {
    pub fn new(path: &Path) -> Result<Self, String> {
        let (tx, rx) = mpsc::sync_channel(1024);
        let changes: Arc<Mutex<Vec<FileChangeEntry>>> = Arc::new(Mutex::new(Vec::new()));
        let file_ages: Arc<Mutex<HashMap<PathBuf, SystemTime>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let _changes_clone = changes.clone();
        let file_ages_clone = file_ages.clone();

        let mut watcher = notify::RecommendedWatcher::new(
            move |event| {
                if tx.send(event).is_err() {
                    log::warn!("file_watcher event send failed: channel closed");
                }
            },
            notify::Config::default(),
        )
        .map_err(|e| format!("Failed to create watcher: {e}"))?;

        watcher
            .watch(path, notify::RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch path: {e}"))?;

        let receiver = rx;
        // seed existing file ages
        if let Ok(dir) = std::fs::read_dir(path) {
            let mut ages = HashMap::new();
            for entry in dir.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Ok(meta) = p.metadata() {
                        if let Ok(mtime) = meta.modified() {
                            ages.insert(p, mtime);
                        }
                    }
                }
            }
            if let Ok(mut fa) = file_ages_clone.lock() {
                *fa = ages;
            }
        }

        Ok(Self {
            receiver,
            _watcher: watcher,
            changes,
            file_ages,
        })
    }

    pub fn poll(&self) -> Vec<FileChangeEntry> {
        let mut results = Vec::new();
        loop {
            match self.receiver.try_recv() {
                Ok(Ok(event)) => {
                    for p in &event.paths {
                        let now = SystemTime::now();
                        let is_deleted = matches!(event.kind, notify::EventKind::Remove(_));
                        if let Ok(mut ch) = self.changes.lock() {
                            ch.push(FileChangeEntry {
                                path: p.clone(),
                                modified_at: now,
                                is_deleted,
                            });
                        }
                        if let Ok(mut fa) = self.file_ages.lock() {
                            if is_deleted {
                                fa.remove(p);
                            } else {
                                fa.insert(p.clone(), now);
                            }
                        }
                        results.push(FileChangeEntry {
                            path: p.clone(),
                            modified_at: now,
                            is_deleted,
                        });
                    }
                }
                Ok(Err(_)) => {}
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        results
    }

    pub fn pending_changes(&self) -> Vec<FileChangeEntry> {
        self.changes.lock().map(|ch| ch.clone()).unwrap_or_default()
    }

    pub fn drain_changes(&self) -> Vec<FileChangeEntry> {
        self.changes
            .lock()
            .map(|mut ch| std::mem::take(&mut *ch))
            .unwrap_or_default()
    }

    pub fn is_stale(&self, path: &Path) -> bool {
        self.file_ages
            .lock()
            .map(|fa| fa.contains_key(path))
            .unwrap_or(false)
    }

    pub fn recently_modified(&self, within_secs: u64) -> Vec<PathBuf> {
        self.file_ages
            .lock()
            .map(|fa| {
                fa.iter()
                    .filter(|(_, t)| {
                        t.elapsed()
                            .map(|d| d.as_secs() <= within_secs)
                            .unwrap_or(false)
                    })
                    .map(|(p, _)| p.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_watcher_create_and_poll() {
        let dir = std::env::temp_dir().join("neotrix_watcher_test");
        let _ = fs::create_dir_all(&dir);

        let watcher = FileWatcher::new(&dir).expect("watcher creation");

        let test_file = dir.join("test.txt");
        fs::write(&test_file, "hello").unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let changes = watcher.poll();
        let created = changes.iter().any(|c| c.path.ends_with("test.txt"));
        assert!(created, "should detect file creation");
    }

    #[tokio::test]
    async fn test_pending_changes() {
        let dir = std::env::temp_dir().join("neotrix_watcher_test2");
        let _ = fs::create_dir_all(&dir);

        let watcher = FileWatcher::new(&dir).expect("watcher creation");
        let test_file = dir.join("pending.txt");
        fs::write(&test_file, "data").unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        watcher.poll(); // drain into internal store
        let pending = watcher.pending_changes();
        assert!(!pending.is_empty());
    }

    #[tokio::test]
    async fn test_modified_tracking() {
        let dir = std::env::temp_dir().join("neotrix_watcher_test3");
        let _ = fs::create_dir_all(&dir);

        let watcher = FileWatcher::new(&dir).expect("watcher creation");
        let test_file = dir.join("tracked.txt");
        fs::write(&test_file, "data").unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        watcher.poll(); // process events
        assert!(watcher.is_stale(&test_file));
    }
}
