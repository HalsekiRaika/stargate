use std::path::Path;
use tracing_subscriber::{EnvFilter, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Setup logging system for the server.
///
/// It should be noted that return value `WorkerGuard` should not be dropped.
/// If you drop WorkerGuard, the logging of [`tracing_appender`] will not work.
///
/// ```rust,ignore
/// #[tokio::main]
/// async fn main() {
///     // `_` would immediately Drop the return value.
///     // let _ = server::logging::setup();
///
///     // In this way, unused values suppressed lint in clippy, etc.
///     // but are not dropped.
///     let _guard = server::logging::setup();
/// }
/// ```
pub fn setup() -> tracing_appender::non_blocking::WorkerGuard {
    let appender = tracing_appender::rolling::daily(Path::new("./.logs/"), "trace.log");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(appender);
    
    let directive = std::env::var("SERVER_LOG")
        .unwrap_or_else(|_| "trace".into());
    
    tracing_subscriber::registry()
        .with(tracing_error::ErrorLayer::default())
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(EnvFilter::new(directive)),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_writer(non_blocking_appender)
                .with_ansi(false)
                .with_filter(EnvFilter::new("trace"))
        )
        .init();
    guard
}
