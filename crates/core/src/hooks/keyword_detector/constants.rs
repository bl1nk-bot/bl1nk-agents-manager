use regex::Regex;
use std::sync::OnceLock;

pub fn code_block_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?s)```.*?```").unwrap())
}

pub fn inline_code_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"`[^`]+`").unwrap())
}

pub const ULTRAWORK_MODE_START: &str = "<ultrawork-mode>";
pub const ULTRAWORK_MODE_END: &str = "</ultrawork-mode>";

pub struct KeywordPattern {
    pub pattern: &'static str,
    pub type_name: &'static str,
    pub message: &'static str,
}

pub const KEYWORD_PATTERNS: &[KeywordPattern] = &[
    KeywordPattern {
        pattern: r"\b(ultrawork|ulw)\b",
        type_name: "ultrawork",
        message: "ULTRAWORK MODE ENABLED! Maximum precision required. Ultrathink before acting.",
    },
    KeywordPattern {
        pattern: r"\b(search|find|locate|lookup|explore|discover|scan|grep|query)\b",
        type_name: "search",
        message: "[search-mode]\nMAXIMIZE SEARCH EFFORT. Launch multiple background agents IN PARALLEL.",
    },
    KeywordPattern {
        pattern: r"\b(analyze|analyse|investigate|examine|research|study|inspect|audit)\b",
        type_name: "analyze",
        message: "[analyze-mode]\nANALYSIS MODE. Gather context before diving deep.",
    }
];

