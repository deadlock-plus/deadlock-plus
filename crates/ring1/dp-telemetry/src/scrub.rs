use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use sentry::protocol::Event;

use crate::gate::Gate;

/// A crash loop must not burn the Sentry quota, so one run reports a handful of events at most.
pub const MAX_EVENTS_PER_SESSION: usize = 20;

fn redact_all(text: &mut Option<String>) {
    if let Some(text) = text {
        *text = dp_crash::redact(text);
    }
}

/// Everything that can carry a path or a value is rewritten here, after Sentry has finished adding its own
/// context, so nothing it attaches can slip past.
pub fn scrub(mut event: Event<'static>) -> Event<'static> {
    redact_all(&mut event.message);
    for exception in &mut event.exception.values {
        exception.value = exception.value.take().map(|value| dp_crash::redact(&value));
        let frames = exception.stacktrace.iter_mut().chain(exception.raw_stacktrace.iter_mut());
        for frame in frames.flat_map(|trace| trace.frames.iter_mut()) {
            redact_all(&mut frame.abs_path);
            redact_all(&mut frame.filename);
            frame.vars.clear();
            frame.pre_context.clear();
            frame.post_context.clear();
            frame.context_line = None;
        }
    }
    for crumb in &mut event.breadcrumbs.values {
        redact_all(&mut crumb.message);
        crumb.data.clear();
    }
    event.server_name = None;
    event.request = None;
    event.extra.clear();
    event.contexts.remove("device");
    event.user =
        event.user.and_then(|user| user.id).map(|id| sentry::protocol::User { id: Some(id), ..Default::default() });
    event
}

type BeforeSend = Arc<dyn Fn(Event<'static>) -> Option<Event<'static>> + Send + Sync>;

pub fn before_send(gate: Gate, max_events: usize) -> BeforeSend {
    let sent = AtomicUsize::new(0);
    Arc::new(move |event| {
        if !gate.is_open() || sent.fetch_add(1, Ordering::SeqCst) >= max_events {
            return None;
        }
        Some(scrub(event))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentry::protocol::{Breadcrumb, Context, Exception, Frame, Stacktrace, User, Values};

    fn frame(path: &str) -> Frame {
        let mut frame = Frame {
            abs_path: Some(path.into()),
            filename: Some(path.into()),
            context_line: Some("let token = \"abc\";".into()),
            ..Default::default()
        };
        frame.vars.insert("password".into(), "hunter2".into());
        frame
    }

    fn leaky_event() -> Event<'static> {
        let mut event = Event {
            message: Some("failed to open C:\\Users\\Someone\\AppData\\file".into()),
            server_name: Some("SOMEONES-PC".into()),
            ..Default::default()
        };
        event.user = Some(User {
            id: Some("install-1".into()),
            email: Some("a@b.c".into()),
            username: Some("someone".into()),
            ip_address: Some(sentry::protocol::IpAddress::Auto),
            ..Default::default()
        });
        event.exception = Values::from(vec![Exception {
            ty: "Panic".into(),
            value: Some("token=abcdef at /home/someone/code".into()),
            stacktrace: Some(Stacktrace {
                frames: vec![frame("C:\\Users\\Someone\\src\\main.rs")],
                ..Default::default()
            }),
            ..Default::default()
        }]);
        event.breadcrumbs = Values::from(vec![Breadcrumb {
            message: Some("opened C:\\Users\\Someone\\Documents".into()),
            ..Default::default()
        }]);
        event.contexts.insert("device".into(), Context::Device(Box::default()));
        event
    }

    #[test]
    fn user_name_paths_and_secrets_are_masked_everywhere() {
        let event = scrub(leaky_event());
        assert_eq!(event.message.as_deref(), Some("failed to open C:\\Users\\<user>\\AppData\\file"));
        let exception = &event.exception.values[0];
        assert_eq!(exception.value.as_deref(), Some("token=<redacted> at /home/<user>/code"));
        let frame = &exception.stacktrace.as_ref().unwrap().frames[0];
        assert_eq!(frame.abs_path.as_deref(), Some("C:\\Users\\<user>\\src\\main.rs"));
        assert_eq!(frame.filename.as_deref(), Some("C:\\Users\\<user>\\src\\main.rs"));
        assert_eq!(event.breadcrumbs.values[0].message.as_deref(), Some("opened C:\\Users\\<user>\\Documents"));
    }

    #[test]
    fn source_lines_and_frame_variables_are_dropped() {
        let event = scrub(leaky_event());
        let frame = &event.exception.values[0].stacktrace.as_ref().unwrap().frames[0];
        assert!(frame.vars.is_empty());
        assert!(frame.context_line.is_none());
    }

    #[test]
    fn only_the_install_id_survives_as_the_user_and_the_server_name_is_gone() {
        let event = scrub(leaky_event());
        let user = event.user.unwrap();
        assert_eq!(user.id.as_deref(), Some("install-1"));
        assert!(user.email.is_none() && user.username.is_none() && user.ip_address.is_none());
        assert!(event.server_name.is_none());
    }

    #[test]
    fn the_device_context_is_removed() {
        assert!(!scrub(leaky_event()).contexts.contains_key("device"));
    }

    #[test]
    fn nothing_is_sent_while_the_gate_is_closed() {
        let hook = before_send(Gate::default(), 5);
        assert!(hook(leaky_event()).is_none());
    }

    #[test]
    fn an_open_gate_passes_a_scrubbed_event() {
        let gate = Gate::default();
        gate.set(true);
        let event = before_send(gate, 5)(leaky_event()).unwrap();
        assert!(event.server_name.is_none());
    }

    #[test]
    fn a_session_stops_after_its_cap_and_closed_events_do_not_count() {
        let gate = Gate::default();
        gate.set(true);
        let hook = before_send(gate.clone(), 2);
        gate.set(false);
        assert!(hook(leaky_event()).is_none());
        gate.set(true);
        assert!(hook(leaky_event()).is_some());
        assert!(hook(leaky_event()).is_some());
        assert!(hook(leaky_event()).is_none());
    }
}
