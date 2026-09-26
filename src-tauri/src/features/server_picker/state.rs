pub struct ServerPickerState {
    pub http: reqwest::Client,
}

impl Default for ServerPickerState {
    fn default() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("deadlock-plus")
                .build()
                .expect("reqwest client builds with static config"),
        }
    }
}
