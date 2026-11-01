use crate::hooks::auto_slash_command::constants::{slash_command_pattern, excluded_commands};
use crate::hooks::auto_slash_command::types::ParsedSlashCommand;
use regex::Regex;
use std::sync::OnceLock;

pub fn remove_code_blocks(text: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(?s)```.*?```").unwrap());
    re.replace_all(text, "").to_string()
}

pub fn parse_slash_command(text: &str) -> Option<ParsedSlashCommand> {
    let trimmed = text.trim();
    if !trimmed.starts_with('/') { return None; }

    let caps = slash_command_pattern().captures(trimmed)?;
    
    Some(ParsedSlashCommand {
        raw: caps.get(0).unwrap().as_str().to_string(),
        command: caps.get(1).unwrap().as_str().to_lowercase(),
        args: caps.get(2).map_or("", |m| m.as_str()).trim().to_string(),
    })
}

pub fn detect_slash_command(text: &str) -> Option<ParsedSlashCommand> {
    let clean_text = remove_code_blocks(text);
    let parsed = parse_slash_command(&clean_text)?;

    if excluded_commands().contains(parsed.command.as_str()) {
        return None;
    }

    Some(parsed)
}
