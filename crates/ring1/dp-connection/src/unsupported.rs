use std::sync::mpsc::{self, Sender};

use super::{Config, Status};

/// Returns a sender; sending on it, or dropping it, stops the source.
pub fn start(config: Config) -> Sender<()> {
    (config.on_status)(Status::Failed("Connection monitoring is not supported on this platform yet.".into()));
    mpsc::channel().0
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn reports_that_it_cannot_run_and_delivers_nothing() {
        let status = Arc::new(Mutex::new(None));
        let sink_status = status.clone();
        let stop = start(Config {
            wanted: Arc::new(|_| true),
            sink: Arc::new(|_| panic!("no packets expected")),
            remotes: Arc::new(Vec::new),
            on_status: Box::new(move |s| *sink_status.lock().unwrap() = Some(s)),
            prompt: true,
        });
        assert!(matches!(status.lock().unwrap().as_ref(), Some(Status::Failed(m)) if m.contains("not supported")));
        assert!(stop.send(()).is_err());
    }
}
