use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

use crate::hooks::anthropic_context_window_limit_recovery::types::ParsedTokenLimitError;

// รูปแบบ regex สำหรับจับคู่ข้อผิดพลาดเกี่ยวกับ token limit
const TOKEN_LIMIT_PATTERNS: &[&str] = &[
    r"(\d+)\s*tokens?\s*>\s*(\d+)\s*maximum",
    r"prompt.*?(\d+).*?tokens.*?exceeds.*?(\d+)",
    r"(\d+).*?tokens.*?limit.*?(\d+)",
    r"context.*?length.*?(\d+).*?maximum.*?(\d+)",
    r"max.*?context.*?(\d+).*?but.*?(\d+)",
];

// คำหลักที่บ่งชี้ถึงข้อผิดพลาดเกี่ยวกับ token limit
const TOKEN_LIMIT_KEYWORDS: &[&str] = &[
    "prompt is too long",
    "is too long",
    "context_length_exceeded",
    "max_tokens",
    "token limit",
    "context length",
    "too many tokens",
    "non-empty content",
];

// รูปแบบ regex สำหรับข้อผิดพลาดเกี่ยวกับ thinking block (ไม่ใช่ token limit)
const THINKING_BLOCK_ERROR_PATTERNS: &[&str] = &[
    r"thinking.*first block",
    r"first block.*thinking",
    r"must.*start.*thinking",
    r"thinking.*redacted_thinking",
    r"expected.*thinking.*found",
    r"thinking.*disabled.*cannot.*contain",
];

const MESSAGE_INDEX_PATTERN: &str = r"messages\.(\d+)";

fn is_thinking_block_error(text: &str) -> bool {
    let lower_text = text.to_lowercase();
    for &pattern in THINKING_BLOCK_ERROR_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&lower_text) {
                return true;
            }
        }
    }
    false
}

