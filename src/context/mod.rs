//! Context Management Module
//!
//! Provides data structures for managing conversation context, workspaces, and secrets.
//!
//! ## Architecture
//!
//! - `Conversation`: A single conversation/session with messages
//! - `Workspace`: A workspace containing multiple conversations  
//! - `Secrets`: Secure key-value storage for sensitive data
//! - `ContextStore`: Trait for abstracting storage mechanisms

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Workspace metadata file stored at .omg/state/workspaces.json
pub const WORKSPACES_INDEX_FILE: &str = "workspaces.json";

/// Workspace data stored at .omg/state/workspaces/{id}.json
pub fn workspace_file_path(id: Uuid) -> String {
    format!("workspaces/{}.json", id)
}

/// Secrets file stored at .omg/state/secrets/{id}.json
pub fn secrets_file_path(id: Uuid) -> String {
    format!("secrets/{}.json", id)
}

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Role of a message sender
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}

/// A conversation represents a single session with multiple messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Conversation {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub messages: Vec<Message>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Conversation {
    /// Create a new conversation with a given workspace ID
    pub fn new(workspace_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            workspace_id,
            messages: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(Message {
            role,
            content,
            timestamp: Utc::now(),
        });
    }
}

/// A workspace contains multiple conversations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub conversations: HashMap<Uuid, Conversation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workspace {
    /// Create a new workspace
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            conversations: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a conversation to the workspace
    pub fn add_conversation(&mut self, conversation: Conversation) {
        self.conversations.insert(conversation.id, conversation);
        self.updated_at = Utc::now();
    }
}

/// Secrets storage for sensitive data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Secrets {
    pub entries: HashMap<String, String>,
    pub encrypted: bool,
}

impl Secrets {
    /// Create a new empty secrets store
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            encrypted: false,
        }
    }

    /// Set a secret value
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.insert(key.into(), value.into());
    }

    /// Get a secret value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.entries.get(key)
    }

    /// Remove a secret
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.entries.remove(key)
    }
}

impl Default for Secrets {
    fn default() -> Self {
        Self::new()
    }
}

/// Context store trait for abstracting storage mechanisms
#[async_trait::async_trait]
pub trait ContextStore: Send + Sync {
    /// Save a workspace
    async fn save_workspace(&self, workspace: &Workspace) -> anyhow::Result<()>;

    /// Load a workspace by ID
    async fn load_workspace(&self, id: Uuid) -> anyhow::Result<Option<Workspace>>;

    /// List all workspaces
    async fn list_workspaces(&self) -> anyhow::Result<Vec<Workspace>>;

    /// Delete a workspace
    async fn delete_workspace(&self, id: Uuid) -> anyhow::Result<()>;

    /// Save secrets for a workspace
    async fn save_secrets(&self, workspace_id: Uuid, secrets: &Secrets) -> anyhow::Result<()>;

    /// Load secrets for a workspace
    async fn load_secrets(&self, workspace_id: Uuid) -> anyhow::Result<Option<Secrets>>;
}

