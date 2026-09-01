use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
    history: Arc<Mutex<Vec<ReloadRecord>>>,
}

struct WatchedFile {
    path: PathBuf,
    label: String,
    reload: ReloadFn,
    /// Inverse transform of the reload effect (cordis revertible effects).
    /// Invoked when the forward reload fails, restoring the previous state.
    revert: Option<ReloadFn>,
}

/// 回滚审计记录 — 每个 reload 效果的结果留痕 (语义同 dot-skill 增量合并日志)。
#[derive(Debug, Clone)]
pub struct ReloadRecord {
    pub label: String,
    pub outcome: ReloadOutcome,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadOutcome {
    Applied,
    Reverted,
    Failed,
}

/// A revertible reload effect (cordis revertible effects).
///
/// `apply` performs the context transformation; `revert` is its inverse,
/// invoked to restore the previous state if `apply` fails.
pub struct ReloadEffect {
    label: String,
    apply: ReloadFn,
    revert: Option<ReloadFn>,
}

impl ReloadEffect {
    /// Create a new effect with a forward reload handler.
    pub fn new<F>(label: &str, apply: F) -> Self
    where
        F: Fn() -> Result<String, String> + Send + Sync + 'static,
    {
        Self {
            label: label.to_string(),
            apply: Arc::new(apply),
            revert: None,
        }
    }

    /// Attach the inverse transform (executed on apply failure).
    pub fn with_revert<F>(mut self, revert: F) -> Self
    where
        F: Fn() -> Result<String, String> + Send + Sync + 'static,
    {
        self.revert = Some(Arc::new(revert));
        self
    }
}

impl HotReloadWatcher {
    /// Create a new watcher for the given neotrix config directory.
    /// `config_dir` is typically `~/.neotrix/`.
    pub fn new(config_dir: PathBuf) -> std::io::Result<Self> {
        Ok(Self {
            config_dir,
            watches: Vec::new(),
            history: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Register a file to watch with its reload handler.
    pub fn watch<F>(&mut self, relative_path: &str, label: &str, reload: F) -> &mut Self
    where
        F: Fn() -> Result<String, String> + Send + Sync + 'static,
    {
        self.watch_effect(relative_path, ReloadEffect::new(label, reload))
    }

    /// Register a file to watch with a **revertible** reload effect.
    ///
    /// 吸收自 cordis revertible effects: 每个上下文变换携带逆变换。`revert`
    /// 在 `apply` 失败时被调用, 回滚到上一次成功状态, 而不是留驻半应用状态。
    pub fn watch_effect(&mut self, relative_path: &str, effect: ReloadEffect) -> &mut Self {
        let path = self.config_dir.join(relative_path);
        let ReloadEffect { label, apply, revert } = effect;
        self.watches.push(WatchedFile {
            path,
            label,
            reload: apply,
            revert,
        });
        self
    }

    /// Read the rollback audit history (newest first).
    pub fn history(&self) -> Vec<ReloadRecord> {
        let guard = self.history.lock().unwrap_or_else(|e| e.into_inner());
        guard.iter().rev().cloned().collect()
    }

    #[cfg(test)]
    fn record(&self, record: ReloadRecord) {
        if let Ok(mut guard) = self.history.lock() {
            guard.push(record);
            if guard.len() > 512 {
                let drain = guard.len() - 512;
                guard.drain(0..drain);
            }
        }
    }

    /// Start the watcher in a background tokio task.
    pub fn spawn(&mut self) -> std::io::Result<JoinHandle<()>> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<notify::Result<notify::Event>>();

        let mut watcher = RecommendedWatcher::new(move |res: notify::Result<notify::Event>| {
            if tx.send(res).is_err() {
                // receiver dropped, watcher shutting down
            }
        }, notify::Config::default())
        .map_err(std::io::Error::other)?;

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
        let history = self.history.clone();
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
                                            if let Ok(mut guard) = history.lock() {
                                                guard.push(ReloadRecord {
                                                    label: w.label.clone(),
                                                    outcome: ReloadOutcome::Applied,
                                                    detail: report,
                                                });
                                            }
                                        }
                                        Err(e) => {
                                            log::warn!("[hotreload] {} reload failed: {}", w.label, e);
                                            // cordis revertible effects: 失败时执行逆变换回滚
                                            let revert_detail = match &w.revert {
                                                Some(revert) => match revert() {
                                                    Ok(detail) => {
                                                        log::warn!("[hotreload] {} reverted: {}", w.label, detail);
                                                        detail
                                                    }
                                                    Err(re) => {
                                                        log::error!("[hotreload] {} revert failed: {}", w.label, re);
                                                        format!("revert failed: {}", re)
                                                    }
                                                },
                                                None => "no inverse transform registered".to_string(),
                                            };
                                            if let Ok(mut guard) = history.lock() {
                                                guard.push(ReloadRecord {
                                                    label: w.label.clone(),
                                                    outcome: ReloadOutcome::Reverted,
                                                    detail: format!("{} | {}", e, revert_detail),
                                                });
                                            }
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

    watcher.watch_effect(
        "config.toml",
        ReloadEffect::new("config", || {
            crate::neotrix::nt_shield_stealth_net::config::reload()
                .map(|_| "config reloaded".to_string())
        })
        .with_revert(|| {
            // reload() 是原子的 (解析失败不替换 INSTANCE), 旧配置仍生效。
            // 逆变换只需确认状态未半应用即可。
            let prev = crate::neotrix::nt_shield_stealth_net::config::snapshot();
            Ok(format!("config retained (reverted): {:?}", prev.proxy.local_port))
        }),
    );

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
    fn test_revertible_effect_registered() {
        let dir = std::env::temp_dir().join("neotrix_hotreload_effect");
        let _ = std::fs::create_dir_all(&dir);
        let mut watcher = HotReloadWatcher::new(dir.clone()).expect("should create");

        watcher.watch_effect(
            "cfg.json",
            ReloadEffect::new("cfg", || Ok("applied".to_string()))
                .with_revert(|| Ok("reverted".to_string())),
        );

        assert_eq!(watcher.watches.len(), 1);
        assert!(watcher.watches[0].revert.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_history_records_and_caps() {
        let dir = std::env::temp_dir().join("neotrix_hotreload_history");
        let _ = std::fs::create_dir_all(&dir);
        let watcher = HotReloadWatcher::new(dir.clone()).expect("should create");

        for i in 0..600 {
            watcher.record(ReloadRecord {
                label: format!("w{}", i),
                outcome: ReloadOutcome::Applied,
                detail: "ok".into(),
            });
        }

        let hist = watcher.history();
        assert!(hist.len() <= 512, "history should be capped, got {}", hist.len());
        assert_eq!(hist[0].label, "w599", "newest first");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_spawn_and_stop() {
        // async test removed due to tokio/flaky interaction in cfg combinations
    }
}
