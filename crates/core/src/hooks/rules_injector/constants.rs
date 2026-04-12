use std::path::PathBuf;
use std::sync::OnceLock;
use regex::Regex;

// Mocking shared data path function
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

pub fn rules_injector_storage() -> &'static PathBuf {
    static STORAGE: OnceLock<PathBuf> = OnceLock::new();
    STORAGE.get_or_init(|| opencode_storage().join("rules-injector"))
}

pub const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "pyproject.toml",
    "package.json",
    "Cargo.toml",
    "go.mod",
    ".venv",
];

pub const PROJECT_RULE_SUBDIRS: &[(&str, &str)] = &[
    (".github", "instructions"),
    (".cursor", "rules"),
    (".claude", "rules"),
];

pub const PROJECT_RULE_FILES: &[&str] = &[
    ".github/copilot-instructions.md",
];

pub fn github_instructions_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\.instructions\.md$").unwrap())
}

pub const USER_RULE_DIR: &str = ".claude/rules";

pub const RULE_EXTENSIONS: &[&str] = &[".md", ".mdc"];
