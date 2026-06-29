use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

static PANIC_HOOK_SET: AtomicBool = AtomicBool::new(false);

pub fn install_panic_filter() {
    if PANIC_HOOK_SET.swap(true, Ordering::Relaxed) {
        return;
    }

    let _prev = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let sanitized = sanitize_panic(info);
        log::error!("{}", sanitized);
    }));
}

pub fn sanitize_panic(info: &panic::PanicHookInfo<'_>) -> String {
    let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "internal fault".to_string()
    };

    let location = info
        .location()
        .map(|l| {
            let file = l.file();
            let sanitized = file
                .rsplit_once('/')
                .map(|(_, basename)| basename)
                .or_else(|| file.rsplit_once('\\').map(|(_, basename)| basename))
                .unwrap_or(file);
            format!("{}:{}", sanitized, l.line())
        })
        .unwrap_or_default();

    format!("[fault] {} @ {}", msg, location)
}

pub struct SafeError {
    inner: String,
}

impl SafeError {
    pub fn new(msg: &str) -> Self {
        Self {
            inner: msg.to_string(),
        }
    }

    pub fn message(&self) -> &str {
        &self.inner
    }
}

impl std::fmt::Display for SafeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl std::fmt::Debug for SafeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SafeError(\"**\")")
    }
}

impl std::error::Error for SafeError {}

pub fn strip_source_path(path: &str) -> String {
    path.split('/')
        .last()
        .or_else(|| path.split('\\').last())
        .unwrap_or(path)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_source_path() {
        let path = "/Users/neo/neotrix/src/core/mod.rs";
        let stripped = strip_source_path(path);
        assert_eq!(stripped, "mod.rs");
    }

    #[test]
    fn test_safe_error_debug_hides_path() {
        let err = SafeError::new("test error");
        let debug_str = format!("{:?}", err);
        assert_eq!(debug_str, "SafeError(\"**\")");
    }

    #[test]
    fn test_safe_error_display() {
        let err = SafeError::new("test error");
        assert_eq!(err.message(), "test error");
        assert_eq!(format!("{}", err), "test error");
    }
}
