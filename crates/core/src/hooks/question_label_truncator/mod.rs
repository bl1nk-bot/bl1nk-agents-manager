use regex::Regex;
use std::sync::OnceLock;

pub struct QuestionLabelTruncatorHook;

impl QuestionLabelTruncatorHook {
    pub fn new() -> Self {
        Self
    }

    pub fn truncate_label(&self, label: &str, max_length: usize) -> String {
        if label.len() <= max_length {
            label.to_string()
        } else {
            format!("{}...", &label[..max_length - 3])
        }
    }

    pub fn truncate_labels_in_text(&self, text: &str, max_length: usize) -> String {
        // ใช้ regex เพื่อหาและตัดป้ายกำกับคำถาม
        static LABEL_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = LABEL_REGEX.get_or_init(|| {
            Regex::new(r#"label\s*:\s*["']([^"']*)["']"#).unwrap()
        });

        regex.replace_all(text, |caps: &regex::Captures| {
            let original_label = &caps[1];
            let truncated_label = self.truncate_label(original_label, max_length);
            format!("label: \"{}\"", truncated_label)
        }).to_string()
    }

    pub fn on_tool_execute_before(&self, tool_name: &str, args: &mut serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        // ตรวจสอบว่าเป็นเครื่องมือที่ต้องการหรือไม่
        let lower_tool_name = tool_name.to_lowercase();
        if lower_tool_name != "askuserquestion" && lower_tool_name != "ask_user_question" {
            return Ok(());
        }

        // ตัดป้ายกำกับใน args ถ้ามี
        if let Some(questions) = args.get_mut("questions").and_then(|v| v.as_array_mut()) {
            for question in questions.iter_mut() {
                if let Some(options) = question.get_mut("options").and_then(|v| v.as_array_mut()) {
                    for option in options.iter_mut() {
                        if let Some(label_value) = option.get_mut("label") {
                            if let Some(label_str) = label_value.as_str() {
                                let truncated_label = self.truncate_label(label_str, 30); // ความยาวสูงสุด 30 ตัวอักษร
                                *label_value = serde_json::Value::String(truncated_label);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}