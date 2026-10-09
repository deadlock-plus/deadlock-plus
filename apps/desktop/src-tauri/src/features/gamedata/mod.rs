pub mod commands {
    use dp_gamedata::{GameData, HeroEntry, ItemEntry};
    use tauri::State;

    use crate::features::error::AppError;

    #[tauri::command]
    pub async fn game_heroes(data: State<'_, GameData>, locale: String) -> Result<Vec<HeroEntry>, AppError> {
        let data = data.inner().clone();
        let list =
            tauri::async_runtime::spawn_blocking(move || data.heroes(&locale)).await.map_err(AppError::internal)?;
        Ok(list.to_vec())
    }

    #[tauri::command]
    pub async fn game_items(data: State<'_, GameData>, locale: String) -> Result<Vec<ItemEntry>, AppError> {
        let data = data.inner().clone();
        let list =
            tauri::async_runtime::spawn_blocking(move || data.items(&locale)).await.map_err(AppError::internal)?;
        Ok(list.to_vec())
    }
}
