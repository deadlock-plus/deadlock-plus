pub mod commands {
    use dp_gamedata::{ArtCache, GameData, HeroArt, HeroEntry, ItemEntry};
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

    #[tauri::command]
    pub async fn game_hero_art(data: State<'_, GameData>, art: State<'_, ArtCache>) -> Result<Vec<HeroArt>, AppError> {
        let data = data.inner().clone();
        let art = art.inner().clone();
        tauri::async_runtime::spawn_blocking(move || art.hero_art(&data.heroes("en"))).await.map_err(AppError::internal)
    }
}
