pub mod commands {
    use dp_voice_bans::{backup_then_write, current_path, game_running_recent, looks_like_voice_ban, VoiceBanFile};

    #[tauri::command]
    pub fn is_game_running() -> bool {
        game_running_recent()
    }

    #[tauri::command]
    pub fn read_voice_ban() -> Result<VoiceBanFile, String> {
        let path = current_path()?;
        let game_running = game_running_recent();
        let (exists, text) = match std::fs::read(&path) {
            Ok(bytes) => (true, String::from_utf8(bytes).map_err(|_| "voice_ban.dt is not valid UTF-8.".to_string())?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (false, String::new()),
            Err(e) => {
                log::error!("could not read voice_ban.dt: {e}");
                return Err(format!("Failed to read {}: {e}", path.display()));
            }
        };
        Ok(VoiceBanFile { path: path.to_string_lossy().into_owned(), exists, text, game_running })
    }

    /// The target is always the current account's own file; the frontend never supplies a path.
    #[tauri::command]
    pub fn write_voice_ban(text: String) -> Result<String, String> {
        if dp_game::is_running() {
            log::warn!("mute list write refused: Deadlock is running");
            return Err("Deadlock is running. Close the game before changing mutes.".into());
        }
        if !looks_like_voice_ban(text.as_bytes()) {
            log::error!("mute list write refused: the content is not a mute list");
            return Err("Refusing to write: the content is not a mute list.".into());
        }
        let path = current_path()?;
        let timestamp =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs();
        backup_then_write(&path, text.as_bytes(), timestamp).map(|p| p.to_string_lossy().into_owned())
    }
}
