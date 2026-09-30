use std::path::PathBuf;

pub fn httpcache_dir() -> Option<PathBuf> {
    let dir = steamlocate::SteamDir::locate().ok()?.path().join("appcache").join("httpcache");
    dir.is_dir().then_some(dir)
}
