use std::path::PathBuf;
use std::sync::OnceLock;

pub const PACKAGE_NAME: &str = "bl1nk-agents-manager";
pub const GITHUB_REPO_URL: &str = "https://api.github.com/repos/billlzzz18/bl1nk-agents-manager/releases/latest";

pub fn cache_dir() -> &'static PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        if cfg!(windows) {
            let local = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(local).join("bl1nk")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".cache").join("bl1nk")
        }
    })
}

pub fn version_file() -> &'static PathBuf {
    static FILE: OnceLock<PathBuf> = OnceLock::new();
    FILE.get_or_init(|| cache_dir().join("version"))
}
