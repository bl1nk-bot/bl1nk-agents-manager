use std::collections::HashSet;
use std::sync::OnceLock;
use regex::Regex;

pub const HOOK_NAME: &str = "auto-slash-command";
pub const AUTO_SLASH_COMMAND_TAG_OPEN: &str = "<auto-slash-command>";
pub const AUTO_SLASH_COMMAND_TAG_CLOSE: &str = "</auto-slash-command>";

pub fn slash_command_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^/([a-zA-Z][\w-]*)\s*(.*)").unwrap())
}

pub fn excluded_commands() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert("ralph-loop");
        s.insert("cancel-ralph");
        s.insert("ulw-loop");
        s
    })
}
