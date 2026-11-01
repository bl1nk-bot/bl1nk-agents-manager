pub mod constants;
pub mod types;
pub mod detector;
pub mod executor;

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::hooks::auto_slash_command::detector::detect_slash_command;
use crate::hooks::auto_slash_command::executor::execute_slash_command;
use crate::hooks::auto_slash_command::constants::{AUTO_SLASH_COMMAND_TAG_OPEN, AUTO_SLASH_COMMAND_TAG_CLOSE};
use std::path::PathBuf;

pub struct AutoSlashCommandHook {
    processed_commands: Arc<RwLock<HashSet<String>>>,
    base_directory: PathBuf,
}

impl AutoSlashCommandHook {
    pub fn new(base_directory: PathBuf) -> Self {
        Self {
            processed_commands: Arc::new(RwLock::new(HashSet::new())),
            base_directory,
        }
    }

    pub async fn on_chat_message(
        &self,
        session_id: &str,
        message_id: &str,
        prompt_text: &str,
    ) -> Option<String> {
        if prompt_text.contains(AUTO_SLASH_COMMAND_TAG_OPEN) {
            return None;
        }

        let parsed = detect_slash_command(prompt_text)?;
        
        let key = format!("{}:{}:{}", session_id, message_id, parsed.command);
        if self.processed_commands.read().await.contains(&key) {
            return None;
        }
        self.processed_commands.write().await.insert(key);

        if let Ok(replacement) = execute_slash_command(&parsed, &self.base_directory).await {
            let tagged = format!(
                "{}\n{}\n{}",
                AUTO_SLASH_COMMAND_TAG_OPEN, replacement, AUTO_SLASH_COMMAND_TAG_CLOSE
            );
            return Some(tagged);
        }

        None
    }
}