pub mod json_store;
pub mod tool_compaction;
pub use json_store::JsonContextStore;

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Conversation Tests ==========

    #[test]
    fn test_conversation_new() {
        let workspace_id = Uuid::new_v4();
        let conversation = Conversation::new(workspace_id);

        assert_eq!(conversation.workspace_id, workspace_id);
        assert!(conversation.messages.is_empty());
        assert!(conversation.metadata.is_empty());
    }

    #[test]
    fn test_conversation_add_message() {
        let mut conversation = Conversation::new(Uuid::new_v4());

        conversation.add_message(MessageRole::User, "Hello".to_string());

        assert_eq!(conversation.messages.len(), 1);
        assert_eq!(conversation.messages[0].role, MessageRole::User);
        assert_eq!(conversation.messages[0].content, "Hello");
    }

    #[test]
    fn test_conversation_multiple_messages() {
        let mut conversation = Conversation::new(Uuid::new_v4());

        conversation.add_message(MessageRole::User, "Hello".to_string());
        conversation.add_message(MessageRole::Assistant, "Hi there!".to_string());
        conversation.add_message(MessageRole::User, "How are you?".to_string());

        assert_eq!(conversation.messages.len(), 3);
        assert_eq!(conversation.messages[0].role, MessageRole::User);
        assert_eq!(conversation.messages[1].role, MessageRole::Assistant);
        assert_eq!(conversation.messages[2].role, MessageRole::User);
    }

    #[test]
    fn test_conversation_serialization() {
        let mut conversation = Conversation::new(Uuid::new_v4());
        conversation.add_message(MessageRole::User, "Test message".to_string());

        let json = serde_json::to_string(&conversation).unwrap();
        let deserialized: Conversation = serde_json::from_str(&json).unwrap();

        assert_eq!(conversation, deserialized);
    }

    // ========== Workspace Tests ==========

    #[test]
    fn test_workspace_new() {
        let workspace = Workspace::new("Test Workspace".to_string());

        assert_eq!(workspace.name, "Test Workspace");
        assert!(workspace.conversations.is_empty());
        assert_eq!(workspace.created_at, workspace.updated_at);
    }

    #[test]
    fn test_workspace_add_conversation() {
        let mut workspace = Workspace::new("Test".to_string());
        let conversation = Conversation::new(workspace.id);
        let conversation_id = conversation.id;

        workspace.add_conversation(conversation);

        assert_eq!(workspace.conversations.len(), 1);
        assert!(workspace.conversations.contains_key(&conversation_id));
        assert!(workspace.updated_at >= workspace.created_at);
    }

    #[test]
    fn test_workspace_serialization() {
        let mut workspace = Workspace::new("Test".to_string());
        workspace.add_conversation(Conversation::new(workspace.id));

        let json = serde_json::to_string(&workspace).unwrap();
        let deserialized: Workspace = serde_json::from_str(&json).unwrap();

        assert_eq!(workspace, deserialized);
    }

    // ========== Secrets Tests ==========

    #[test]
    fn test_secrets_new() {
        let secrets = Secrets::new();

        assert!(secrets.entries.is_empty());
        assert!(!secrets.encrypted);
    }

    #[test]
    fn test_secrets_set_and_get() {
        let mut secrets = Secrets::new();

        secrets.set("api_key", "secret123");

        assert_eq!(secrets.get("api_key"), Some(&"secret123".to_string()));
        assert_eq!(secrets.get("nonexistent"), None);
    }

    #[test]
    fn test_secrets_remove() {
        let mut secrets = Secrets::new();

        secrets.set("key1", "value1");
        assert_eq!(secrets.get("key1"), Some(&"value1".to_string()));

        let removed = secrets.remove("key1");
        assert_eq!(removed, Some("value1".to_string()));
        assert_eq!(secrets.get("key1"), None);
    }

    #[test]
    fn test_secrets_default() {
        let secrets = Secrets::default();

        assert!(secrets.entries.is_empty());
        assert!(!secrets.encrypted);
    }

    #[test]
    fn test_secrets_serialization() {
        let mut secrets = Secrets::new();
        secrets.set("token", "abc123");

        let json = serde_json::to_string(&secrets).unwrap();
        let deserialized: Secrets = serde_json::from_str(&json).unwrap();

        assert_eq!(secrets, deserialized);
    }

    // ========== MessageRole Tests ==========

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::User;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"user\"");

        let deserialized: MessageRole = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MessageRole::User);
    }

    #[test]
    fn test_message_role_all_variants() {
        // Test that all variants can be serialized and deserialized
        for role in [MessageRole::User, MessageRole::Assistant, MessageRole::System] {
            let json = serde_json::to_string(&role).unwrap();
            let deserialized: MessageRole = serde_json::from_str(&json).unwrap();
            assert_eq!(role, deserialized);
        }
    }
}
