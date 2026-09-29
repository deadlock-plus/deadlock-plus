pub mod commands {
    use dp_steam::SteamAccount;

    #[tauri::command]
    pub fn current_steam_account() -> Option<SteamAccount> {
        dp_steam::current_account()
    }

    #[tauri::command]
    pub fn local_steam_account_ids() -> Vec<u32> {
        dp_steam::local_account_ids()
    }
}
