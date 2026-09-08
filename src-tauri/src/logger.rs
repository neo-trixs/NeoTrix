use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::rolling;

pub fn init() {
    let log_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".neotrix")
        .join("logs");

    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = rolling::daily(&log_dir, "desktop.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let _ = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_writer(non_blocking)
        .compact()
        .try_init();

    // Keep guard alive for the program duration
    Box::leak(Box::new(_guard));
}
