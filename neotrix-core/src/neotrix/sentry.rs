use std::sync::OnceLock;

static SENTRY_GUARD: OnceLock<Option<sentry::ClientInitGuard>> = OnceLock::new();

pub fn init_sentry() -> Option<&'static Option<sentry::ClientInitGuard>> {
    SENTRY_GUARD.get_or_init(|| {
        let dsn = match std::env::var("NEOTRIX_SENTRY_DSN") {
            Ok(dsn) if !dsn.is_empty() => dsn,
            _ => return None,
        };
        sentry::configure_scope(|scope| {
            scope.set_tag("os", std::env::consts::OS);
            scope.set_tag("arch", std::env::consts::ARCH);
            if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
                scope.set_tag("home", &home);
            }
        });
        let guard = sentry::init((
            dsn,
            sentry::ClientOptions {
                release: Some(std::borrow::Cow::Owned(env!("CARGO_PKG_VERSION").to_string())),
                attach_stacktrace: true,
                max_breadcrumbs: 50,
                ..Default::default()
            },
        ));
        Some(guard)
    });
    SENTRY_GUARD.get()
}

pub fn capture_error(msg: &str) {
    if SENTRY_GUARD.get().is_some() {
        sentry::capture_message(msg, sentry::Level::Error);
    }
}

pub fn capture_error_with_source(msg: &str, source: &str) {
    if SENTRY_GUARD.get().is_some() {
        sentry::with_scope(
            |scope| { scope.set_tag("source", source); },
            || sentry::capture_message(msg, sentry::Level::Error),
        );
    }
}

pub fn is_active() -> bool {
    SENTRY_GUARD.get().map(|g| g.is_some()).unwrap_or(false)
}
