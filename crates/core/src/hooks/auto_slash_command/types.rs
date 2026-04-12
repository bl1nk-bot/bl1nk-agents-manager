use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedSlashCommand {
    pub command: String,
    pub args: String,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSlashCommandResult {
    pub detected: bool,
    pub parsed_command: Option<ParsedSlashCommand>,
    pub injected_message: Option<String>,
}
