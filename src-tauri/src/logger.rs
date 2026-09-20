use std::io;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize structured logging with tracing.
/// - stderr: human-readable, colorized in dev, plain in release
/// - file: JSON structured logs in app data dir, rotated daily
pub fn init_logging(app_handle: &tauri::AppHandle) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("neotrix_tauri=info,warn"));

    let stderr_layer = fmt::layer()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_writer(io::stderr)
        .with_filter(env_filter);

    let log_dir = app_handle
        .path()
        .app_log_dir()
        .unwrap_or_else(|e| {
            tracing::error!("failed to resolve log dir: {e}");
            std::path::PathBuf::from(".")
        });
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = rolling::daily(&log_dir, "neotrix.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    Box::leak(Box::new(_guard));

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .json();

    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %log_dir.display(),
        "NeoTrix logging initialized"
    );
}
