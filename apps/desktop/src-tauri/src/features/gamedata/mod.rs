pub mod commands {
    use dp_gamedata::{ArtCache, ClassArt, GameData, HeroArt, HeroEntry, ItemEntry, ItemImage, RankArt};
    use tauri::State;

    use crate::features::error::AppError;

    const RANK_TIERS: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

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

    #[tauri::command]
    pub async fn game_rank_art(art: State<'_, ArtCache>) -> Result<Vec<RankArt>, AppError> {
        let art = art.inner().clone();
        tauri::async_runtime::spawn_blocking(move || art.rank_art(&RANK_TIERS)).await.map_err(AppError::internal)
    }

    #[tauri::command]
    pub async fn game_ability_art(art: State<'_, ArtCache>, names: Vec<String>) -> Result<Vec<ClassArt>, AppError> {
        let art = art.inner().clone();
        tauri::async_runtime::spawn_blocking(move || art.ability_art(&names)).await.map_err(AppError::internal)
    }

    #[tauri::command]
    pub async fn game_item_art(
        art: State<'_, ArtCache>,
        names: Vec<String>,
        kind: ItemImage,
    ) -> Result<Vec<ClassArt>, AppError> {
        let art = art.inner().clone();
        tauri::async_runtime::spawn_blocking(move || art.item_art(&names, kind)).await.map_err(AppError::internal)
    }
}
