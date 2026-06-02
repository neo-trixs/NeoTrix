//! NeoTrix 日志工具 — tracing 集成 + 轻量级 fallback
//!
//! 使用 tracing_subscriber 初始化，替代 println!/eprintln!
//!
//! 使用方式:
//!   log_info!("模块名", "消息 {}", arg);
//!   log_warn!("模块名", "警告信息");
//!   log_error!("模块名", "错误: {}", err);

pub use tracing::{info, warn, error, debug};

/// 初始化 tracing 日志（带环境变量过滤）
pub fn init_tracing() {
    use tracing_subscriber::fmt;
    fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neotrix=info".into())
        )
        .with_target(true)
        .init();
}

use std::sync::atomic::{AtomicU8, Ordering};

const LEVEL_ERROR: u8 = 0;
const LEVEL_WARN: u8 = 1;
const LEVEL_INFO: u8 = 2;
const LEVEL_DEBUG: u8 = 3;

static LOG_LEVEL: AtomicU8 = AtomicU8::new(LEVEL_INFO);

pub fn set_level(level: &str) {
    let lvl = match level.to_lowercase().as_str() {
        "error" => LEVEL_ERROR,
        "warn" | "warning" => LEVEL_WARN,
        "info" => LEVEL_INFO,
        "debug" => LEVEL_DEBUG,
        _ => LEVEL_INFO,
    };
    LOG_LEVEL.store(lvl, Ordering::Relaxed);
}

fn should_log(level: u8) -> bool {
    level <= LOG_LEVEL.load(Ordering::Relaxed)
}

fn level_prefix(level: u8) -> &'static str {
    match level {
        LEVEL_ERROR => "ERROR",
        LEVEL_WARN => " WARN",
        LEVEL_INFO => " INFO",
        LEVEL_DEBUG => "DEBUG",
        _ => "?????",
    }
}

pub fn log(level: u8, module: &str, msg: &str) {
    if should_log(level) {
        eprintln!("[{}] [{}] {}", level_prefix(level), module, msg);
    }
}

#[macro_export]
macro_rules! log_error {
    ($module:expr, $($arg:tt)*) => {
        $crate::neotrix::logging::log(
            $crate::neotrix::logging::LEVEL_ERROR, $module,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($module:expr, $($arg:tt)*) => {
        $crate::neotrix::logging::log(
            $crate::neotrix::logging::LEVEL_WARN, $module,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_info {
    ($module:expr, $($arg:tt)*) => {
        $crate::neotrix::logging::log(
            $crate::neotrix::logging::LEVEL_INFO, $module,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_debug {
    ($module:expr, $($arg:tt)*) => {
        $crate::neotrix::logging::log(
            $crate::neotrix::logging::LEVEL_DEBUG, $module,
            &format!($($arg)*)
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_filtering() {
        set_level("warn");
        assert!(!should_log(LEVEL_DEBUG));
        assert!(!should_log(LEVEL_INFO));
        assert!(should_log(LEVEL_WARN));
        assert!(should_log(LEVEL_ERROR));

        set_level("debug");
        assert!(should_log(LEVEL_DEBUG));
    }

    #[test]
    fn test_level_prefixes() {
        assert_eq!(level_prefix(LEVEL_ERROR), "ERROR");
        assert_eq!(level_prefix(LEVEL_WARN), " WARN");
        assert_eq!(level_prefix(LEVEL_INFO), " INFO");
        assert_eq!(level_prefix(LEVEL_DEBUG), "DEBUG");
    }
}
