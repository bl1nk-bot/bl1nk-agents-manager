use regex::Regex;
use std::sync::OnceLock;

pub fn think_patterns() -> &'static Vec<Regex> {
    static RE: OnceLock<Vec<Regex>> = OnceLock::new();
    RE.get_or_init(|| {
        let keywords = vec![
            "think", "ultrathink", "생각", "고민", "思考", "考え", "คิด", "พิจารณา",
            "pensar", "pense", "penser", "denken", "думать"
        ];
        keywords.into_iter().map(|kw| Regex::new(&format!(r"(?i)\b{}\b", kw)).unwrap()).collect()
    })
}

pub fn remove_code_blocks(text: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(?s)```.*?```|`[^`]+`").unwrap());
    re.replace_all(text, "").to_string()
}

pub fn detect_think_keyword(text: &str) -> bool {
    let clean_text = remove_code_blocks(text);
    think_patterns().iter().any(|re| re.is_match(&clean_text))
}
