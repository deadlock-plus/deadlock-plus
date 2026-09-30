pub struct ServerPickerState {
    pub sync_lock: tokio::sync::Mutex<()>,
    /// Wakes the background sync ahead of its interval, e.g. when its switch turns on.
    pub sync_now: std::sync::Arc<tokio::sync::Notify>,
}

impl Default for ServerPickerState {
    fn default() -> Self {
        Self { sync_lock: tokio::sync::Mutex::new(()), sync_now: Default::default() }
    }
}
