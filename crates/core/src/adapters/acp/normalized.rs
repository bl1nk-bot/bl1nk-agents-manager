use crate::adapters::acp::{
    ContentBlock, Location, SessionUpdate, ToolCallContentItem, ToolCallKind, ToolCallStatus,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedUpdateKind {
    AssistantMessageChunk,
    AssistantThoughtChunk,
    ToolCall,
    ToolCallUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NormalizedToolCall {
    pub tool_call_id: String,
    pub status: ToolCallStatus,
    pub title: Option<String>,
    pub content: Vec<ToolCallContentItem>,
    pub locations: Vec<Location>,
    pub kind: Option<ToolCallKind>,
    pub server_name: Option<String>,
    pub tool_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NormalizedUpdate {
    pub kind: NormalizedUpdateKind,
    pub content: Option<ContentBlock>,
    pub tool_call: Option<NormalizedToolCall>,
}

impl From<&SessionUpdate> for NormalizedUpdate {
    fn from(update: &SessionUpdate) -> Self {
        match update {
            SessionUpdate::AgentMessageChunk { content } => Self {
                kind: NormalizedUpdateKind::AssistantMessageChunk,
                content: Some(content.clone()),
                tool_call: None,
            },
            SessionUpdate::AgentThoughtChunk { content } => Self {
                kind: NormalizedUpdateKind::AssistantThoughtChunk,
                content: Some(content.clone()),
                tool_call: None,
            },
            SessionUpdate::ToolCall {
                tool_call_id,
                status,
                title,
                content,
                locations,
                kind,
                server_name,
                tool_name,
            } => Self {
                kind: NormalizedUpdateKind::ToolCall,
                content: None,
                tool_call: Some(NormalizedToolCall {
                    tool_call_id: tool_call_id.clone(),
                    status: status.clone(),
                    title: Some(title.clone()),
                    content: content.clone(),
                    locations: locations.clone(),
                    kind: Some(kind.clone()),
                    server_name: server_name.clone(),
                    tool_name: tool_name.clone(),
                }),
            },
            SessionUpdate::ToolCallUpdate {
                tool_call_id,
                status,
                content,
                server_name,
                tool_name,
            } => Self {
                kind: NormalizedUpdateKind::ToolCallUpdate,
                content: None,
                tool_call: Some(NormalizedToolCall {
                    tool_call_id: tool_call_id.clone(),
                    status: status.clone(),
                    title: None,
                    content: content.clone(),
                    locations: Vec::new(),
                    kind: None,
                    server_name: server_name.clone(),
                    tool_name: tool_name.clone(),
                }),
            },
        }
    }
}

