#[cfg(not(target_os = "macos"))]
use crate::project::config;

pub fn init(enabled: bool) -> Option<sentry::ClientInitGuard> {
    #[cfg(target_os = "macos")]
    {
        // MacUtl is an unofficial fork. The upstream project's GlitchTip DSN
        // must never receive data from this build, even if the user enables
        // the crash-reporting toggle. macOS therefore never initializes the
        // Sentry client; capture_error / capture_message remain no-ops.
        let _ = enabled;
        None
    }
    #[cfg(not(target_os = "macos"))]
    {
        if !enabled {
            return None;
        }
        let options = sentry::ClientOptions::new()
            .dsn(config::SENTRY_DSN)
            .maybe_release(sentry::release_name!())
            .traces_sample_rate(0.01);
        Some(sentry::init(options))
    }
}

pub fn capture_error(err: &dyn std::error::Error) {
    sentry::capture_error(err);
}

pub fn capture_message(msg: &str) {
    sentry::capture_message(msg, sentry::Level::Error);
}
