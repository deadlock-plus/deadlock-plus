pub mod commands {
    use tauri_plugin_dialog::DialogExt;

    /// Asks the user where to save, then writes there. Returns false when the dialog is cancelled.
    #[tauri::command]
    pub async fn save_text_file(
        app: tauri::AppHandle,
        default_name: String,
        filter_name: String,
        extension: String,
        contents: String,
    ) -> Result<bool, String> {
        dp_export::validate_request(&default_name, &extension)?;
        let picked = tauri::async_runtime::spawn_blocking(move || {
            app.dialog()
                .file()
                .set_file_name(default_name)
                .add_filter(filter_name, &[extension.as_str()])
                .blocking_save_file()
        })
        .await
        .map_err(|e| e.to_string())?;
        let Some(picked) = picked else { return Ok(false) };
        let path = picked.into_path().map_err(|e| e.to_string())?;
        dp_export::write_export(&path, &contents)?;
        Ok(true)
    }
}
