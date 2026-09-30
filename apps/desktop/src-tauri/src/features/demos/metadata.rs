pub mod commands {
    use dp_demos::metadata::{lookup, DemoMetaCache, MetaResult};
    use tauri::{Manager, State};

    #[tauri::command]
    pub async fn demo_metadata(
        app: tauri::AppHandle,
        cache: State<'_, DemoMetaCache>,
        match_id: u64,
    ) -> Result<MetaResult, String> {
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        Ok(lookup(&cache, &dir, match_id).await)
    }
}
