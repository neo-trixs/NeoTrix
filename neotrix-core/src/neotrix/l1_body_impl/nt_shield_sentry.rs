use std::sync::LazyLock;

// TODO: inject via DI — pass sentry options through application config
static SENTRY_GUARD: LazyLock<Option<sentry::ClientInitGuard>> = LazyLock::new(|| {
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

pub fn init_sentry() -> &'static Option<sentry::ClientInitGuard> {
    &SENTRY_GUARD
}

pub fn capture_error(msg: &str) {
    if SENTRY_GUARD.is_some() {
        sentry::capture_message(msg, sentry::Level::Error);
    }
}

pub fn capture_error_with_source(msg: &str, source: &str) {
    if SENTRY_GUARD.is_some() {
        sentry::with_scope(
            |scope| { scope.set_tag("source", source); },
            || sentry::capture_message(msg, sentry::Level::Error),
        );
    }
}

pub fn is_active() -> bool {
    SENTRY_GUARD.is_some()
}
