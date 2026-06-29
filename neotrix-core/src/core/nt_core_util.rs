/// Returns the current NeoTrix codegen version string.
pub fn codegen_version() -> &'static str {
    "0.1.0"
}

/// Default Tor SOCKS5 proxy port.
pub const TOR_SOCKS_PORT: u16 = 9050;
/// Default Tor control port.
pub const TOR_CONTROL_PORT: u16 = 9051;
/// Default Tor SOCKS5 address.
pub const TOR_SOCKS_ADDR: &str = "127.0.0.1:9050";
/// Default Tor control address.
pub const TOR_CONTROL_ADDR: &str = "127.0.0.1:9051";

/// A2A default server port.
pub const A2A_DEFAULT_PORT: u16 = 42071;
/// A2A health check port.
pub const A2A_HEALTH_PORT: u16 = 42072;
/// A2A metrics / agent server port.
pub const A2A_METRICS_PORT: u16 = 42070;
/// A2A internal / discovery / inter-agent port.
pub const A2A_INTERNAL_PORT: u16 = 42069;

/// Returns the current Unix timestamp in nanoseconds since epoch.
pub fn unix_now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Returns the current Unix timestamp in seconds since epoch.
pub fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Returns the current Unix timestamp in milliseconds since epoch.
pub fn unix_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Returns the user's home directory.
/// Falls back to `/tmp` when `HOME` is unset (e.g. CI environments).
pub fn home_dir() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"))
}

use fs2::FileExt;
use std::fs::File;
use std::path::Path;

/// Advisory cross-process file lock.
/// Drops the lock (and releases the file handle) when the guard goes out of scope.
pub struct FileLock {
    _file: File,
}

impl FileLock {
    /// Acquire an exclusive (write) advisory lock on `path`.
    /// Creates the file if it doesn't exist. Blocks until the lock is acquired.
    pub fn exclusive(path: &Path) -> std::io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(path)?;
        file.lock_exclusive()?;
        Ok(Self { _file: file })
    }

    /// Acquire a shared (read) advisory lock on `path`.
    /// Creates the file if it doesn't exist. Blocks until the lock is acquired.
    pub fn shared(path: &Path) -> std::io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .open(path)?;
        file.lock_shared()?;
        Ok(Self { _file: file })
    }
}

/// Atomically write JSON data to a file using temp+rename.
/// Ensures the file is never left in a partially-written state on crash.
pub fn atomic_write_json<T: serde::Serialize>(path: &Path, data: &T) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("tmp");
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Atomically write text content to a file using temp+rename.
pub fn atomic_write_text(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, content)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Atomically write raw bytes to a file using temp+rename.
pub fn atomic_write_bytes(path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, data)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Test infrastructure: global lock + tempdir helpers
// ---------------------------------------------------------------------------
// Purpose: prevent file lock starvation / tempdir collision when 42+ parallel
// test threads share filesystem state and ~30 global LazyLock<Mutex<...>>
//
// Usage in #[test]:
//   let _g = GLOBAL_TEST_LOCK.lock().unwrap();
//   let tmp = TestDir::new();    // auto-cleaned on drop
//   ...shared state access...
//
// All tests that touch global statics or filesystem should acquire this lock.
// For CI with 3 OS × 42 threads → test-threads=4 + GLOBAL_TEST_LOCK ensures
// zero flaky lock contention.

use std::sync::{LazyLock, Mutex};

/// Global test serialization lock.
/// Guards: filesystem I/O, global statics (OnceLock<Mutex<>>), tempdirs,
///         port binding, and any shared mutable state.
/// Only available under `#[cfg(test)]`.
pub static GLOBAL_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Scoped tempdir that auto-cleans on drop.
/// Wraps `tempfile::TempDir` for convenience.
#[cfg(test)]
pub struct TestDir {
    inner: tempfile::TempDir,
}

#[cfg(test)]
impl TestDir {
    pub fn new() -> Self {
        Self {
            inner: tempfile::tempdir().expect("TestDir: tempdir creation failed"),
        }
    }
    pub fn path(&self) -> &std::path::Path {
        self.inner.path()
    }
    pub fn into_path(self) -> std::path::PathBuf {
        self.inner.keep()
    }
}

#[cfg(test)]
impl Default for TestDir {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a temporary file with given content and return (path, TempDir).
/// The TempDir is returned so it lives as long as the test function scope.
#[cfg(test)]
pub fn test_tempfile(prefix: &str, content: &[u8]) -> (std::path::PathBuf, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("test_tempfile: tempdir");
    let path = dir.path().join(prefix);
    std::fs::write(&path, content).expect("test_tempfile: write");
    (path, dir)
}
