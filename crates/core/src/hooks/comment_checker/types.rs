use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentInfo {
    pub text: String,
    pub line_number: usize,
    pub file_path: String,
    pub comment_type: String,
}
