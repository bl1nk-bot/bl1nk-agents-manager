//! Ask User Question Tool
//! 
//! Tool for asking users multiple-choice questions during AI execution.
//! Used to gather preferences, clarify requirements, or get decisions.

use serde::{Deserialize, Serialize};

/// A single option for a question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionItem {
    /// 1-5 words, concise choice text
    pub label: String,
    /// Explanation of trade-offs/implications  
    pub description: String,
}

/// A question to ask the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    /// The complete question to ask, ends with '?'
    pub question: String,
    /// Short label for tab/chip display (max 12 chars)
    pub header: String,
    /// Available options (2-4)
    pub options: Vec<OptionItem>,
    /// Allow multiple selections
    #[serde(default = "default_false")]
    pub multi_select: bool,
}

fn default_false() -> bool {
    false
}

/// Input for the askUserQuestion tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskUserQuestionInput {
    /// Questions to ask (1-4)
    pub questions: Vec<Question>,
}

/// An answer value (string or array of strings)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnswerValue {
    Single(String),
    Multiple(Vec<String>),
}

/// Output when user provides answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswersOutput {
    pub answers: serde_json::Map<String, serde_json::Value>,
}

/// Output when user declines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclinedOutput {
    pub declined: bool,
}

/// Output from the askUserQuestion tool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AskUserQuestionOutput {
    Answers(AnswersOutput),
    Declined(DeclinedOutput),
}

/// Transform tool output to model-friendly format
pub fn to_model_output(output: Option<&AskUserQuestionOutput>) -> String {
    match output {
        None => "User did not respond to questions.".to_string(),
        Some(AskUserQuestionOutput::Declined(_)) => {
            "User declined to answer questions. You should continue without this information or ask in a different way.".to_string()
        }
        Some(AskUserQuestionOutput::Answers(answers)) => {
            let formatted = answers.answers
                .iter()
                .map(|(q, a)| {
                    let answer_str = match a {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Array(arr) => arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<Vec<_>>()
                            .join(", "),
                        _ => a.to_string(),
                    };
                    format!("\"{}\"=\"{}\"", q, answer_str)
                })
                .collect::<Vec<_>>()
                .join(", ");
            
            if formatted.is_empty() {
                "User responded to questions.".to_string()
            } else {
                format!("User has answered your questions: {}. You can now continue with the user's answers in mind.", formatted)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_serialization() {
        let q = Question {
            question: "What language should I use?".to_string(),
            header: "Language".to_string(),
            options: vec![
                OptionItem {
                    label: "Rust".to_string(),
                    description: "Fast, safe, but steeper learning curve".to_string(),
                },
                OptionItem {
                    label: "Python".to_string(),
                    description: "Easy to learn, good ecosystem".to_string(),
                },
            ],
            multi_select: false,
        };
        
        let json = serde_json::to_string(&q).unwrap();
        assert!(json.contains("Rust"));
        assert!(json.contains("Language"));
    }

    #[test]
    fn test_ask_user_question_input() {
        let input = AskUserQuestionInput {
            questions: vec![Question {
                question: "What should I name this?".to_string(),
                header: "Name".to_string(),
                options: vec![],
                multi_select: false,
            }],
        };
        
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("What should I name this?"));
    }

    #[test]
    fn test_to_model_output_none() {
        let result = to_model_output(None);
        assert_eq!(result, "User did not respond to questions.");
    }

    #[test]
    fn test_to_model_output_declined() {
        let output = AskUserQuestionOutput::Declined(DeclinedOutput { declined: true });
        let result = to_model_output(Some(&output));
        assert!(result.contains("declined"));
    }

    #[test]
    fn test_to_model_output_answers() {
        let mut answers = serde_json::Map::new();
        answers.insert("language".to_string(), serde_json::Value::String("Rust".to_string()));
        
        let output = AskUserQuestionOutput::Answers(AnswersOutput { answers });
        let result = to_model_output(Some(&output));
        
        assert!(result.contains("language"));
        assert!(result.contains("Rust"));
    }
}