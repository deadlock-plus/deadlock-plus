use std::borrow::Cow;

use sentry::protocol::{Event, Level, User};

use crate::config::Config;
use crate::gate::Gate;
use crate::scrub::{before_send, MAX_EVENTS_PER_SESSION};

/// `None` when the build has no DSN, which leaves every `capture_*` call a no-op. The panic integration
/// wraps whatever hook is installed when this runs, so call it after the app's own hook is in place.
pub fn init(config: &Config, release: &str, os: &str, gate: Gate, install_id: &str) -> Option<sentry::ClientInitGuard> {
    let dsn = config.sentry_dsn.as_deref()?;
    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: Some(Cow::Owned(release.to_string())),
            send_default_pii: false,
            max_breadcrumbs: 20,
            before_send: Some(before_send(gate, MAX_EVENTS_PER_SESSION)),
            ..Default::default()
        },
    ));
    sentry::configure_scope(|scope| scope.set_tag("os", os));
    set_install_id(install_id);
    Some(guard)
}

pub fn set_install_id(id: &str) {
    sentry::configure_scope(|scope| scope.set_user(Some(User { id: Some(id.to_string()), ..Default::default() })));
}

pub fn capture_webview_error(message: &str, stack: &str) {
    let mut event = Event { level: Level::Error, message: Some(format!("{message}\n{stack}")), ..Default::default() };
    event.tags.insert("source".into(), "webview".into());
    sentry::capture_event(event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_build_without_a_dsn_makes_no_client() {
        assert!(init(&Config::default(), "0.8.0", "windows", Gate::default(), "id").is_none());
    }
}
