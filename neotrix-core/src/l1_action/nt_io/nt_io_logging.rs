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
///
/// 2026-10-05 修：**日志必须走 stderr**。
///
/// ## 缺陷实测（本窗口用 `neotrix web fetch | python3 -c json.loads` 时撞上）
/// `fmt()` **默认写 stdout** ⇒ 任何 CLI 的标准输出都被日志行污染：
/// ```text
/// $ neotrix web fetch <json-api> 2>/dev/null | python3 -c "import sys,json; json.load(sys.stdin)"
/// 2026-10-05T00:55:42.073244Z ERROR neotrix::…: [universal-browser] CDP handler error: …
/// json.decoder.JSONDecodeError: Expecting value: line 1 column 1 (char 0)
/// ```
/// ⇒ `web fetch | jq`、`| python3 -c json.loads` 这类**标准用法全部不可用**。
/// ⚠️ 注意 `2>/dev/null` **滤不掉**它：tracing 走的是 stdout，不是 stderr。
///
/// ## 判据
/// 通用 CLI 契约：**stdout 是数据通道，stderr 是诊断通道**。
/// 日志是诊断 ⇒ 落 stderr。这也是 `jq`/`grep`/`python3 -c` 能安全接管管道的前提。
pub fn init_tracing() {
    use tracing_subscriber::fmt;
    fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neotrix=info".into())
        )
        .with_target(true)
        // ⛔ 不显式指定 writer，tracing_subscriber::fmt 默认 = stdout ⇒ 污染管道。
        .with_writer(std::io::stderr)
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
        $super::nt_io_logging::log(
            $super::nt_io_logging::LEVEL_ERROR, $module,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($module:expr, $($arg:tt)*) => {
        $super::nt_io_logging::log(
            $super::nt_io_logging::LEVEL_WARN, $module,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_info {
    ($module:expr, $($arg:tt)*) => {
        $super::nt_io_logging::log(
            $super::nt_io_logging::LEVEL_INFO, $module,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_debug {
    ($module:expr, $($arg:tt)*) => {
        $super::nt_io_logging::log(
            $super::nt_io_logging::LEVEL_DEBUG, $module,
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

#[cfg(test)]
mod ansi_pipe_tests {
    //! 反向锁：**日志不得写 stdout**。
    //!
    //! 缺陷实测：`neotrix web fetch <json> | json.loads` 因日志行混入而失败，
    //! 且 `2>/dev/null` 滤不掉（tracing 默认 writer 就是 stdout）。
    //!
    //! ⛔ 本测试**不能**直接断言 `init_tracing()` 的 writer —— 它是
    //! process-global（`.init()` 只能调一次，且会被其它测试抢先）。
    //! ⇒ 改为锁**判据本身**：源码里必须显式出现 `with_writer`。
    //!   这条锁的价值在于：有人删掉 `with_writer` 时它立刻红。

    /// 守住 `init_tracing` 显式指定 `stderr` writer。
    #[test]
    fn init_tracing_must_target_stderr() {
        let src = include_str!("nt_io_logging.rs");
        let body = src
            .split("pub fn init_tracing()")
            .nth(1)
            .expect("应能找到 init_tracing");
        let body = body.split('\n').take(20).collect::<Vec<_>>().join("\n");
        assert!(
            body.contains("with_writer"),
            "init_tracing 必须显式 .with_writer(..)：tracing_subscriber::fmt 默认写 stdout \
             ⇒ 会污染 CLI 管道输出（实测 web fetch | json.loads 失败）"
        );
        assert!(
            body.contains("stderr"),
            "writer 应为 stderr（stdout 是数据通道，日志是诊断通道）"
        );
    }

    /// 反向锁：确认本 crate 存在这条约定，而非凭空断言。
    #[test]
    fn stderr_writer_is_the_documented_contract() {
        let src = include_str!("nt_io_logging.rs");
        assert!(
            src.contains("stdout 是数据通道"),
            "应保留判据说明，避免后人误以为日志走 stdout 是无害的"
        );
    }
}
