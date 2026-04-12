pub mod constants;
pub mod detector;

use crate::hooks::keyword_detector::detector::detect_keywords;

pub struct KeywordDetectorHook;

impl KeywordDetectorHook {
    pub fn new() -> Self { Self }

    pub async fn on_chat_message(&self, prompt_text: &str) -> Option<String> {
        let detected = detect_keywords(prompt_text);
        if detected.is_empty() { return None; }

        let mut all_messages = Vec::new();
        for d in detected {
            all_messages.push(d.message);
        }

        Some(all_messages.join("\n\n"))
    }
}

