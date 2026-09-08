# NeoTrix Tauri Optimization Plan

## Executive Summary

Based on comprehensive analysis of the current codebase (~6,600 lines) and Tauri 2.x best practices research, this plan outlines iterative optimizations across 5 phases. Each phase delivers measurable improvements while building toward the ultimate goal: a production-grade, self-evolving desktop AI toolkit.

---

## Phase 1: Critical Fixes (Week 1)

### 1.1 Remove Dead Code & Fix Architecture

**Issues Found:**
- Root `main.rs` (8 lines) is dead code — `src/main.rs` is the real entry
- `config.rs` (546 lines) has full `NeoTrixConfig` but isn't wired into runtime
- 8 of 12 domain plugins use stub implementations while real implementations exist
- `request_permission`/`respond_permission` commands defined but not registered

**Actions:**
```rust
// Delete root main.rs (it's not compiled)
// Wire NeoTrixConfig into runtime as managed state
// Replace stub plugins with real implementations
// Register permission commands in invoke_handler
```

### 1.2 SQLite Connection Pooling

**Current:** `open_db()` called on every operation — 50+ connections per session

**Fix:** Implement `SqlitePool` with `r2d2` or `deadpool-sqlite`:
```rust
pub struct DbPool {
    pool: Arc<r2d2::Pool<SqliteConnectionManager>>,
}

impl DbPool {
    pub fn new(path: &Path) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path)
            .with_flags(rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE);
        let pool = r2d2::Pool::builder()
            .max_size(10)
            .build(manager)?;
        Ok(Self { pool: Arc::new(pool) })
    }
}
```

### 1.3 Config Caching

**Current:** File read from disk on every `config_get` call

**Fix:** `RwLock<NeoTrixConfig>` with lazy loading:
```rust
pub struct CachedConfig {
    inner: RwLock<Option<NeoTrixConfig>>,
    path: PathBuf,
}

impl CachedConfig {
    pub fn get(&self) -> Result<NeoTrixConfig> {
        if let Some(ref cfg) = *self.inner.read().unwrap() {
            return Ok(cfg.clone());
        }
        let cfg = NeoTrixConfig::load(&self.path)?;
        *self.inner.write().unwrap() = Some(cfg.clone());
        Ok(cfg)
    }
}
```

---

## Phase 2: Performance Optimization (Week 2)

### 2.1 IPC Optimization

**Current:** Single `domain_call` entry point with JSON serialization overhead

**Optimizations:**
1. **Binary IPC for large payloads** — Use `tauri::ipc::Channel` for streaming
2. **Batch commands** — Group multiple operations in single IPC call
3. **Response caching** — Cache frequent read-only queries

```rust
#[tauri::command]
async fn domain_call_batch(
    calls: Vec<DomainCallRequest>,
    state: State<'_, DomainState>,
) -> Result<Vec<DomainResponse>, String> {
    let registry = state.0.read().await;
    let mut results = Vec::with_capacity(calls.len());
    for call in calls {
        results.push(registry.call(&call.domain, &call.action, &call.args).await);
    }
    Ok(results)
}
```

### 2.2 Lazy Plugin Initialization

**Current:** All 12 plugins initialized at startup

**Fix:** Lazy-load plugins on first use:
```rust
pub struct LazyPluginRegistry {
    plugins: DashMap<String, Box<dyn DomainPlugin>>,
    init_status: DashMap<String, bool>,
}

impl LazyPluginRegistry {
    pub async fn call(&self, domain: &str, action: &str, args: Value) -> DomainResponse {
        if !self.init_status.get(domain).unwrap_or(&false) {
            self.init_plugin(domain).await;
        }
        // ... call plugin
    }
}
```

### 2.3 Memory Optimization

**Current:** Each domain plugin holds its own state

**Fix:** Shared state with `Arc` and `DashMap`:
```rust
pub struct SharedState {
    pub sessions: Arc<DashMap<String, Session>>,
    pub kb: Arc<SqlitePool>,
    pub config: Arc<CachedConfig>,
}
```

---

## Phase 3: Security Hardening (Week 3)

### 3.1 Permission System

**Current:** Stub permission commands

**Fix:** Implement proper Tauri 2.x capability-based permissions:
```toml
# capabilities/desktop.json
{
  "identifier": "desktop-capability",
  "description": "Desktop app capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "fs:allow-read",
    "fs:allow-write",
    "dialog:allow-open",
    "dialog:allow-save"
  ]
}
```

### 3.2 Input Validation

**Current:** Basic path sanitization in `FilePlugin`

**Fix:** Comprehensive input validation:
```rust
pub struct InputValidator;

impl InputValidator {
    pub fn validate_domain(domain: &str) -> Result<()> {
        if !domain.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(DomainError::invalid_input("Invalid domain name"));
        }
        Ok(())
    }

    pub fn validate_action(action: &str) -> Result<()> {
        if action.len() > 100 || action.contains('\0') {
            return Err(DomainError::invalid_input("Invalid action"));
        }
        Ok(())
    }
}
```

### 3.3 Secure IPC

