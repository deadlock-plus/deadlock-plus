use std::path::PathBuf;

pub fn httpcache_dir() -> Option<PathBuf> {
    dp_steam::httpcache_dir()
}
