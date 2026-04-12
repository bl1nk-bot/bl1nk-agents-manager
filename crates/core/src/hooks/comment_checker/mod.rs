pub mod types;

use std::process::Command;
use crate::hooks::comment_checker::types::CommentInfo;

pub struct CommentCheckerHook;

impl CommentCheckerHook {
    pub fn new() -> Self { Self }

    pub async fn run_checker(&self, file_path: &str) -> Option<String> {
        let output = Command::new("comment-checker")
            .arg(file_path)
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            None
        }
    }
}