**Fix:** Add IPC message signing and validation:
```rust
#[tauri::command]
async fn secure_domain_call(
    request: SignedDomainRequest,
    state: State<'_, DomainState>,
) -> Result<DomainResponse, String> {
    // Verify signature
    request.verify_signature(&state.0.public_key)?;
    // Rate limiting
    state.1.check_rate_limit(&request.client_id)?;
    // Call domain
    state.0.call(&request.domain, &request.action, &request.args).await
}
```

---

## Phase 4: Developer Experience (Week 4)

### 4.1 Hot Reload

**Current:** Manual restart for Rust changes

**Fix:** Implement hot reload for plugins:
```rust
pub struct HotReloadManager {
    watch_dirs: Vec<PathBuf>,
    plugin_paths: DashMap<String, PathBuf>,
}

impl HotReloadManager {
    pub async fn watch(&self) {
        let mut watcher = notify::recommended_watcher(|res| {
            // Reload plugin on file change
        })?;
        for dir in &self.watch_dirs {
            watcher.watch(dir, RecursiveMode::Recursive)?;
        }
    }
}
```

### 4.2 Debug Console

**Fix:** Add built-in debug console:
```rust
pub struct DebugConsole {
    logs: Arc<RwLock<VecDeque<LogEntry>>>,
    max_entries: usize,
}

#[tauri::command]
async fn get_logs(
    state: State<'_, DebugConsole>,
    level: Option<String>,
    limit: Option<usize>,
) -> Vec<LogEntry> {
    let logs = state.logs.read().await;
    logs.iter()
        .filter(|e| level.as_ref().map_or(true, |l| e.level == *l))
        .take(limit.unwrap_or(100))
        .cloned()
        .collect()
}
```

### 4.3 Performance Profiler

**Fix:** Add built-in performance tracking:
```rust
pub struct PerformanceProfiler {
    spans: DashMap<String, Duration>,
}

impl PerformanceProfiler {
    pub fn measure<F, R>(&self, name: &str, f: F) -> R
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        self.spans.insert(name.to_string(), duration);
        result
    }
}
```

---

## Phase 5: Production Readiness (Week 5)

### 5.1 Auto-Update

**Current:** `tauri_plugin_updater` conditionally enabled

**Fix:** Implement proper auto-update with rollback:
```rust
pub struct UpdateManager {
    current_version: String,
    update_url: String,
    backup_dir: PathBuf,
}

impl UpdateManager {
    pub async fn check_and_update(&self) -> Result<UpdateResult> {
        // Check for updates
        // Download and verify signature
        // Create backup
        // Apply update
        // Verify update
        // Rollback if failed
    }
}
```

### 5.2 Crash Recovery

**Fix:** Implement crash recovery with state persistence:
```rust
pub struct CrashRecovery {
    state_dir: PathBuf,
    checkpoint_interval: Duration,
}

impl CrashRecovery {
    pub async fn save_checkpoint(&self, state: &AppState) -> Result<()> {
        let checkpoint = Checkpoint {
            timestamp: Utc::now(),
            sessions: state.sessions.snapshot(),
            kb_state: state.kb.snapshot(),
        };
        let path = self.state_dir.join("checkpoint.json");
        serde_json::to_writer_pretty(File::create(&path)?, &checkpoint)?;
        Ok(())
    }

    pub async fn recover(&self) -> Result<Option<AppState>> {
        let path = self.state_dir.join("checkpoint.json");
        if path.exists() {
            let checkpoint: Checkpoint = serde_json::from_reader(File::open(&path)?)?;
            return Ok(Some(checkpoint.restore()));
        }
        Ok(None)
    }
}
```

### 5.3 Telemetry

**Fix:** Add optional telemetry (opt-in):
```rust
pub struct Telemetry {
    enabled: bool,
    events: mpsc::UnboundedSender<TelemetryEvent>,
}

impl Telemetry {
    pub fn track_event(&self, event: &str, props: HashMap<String, Value>) {
        if !self.enabled { return; }
        let _ = self.events.send(TelemetryEvent {
            event: event.to_string(),
            properties: props,
            timestamp: Utc::now(),
        });
    }
}
```

---

## Success Metrics

| Metric | Current | Target | Phase |
|--------|---------|--------|-------|
| Startup time | ~2s | <500ms | 2 |
| Memory usage | ~200MB | <100MB | 2 |
| SQLite ops/sec | ~100 | >1000 | 1 |
| IPC latency | ~10ms | <2ms | 2 |
| Bundle size | ~50MB | <30MB | 5 |
| Crash recovery | None | Full | 5 |

---

## Implementation Priority

1. **P0 (Critical):** 1.1, 1.2, 1.3 — Fix dead code, pooling, caching
2. **P1 (High):** 2.1, 2.2 — IPC optimization, lazy loading
3. **P2 (Medium):** 3.1, 3.2 — Security hardening
4. **P3 (Low):** 4.1, 4.2 — Developer experience
5. **P4 (Future):** 5.1, 5.2 — Production readiness

---

## Conclusion

This plan transforms NeoTrix from a functional prototype to a production-grade desktop application. Each phase builds on the previous, delivering incremental value while maintaining system stability. The ultimate goal is a self-evolving AI toolkit that rivals commercial offerings in performance, security, and developer experience.
