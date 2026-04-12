use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use regex::Regex;

pub fn normalize_model_id(model_id: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\.(\d+)").unwrap());
    re.replace_all(model_id, "-$1").to_string()
}

pub fn high_variant_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("claude-sonnet-4-5", "claude-sonnet-4-5-high");
        m.insert("gemini-3-pro", "gemini-3-pro-high");
        m.insert("gpt-5", "gpt-5-high");
        m
    })
}

pub fn get_high_variant(model_id: &str) -> Option<String> {
    let normalized = normalize_model_id(model_id);
    let base = if let Some(idx) = normalized.find('/') {
        &normalized[idx+1..]
    } else {
        &normalized
    };

    if base.ends_with("-high") { return None; }

    high_variant_map().get(base).map(|v| {
        if let Some(idx) = normalized.find('/') {
            format!("{}{}", &normalized[..idx+1], v)
        } else {
            v.to_string()
        }
    })
}
