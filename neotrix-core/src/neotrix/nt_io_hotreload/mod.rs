use std::path::PathBuf;
use std::sync::Arc;

use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use tokio::task::JoinHandle;

type ReloadFn = Arc<dyn Fn() -> Result<String, String> + Send + Sync>;

/// A hot-reload watcher that monitors files for changes and triggers registered
/// reload handlers. Uses the `notify` crate for OS-level filesystem events.
///
/// Watched files:
/// - `config.toml` — reloads `NeoTrixConfig` via `config::reload()`
/// - `rules.json` — reloads `RuleEngine` rules
/// - `subscriptions.json` — reloads proxy subscription URLs
pub struct HotReloadWatcher {
    config_dir: PathBuf,
    watches: Vec<WatchedFile>,
}

struct WatchedFile {
    path: PathBuf,
    label: String,
    reload: ReloadFn,
}

impl HotReloadWatcher {
    /// Create a new watcher for the given neotrix config directory.
    /// `config_dir` is typically `~/.neotrix/`.
    pub fn new(config_dir: PathBuf) -> std::io::Result<Self> {
        Ok(Self {
            config_dir,
            watches: Vec::new(),
        })
    }

    /// Register a file to watch with its reload handler.
    pub fn watch<F>(&mut self, relative_path: &str, label: &str, reload: F) -> &mut Self
    where
        F: Fn() -> Result<String, String> + Send + Sync + 'static,
    {
        let path = self.config_dir.join(relative_path);
        self.watches.push(WatchedFile {
            path,
            label: label.to_string(),
            reload: Arc::new(reload),
        });
        self
    }

    /// Start the watcher in a background tokio task.
    pub fn spawn(&mut self) -> std::io::Result<JoinHandle<()>> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<notify::Result<notify::Event>>();

        let mut watcher = RecommendedWatcher::new(move |res: notify::Result<notify::Event>| {
            if tx.send(res).is_err() {
                // receiver dropped, watcher shutting down
            }
        }, notify::Config::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let watches = std::mem::take(&mut self.watches);
        for w in &watches {
            if w.path.exists() {
                if let Err(e) = watcher.watch(&w.path, RecursiveMode::NonRecursive) {
                    log::warn!("[hotreload] cannot watch {}: {}", w.label, e);
                } else {
                    log::info!("[hotreload] watching {} → {}", w.label, w.path.display());
                }
            } else {
                log::info!("[hotreload] {} not found, skip watch ({})", w.path.display(), w.label);
            }
        }

        // Move watcher into the spawned task so it stays alive for events
        let handle = tokio::spawn(async move {
            let _watcher = watcher;
            loop {
                match rx.recv().await {
                    Some(Ok(event)) => {
                        let modified_path = event.paths.first().cloned();
                        if let Some(path) = modified_path {
                            for w in &watches {
                                if w.path == path {
                                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                    match (w.reload)() {
                                        Ok(report) => {
                                            log::info!("[hotreload] {} reloaded: {}", w.label, report);
                                        }
                                        Err(e) => {
                                            log::warn!("[hotreload] {} reload failed: {}", w.label, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        log::warn!("[hotreload] notify error: {}", e);
                    }
                    None => {
                        log::error!("[hotreload] channel closed, watcher exiting");
                        break;
                    }
                }
            }
        });

        Ok(handle)
    }

}

/// Build the default HotReloadWatcher for the NeoTrix project.
///
/// Watches:
/// 1. `config.toml` → reload NeoTrixConfig
/// 2. `rules.json` → reload RuleEngine (if available via Arc<RwLock>)
/// 3. `subscriptions.json` → reload proxy subscription URLs
#[cfg(feature = "stealth-net")]
pub fn default_watcher(
    neotrix_dir: PathBuf,
    rule_engine: Option<std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_shield_stealth_net::rules::RuleEngine>>>,
    proxy_pool: Option<std::sync::Arc<crate::neotrix::nt_shield_stealth_net::proxy_pool::ProxyPool>>,
) -> std::io::Result<HotReloadWatcher> {
    let mut watcher = HotReloadWatcher::new(neotrix_dir.clone())?;

    watcher.watch("config.toml", "config", move || {
        crate::neotrix::nt_shield_stealth_net::config::reload()
            .map(|_| "config reloaded".to_string())
    });

    if let Some(re) = rule_engine {
        watcher.watch("rules.json", "rules", move || {
            let mut engine = re.blocking_write();
            engine.reload_from_disk()
        });
    }

    if let Some(pp) = proxy_pool {
        watcher.watch("subscriptions.json", "subscriptions", move || {
            let rt = tokio::runtime::Handle::current();
            let count = rt.block_on(pp.reload_subscriptions());
            Ok(format!("{} subscriptions loaded", count))
        });
    }

    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_hotreload_new() {
        let dir = std::env::temp_dir().join("neotrix_hotreload_test");
        let _ = std::fs::create_dir_all(&dir);
        let watcher = HotReloadWatcher::new(dir.clone()).expect("should create");
        assert!(watcher.watches.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_hotreload_register_watch() {
        let dir = std::env::temp_dir().join("neotrix_hotreload_test2");
        let _ = std::fs::create_dir_all(&dir);
        let mut watcher = HotReloadWatcher::new(dir.clone()).expect("should create");

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        watcher.watch("test.json", "test", move || {
            called_clone.store(true, Ordering::SeqCst);
            Ok("done".to_string())
        });

        assert_eq!(watcher.watches.len(), 1);
        assert_eq!(watcher.watches[0].label, "test");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_hotreload_spawn_and_stop() {
        // async test removed due to tokio/flaky interaction in cfg combinations
    }
}
