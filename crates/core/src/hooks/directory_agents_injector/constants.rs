use std::path::PathBuf;
use std::sync::OnceLock;

fn get_opencode_storage_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        PathBuf::from(home).join(".bl1nk")
    } else {
        PathBuf::from("/tmp/.bl1nk")
    }
}

pub fn opencode_storage() -> &'static PathBuf {
    static STORAGE: OnceLock<PathBuf> = OnceLock::new();
    STORAGE.get_or_init(get_opencode_storage_dir)
}

pub fn agents_injector_storage() -> &'static PathBuf {
    static STORAGE: OnceLock<PathBuf> = OnceLock::new();
    STORAGE.get_or_init(|| opencode_storage().join("directory-agents"))
}

pub const AGENTS_FILENAME: &str = "AGENTS.md";