fn extract_tokens_from_message(message: &str) -> Option<(u32, u32)> {
    let lower_message = message.to_lowercase();
    
    for &pattern_str in TOKEN_LIMIT_PATTERNS {
        if let Ok(re) = Regex::new(pattern_str) {
            if let Some(caps) = re.captures(&lower_message) {
                if caps.len() >= 3 {
                    if let (Ok(num1_str), Ok(num2_str)) = (caps.get(1), caps.get(2)) {
                        if let (Ok(num1), Ok(num2)) = (num1_str.as_str().parse::<u32>(), num2_str.as_str().parse::<u32>()) {
                            return if num1 > num2 {
                                Some((num1, num2))
                            } else {
                                Some((num2, num1))
                            };
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_message_index(text: &str) -> Option<u32> {
    if let Ok(re) = Regex::new(MESSAGE_INDEX_PATTERN) {
        if let Some(caps) = re.captures(text) {
            if let Some(matched) = caps.get(1) {
                return matched.as_str().parse::<u32>().ok();
            }
        }
    }
    None
}

fn is_token_limit_error(text: &str) -> bool {
    if is_thinking_block_error(text) {
        return false;
    }
    
    let lower = text.to_lowercase();
    for &kw in TOKEN_LIMIT_KEYWORDS {
        if lower.contains(&kw.to_lowercase()) {
            return true;
        }
    }
    false
}

pub fn parse_anthropic_token_limit_error(err: Option<&Value>) -> Option<ParsedTokenLimitError> {
    if let Some(error_value) = err {
        // ถ้า error เป็น string
        if let Some(error_str) = error_value.as_str() {
            let lower_error = error_str.to_lowercase();
            
            if lower_error.contains("non-empty content") {
                return Some(ParsedTokenLimitError {
                    current_tokens: 0,
                    max_tokens: 0,
                    request_id: None,
                    error_type: "non-empty content".to_string(),
                    provider_id: None,
                    model_id: None,
                    message_index: extract_message_index(error_str),
                });
            }
            
            if is_token_limit_error(error_str) {
                if let Some((current, max)) = extract_tokens_from_message(error_str) {
                    return Some(ParsedTokenLimitError {
                        current_tokens: current,
                        max_tokens: max,
                        request_id: None,
                        error_type: "token_limit_exceeded_string".to_string(),
                        provider_id: None,
                        model_id: None,
                        message_index: None,
                    });
                }
            }
            
            return None;
        }

        // ถ้า error เป็น object
        if let Some(error_obj) = error_value.as_object() {
            let mut text_sources = Vec::new();

            // ดึงข้อมูล string จากแหล่งต่างๆ
            if let Some(response_body_val) = error_obj.get("responseBody") {
                if let Some(response_body_str) = response_body_val.as_str() {
                    text_sources.push(response_body_str.to_string());
                }
            }

            if let Some(message_val) = error_obj.get("message") {
                if let Some(message_str) = message_val.as_str() {
                    text_sources.push(message_str.to_string());
                }
            }

            if let Some(error_field_val) = error_obj.get("error") {
                if let Some(error_field_obj) = error_field_val.as_object() {
                    if let Some(error_message_val) = error_field_obj.get("message") {
                        if let Some(error_message_str) = error_message_val.as_str() {
                            text_sources.push(error_message_str.to_string());
                        }
                    }
                }
            }

            if let Some(body_val) = error_obj.get("body") {
                if let Some(body_str) = body_val.as_str() {
                    text_sources.push(body_str.to_string());
                }
            }

            if let Some(details_val) = error_obj.get("details") {
                if let Some(details_str) = details_val.as_str() {
                    text_sources.push(details_str.to_string());
                }
            }

            if let Some(reason_val) = error_obj.get("reason") {
                if let Some(reason_str) = reason_val.as_str() {
                    text_sources.push(reason_str.to_string());
                }
            }

            if let Some(description_val) = error_obj.get("description") {
                if let Some(description_str) = description_val.as_str() {
                    text_sources.push(description_str.to_string());
                }
            }

            // ถ้าไม่พบข้อความใดๆ ให้ลองแปลง object เป็น JSON string
            if text_sources.is_empty() {
                if let Ok(json_str) = serde_json::to_string(error_obj) {
                    if is_token_limit_error(&json_str) {
                        text_sources.push(json_str);
                    }
                }
            }

            let combined_text = text_sources.join(" ");
            if !is_token_limit_error(&combined_text) {
                return None;
            }

            // ประมวลผล response body ถ้าเป็น string
            if let Some(response_body_val) = error_obj.get("responseBody") {
                if let Some(response_body_str) = response_body_val.as_str() {
                    // ลอง parse เป็น JSON และดึงข้อมูล
                    if let Ok(json_value) = serde_json::from_str::<Value>(response_body_str) {
                        if let Some(json_obj) = json_value.as_object() {
                            if let Some(error_field) = json_obj.get("error").and_then(|e| e.as_object()) {
                                if let Some(message_val) = error_field.get("message") {
                                    if let Some(message_str) = message_val.as_str() {
                                        if let Some((current, max)) = extract_tokens_from_message(message_str) {
                                            return Some(ParsedTokenLimitError {
                                                current_tokens: current,
                                                max_tokens: max,
                                                request_id: json_obj.get("request_id").and_then(|id| id.as_str()).map(|s| s.to_string()),
                                                error_type: error_field.get("type").and_then(|t| t.as_str()).unwrap_or("token_limit_exceeded").to_string(),
                                                provider_id: None,
                                                model_id: None,
                                                message_index: None,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ค้นหา token จาก text sources
            for text in &text_sources {
                if let Some((current, max)) = extract_tokens_from_message(text) {
                    return Some(ParsedTokenLimitError {
                        current_tokens: current,
                        max_tokens: max,
                        request_id: None,
                        error_type: "token_limit_exceeded".to_string(),
                        provider_id: None,
                        model_id: None,
                        message_index: None,
                    });
                }
            }

            // ตรวจสอบว่าเป็น non-empty content หรือไม่
            if combined_text.to_lowercase().contains("non-empty content") {
                return Some(ParsedTokenLimitError {
                    current_tokens: 0,
                    max_tokens: 0,
                    request_id: None,
                    error_type: "non-empty content".to_string(),
                    provider_id: None,
                    model_id: None,
                    message_index: extract_message_index(&combined_text),
                });
            }

            if is_token_limit_error(&combined_text) {
                return Some(ParsedTokenLimitError {
                    current_tokens: 0,
                    max_tokens: 0,
                    request_id: None,
                    error_type: "token_limit_exceeded_unknown".to_string(),
                    provider_id: None,
                    model_id: None,
                    message_index: None,
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_token_limit_error() {
        assert!(is_token_limit_error("prompt is too long"));
        assert!(is_token_limit_error("context length exceeded"));
        assert!(is_token_limit_error("too many tokens"));
        assert!(!is_token_limit_error("connection timeout"));
    }

    #[test]
    fn test_extract_tokens_from_message() {
        let result = extract_tokens_from_message("1500 tokens > 1000 maximum");
        assert_eq!(result, Some((1500, 1000)));
        
        let result = extract_tokens_from_message("context length 2000 exceeds maximum 1800");
        assert_eq!(result, Some((2000, 1800)));
    }

    #[test]
    fn test_extract_message_index() {
        let result = extract_message_index("Error in messages.5 field");
        assert_eq!(result, Some(5));
        
        let result = extract_message_index("No index here");
        assert_eq!(result, None);
    }

    #[test]
    fn test_is_thinking_block_error() {
        assert!(is_thinking_block_error("thinking must come first block"));
        assert!(is_thinking_block_error("first block must be thinking"));
        assert!(!is_thinking_block_error("token limit exceeded"));
    }
}