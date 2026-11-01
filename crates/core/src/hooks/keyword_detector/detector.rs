use crate::hooks::keyword_detector::constants::{KEYWORD_PATTERNS, code_block_pattern, inline_code_pattern};
use regex::Regex;

pub struct DetectedKeyword {
    pub type_name: String,
    pub message: String,
}

pub fn remove_code_blocks(text: &str) -> String {
    let text = code_block_pattern().replace_all(text, "");
    inline_code_pattern().replace_all(&text, "").to_string()
}

pub fn detect_keywords(text: &str) -> Vec<DetectedKeyword> {
    let clean_text = remove_code_blocks(text);
    let mut results = Vec::new();

    for kp in KEYWORD_PATTERNS {
        let re = Regex::new(kp.pattern).unwrap();
        if re.is_match(&clean_text) {
            results.push(DetectedKeyword {
                type_name: kp.type_name.to_string(),
                message: kp.message.to_string(),
            });
        }
    }

    results
}
