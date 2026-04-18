# Context Management Implementation Plan (Phases 3-5)

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Complete the Context Management system with compaction, secrets handling, and orchestrator integration.

**Architecture:** 
- Phase 3: Sliding window compaction for token management + file-based offloading
- Phase 4: Secure environment variable and secrets injection
- Phase 5: Integration with agent executor for context injection

**Tech Stack:** Rust (2021), tokio async, serde_json, existing bl1nk-agents-manager codebase

---

## Phase 3: Context Compaction & Size Management

### Task 1: Add message count and token estimate fields to Conversation

**Objective:** Track message count and estimate tokens for compaction decisions

**Files:**
- Modify: `src/context/mod.rs:51-64` (Conversation struct and impl)

**Step 1: Add fields to Conversation struct**

```rust
// Add after line 54 (messages field)
pub token_count: u32,
pub message_count: u32,
```

**Step 2: Update Conversation::new()**

```rust
pub fn new(workspace_id: Uuid) -> Self {
    Self {
        id: Uuid::new_v4(),
        workspace_id,
        messages: Vec::new(),
        metadata: HashMap::new(),
        token_count: 0,
        message_count: 0,
    }
}
```

**Step 3: Update add_message() to track counts**

```rust
pub fn add_message(&mut self, role: MessageRole, content: String) {
    let token_estimate = estimate_tokens(&content);
    self.token_count += token_estimate;
    self.message_count += 1;
    self.messages.push(Message {
        role,
        content,
        timestamp: Utc::now(),
    });
}
```

**Step 4: Add token estimation helper**

```rust
/// Estimate token count (rough approximation: ~4 chars per token)
fn estimate_tokens(text: &str) -> u32 {
    (text.len() / 4) as u32
}
```

**Step 5: Run tests**

Run: `cargo test --lib context::tests -v`
Expected: COMPILATION ERROR - need to fix the new struct

**Step 6: Fix compilation**

Add default values or update tests to use new fields.

**Step 7: Run tests to verify pass**

Run: `cargo test --lib context -v`
Expected: 21 passed

**Step 8: Commit**

```bash
git add src/context/mod.rs
git commit -m "feat(context): add token and message count to Conversation"
```

---

### Task 2: Implement compaction logic (retain top N%)

**Objective:** Add method to compact conversation to a percentage of messages

**Files:**
- Modify: `src/context/mod.rs:58-76` (add after add_message method)

**Step 1: Add compact() method to Conversation**

```rust
/// Compact conversation by retaining top percentage of messages (by timestamp)
/// Retains recent messages, discards oldest
pub fn compact(&mut self, retain_percent: f32) {
    if retain_percent >= 1.0 || self.messages.is_empty() {
        return;
    }
    
    let retain_count = ((self.messages.len() as f32) * retain_percent).ceil() as usize;
    let remove_count = self.messages.len() - retain_count;
    
    // Remove oldest messages
    self.messages.drain(0..remove_count);
    
    // Recalculate token count
    self.token_count = self.messages.iter()
        .map(|m| estimate_tokens(&m.content))
        .sum();
    self.message_count = self.messages.len() as u32;
}
```

**Step 2: Add tests for compaction**

Add to `src/context/mod.rs` tests section:

```rust
#[test]
fn test_conversation_compact_retains_recent() {
    let mut conversation = Conversation::new(Uuid::new_v4());
    for i in 0..10 {
        conversation.add_message(MessageRole::User, format!("Message {}", i));
    }
    
    conversation.compact(0.5); // Keep 50%
    
    assert_eq!(conversation.messages.len(), 5);
    // Should retain messages 5-9 (most recent)
    assert!(conversation.messages[0].content.contains("Message 5"));
}

#[test]
fn test_conversation_compact_no_op_for_full() {
    let mut conversation = Conversation::new(Uuid::new_v4());
    conversation.add_message(MessageRole::User, "Test".to_string());
    conversation.compact(1.0);
    assert_eq!(conversation.messages.len(), 1);
}

#[test]
fn test_conversation_compact_empty() {
    let mut conversation = Conversation::new(Uuid::new_v4());
    conversation.compact(0.5);
    assert!(conversation.messages.is_empty());
}
```

**Step 3: Run tests**

Run: `cargo test --lib context::tests::test_conversation_compact -v`
Expected: FAIL - compact method doesn't exist yet

**Step 4: Write compact method (from Step 1)**

**Step 5: Run tests**

Run: `cargo test --lib context::tests::test_conversation_compact -v`
Expected: PASS

**Step 6: Commit**

```rust
git add src/context/mod.rs
git commit -m "feat(context): add conversation compaction method"
```

---

### Task 3: Implement context offloading to file

**Objective:** Save overflow context to readable file instead of losing it

**Files:**
- Modify: `src/context/mod.rs` (add offload method)
- Modify: `src/context/json_store.rs` (add offload storage)

**Step 1: Add offload_to_file() method to Workspace**

```rust
/// Offload older conversations to a readable archive file
pub fn offload_to_file(&self, path: &Path) -> Result<()> {
    let mut content = String::new();
    content.push_str(&format!("# Context Archive: {}\n", self.name));
    content.push_str(&format!("Created: {}\n\n", Utc::now()));
    
    for (id, conv) in &self.conversations {
        content.push_str(&format!("## Conversation: {}\n", id));
        for msg in &conv.messages {
            content.push_str(&format!(
                "[{}] {}: {}\n",
                msg.timestamp.format("%Y-%m-%d %H:%M"),
                format!("{:?}", msg.role),
                msg.content
            ));
        }
        content.push_str("\n");
    }
    
    fs::write(path, content)?;
    Ok(())
}
```

**Step 2: Add offload method to JsonContextStore**

```rust
/// Offload workspace context to a readable archive file
pub async fn offload_workspace(&self, workspace_id: Uuid) -> Result<PathBuf> {
    let workspace = self.load_workspace(workspace_id)
        .await?
        .context("Workspace not found")?;
    
    let archive_dir = self.base_path.join(".omg").join("state").join("archives");
    fs::create_dir_all(&archive_dir).await?;
    
    let filename = format!("{}_{}.md", workspace.name, workspace_id);
    let path = archive_dir.join(&filename);
    
    let mut content = String::new();
    content.push_str(&format!("# Context Archive: {}\n", workspace.name));
    content.push_str(&format!("Created: {}\n\n", Utc::now()));
    
    for (id, conv) in &workspace.conversations {
        content.push_str(&format!("## Conversation: {}\n", id));
        for msg in &conv.messages {
            content.push_str(&format!(
                "[{}] {}: {}\n",
                msg.timestamp.format("%Y-%m-%d %H:%M"),
                format!("{:?}", msg.role),
                msg.content
            ));
        }
        content.push_str("\n");
    }
    
    fs::write(&path, content).await?;
    Ok(path)
}
```

**Step 3: Add tests**

```rust
#[tokio::test]
async fn test_offload_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let store = JsonContextStore::new(temp_dir.path().to_path_buf());
    
    let workspace = Workspace::new("Test".to_string());
    store.save_workspace(&workspace).await.unwrap();
    
    let path = store.offload_workspace(workspace.id).await.unwrap();
    assert!(path.exists());
    
    let content = fs::read_to_string(&path).await.unwrap();
    assert!(content.contains("Context Archive"));
}
```

**Step 4: Run tests**

Run: `cargo test --lib context -v`
Expected: PASS (all tests including new ones)

**Step 5: Commit**

```bash
git add src/context/mod.rs src/context/json_store.rs
git commit -m "feat(context): add context offloading to archive files"
```

---

## Phase 4: Environment & Secrets Handling

### Task 1: Add secrets masking for logging

**Objective:** Prevent secrets from appearing in logs

**Files:**
- Modify: `src/context/mod.rs:128-145` (Secrets impl)

**Step 1: Add mask_value() method**

```rust
/// Mask a secret value for safe logging (shows first 4 chars)
pub fn mask_value(&self, key: &str) -> String {
    match self.get(key) {
        Some(value) if value.len() > 4 => {
            format!("{}****", &value[..4])
        }
        Some(value) => "****".to_string(),
        None => "(not set)".to_string(),
    }
}

/// Get all keys (for enumeration without exposing values)
pub fn keys(&self) -> Vec<&String> {
    self.entries.keys().collect()
}
```

**Step 2: Add tests**

```rust
#[test]
fn test_secrets_mask_value() {
    let mut secrets = Secrets::new();
    secrets.set("api_key", "secret12345");
    
    assert_eq!(secrets.mask_value("api_key"), "secr****");
    assert_eq!(secrets.mask_value("nonexistent"), "(not set)");
}

#[test]
fn test_secrets_mask_short_value() {
    let mut secrets = Secrets::new();
    secrets.set("key", "abc");
    
    assert_eq!(secrets.mask_value("key"), "****");
}
```

**Step 3: Run tests**

Run: `cargo test --lib context::tests -v`
Expected: PASS

**Step 4: Commit**

```bash
git add src/context/mod.rs
git commit -m "feat(context): add secrets masking for safe logging"
```

---

### Task 2: Implement environment variable injection

**Objective:** Inject secrets as environment variables for agent processes

**Files:**
- Modify: `src/context/mod.rs` (add injection method)

**Step 1: Add to_json_env() method**

```rust
/// Convert secrets to environment variable map for process injection
/// Keys are prefixed with APP_ to avoid conflicts
pub fn to_env_map(&self) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (key, value) in &self.entries {
        let env_key = format!("APP_{}", key.to_uppercase().replace('-', "_"));
        map.insert(env_key, value.clone());
    }
    map
}
```

**Step 2: Add tests**

```rust
#[test]
fn test_secrets_to_env_map() {
    let mut secrets = Secrets::new();
    secrets.set("api-key", "secret123");
    secrets.set("DATABASE_URL", "postgres://localhost");
    
    let env = secrets.to_env_map();
    
    assert_eq!(env.get("APP_API_KEY"), Some(&"secret123".to_string()));
    assert_eq!(env.get("APP_DATABASE_URL"), Some(&"postgres://localhost".to_string()));
}
```

**Step 3: Run and commit**

```bash
cargo test --lib context::tests -v
git add src/context/mod.rs
git commit -m "feat(context): add secrets to env map conversion"
```

---

## Phase 5: Integration with Orchestrator

### Task 1: Create ContextManager service

**Objective:** High-level service for managing context lifecycle

**Files:**
- Create: `src/context/manager.rs`
- Modify: `src/context/mod.rs` (export new module)

**Step 1: Create ContextManager**

```rust
//! Context Manager - High-level context lifecycle management

use crate::context::{ContextStore, Secrets, Workspace, JsonContextStore};
use std::path::PathBuf;
use uuid::Uuid;

/// Manages context lifecycle: load, save, inject, compact
pub struct ContextManager {
    store: JsonContextStore,
    max_tokens_per_workspace: u32,
}

impl ContextManager {
    pub fn new(base_path: PathBuf, max_tokens: u32) -> Self {
        Self {
            store: JsonContextStore::new(base_path),
            max_tokens_per_workspace: max_tokens,
        }
    }

    /// Load workspace with auto-compaction if over token limit
    pub async fn load_workspace(&self, id: Uuid) -> anyhow::Result<Option<Workspace>> {
        let mut workspace = self.store.load_workspace(id).await?;
        
        if let Some(ref mut ws) = workspace {
            // Check and compact if needed
            for conv in ws.conversations.values_mut() {
                let total_tokens: u32 = conv.messages.iter()
                    .map(|m| m.content.len() as u32 / 4)
                    .sum();
                
                if total_tokens > self.max_tokens_per_workspace {
                    // Compact to 50%
                    conv.compact(0.5);
                }
            }
        }
        
        Ok(workspace)
    }

    /// Save workspace with secrets
    pub async fn save_workspace(&self, workspace: &Workspace, secrets: Option<&Secrets>) -> anyhow::Result<()> {
        self.store.save_workspace(workspace).await?;
        
        if let Some(sec) = secrets {
            self.store.save_secrets(workspace.id, sec).await?;
        }
        
        Ok(())
    }

    /// Load secrets for workspace
    pub async fn load_secrets(&self, workspace_id: Uuid) -> anyhow::Result<Option<Secrets>> {
        self.store.load_secrets(workspace_id).await
    }

    /// Create new workspace
    pub async fn create_workspace(&self, name: String) -> anyhow::Result<Workspace> {
        let workspace = Workspace::new(name);
        self.store.save_workspace(&workspace).await?;
        Ok(workspace)
    }

    /// List all workspaces
    pub async fn list_workspaces(&self) -> anyhow::Result<Vec<Workspace>> {
        self.store.list_workspaces().await
    }

    /// Delete workspace and associated data
    pub async fn delete_workspace(&self, id: Uuid) -> anyhow::Result<()> {
        self.store.delete_workspace(id).await
    }
}
```

**Step 2: Export from mod.rs**

```rust
pub mod manager;
pub use manager::ContextManager;
```

**Step 3: Add integration tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_context_manager_create_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ContextManager::new(temp_dir.path().to_path_buf(), 1000);
        
        let workspace = manager.create_workspace("Test".to_string()).await.unwrap();
        
        assert_eq!(workspace.name, "Test");
        
        let loaded = manager.load_workspace(workspace.id).await.unwrap();
        assert!(loaded.is_some());
    }

    #[tokio::test]
    async fn test_context_manager_with_secrets() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ContextManager::new(temp_dir.path().to_path_buf(), 1000);
        
        let workspace = manager.create_workspace("Test".to_string()).await.unwrap();
        
        let mut secrets = Secrets::new();
        secrets.set("api_key", "secret123");
        
        manager.save_workspace(&workspace, Some(&secrets)).await.unwrap();
        
        let loaded_secrets = manager.load_secrets(workspace.id).await.unwrap();
        assert!(loaded_secrets.is_some());
        assert_eq!(loaded_secrets.unwrap().get("api_key"), Some(&"secret123".to_string()));
    }
}
```

**Step 4: Run tests**

Run: `cargo test --lib context -v`
Expected: PASS (all tests)

**Step 5: Commit**

```bash
git add src/context/
git commit -m "feat(context): add ContextManager service for lifecycle management"
```

---

### Task 2: Integrate with AgentExecutor

**Objective:** Add context loading to the agent execution flow

**Files:**
- Modify: `src/agents/executor.rs`

**Step 1: Add context module to agents**

First check what's in `src/agents/mod.rs`:

```bash
cat src/agents/mod.rs
```

**Step 2: Add context integration**

Add to `src/agents/executor.rs`:

```rust
// Add import at top
use crate::context::ContextManager;
use std::path::PathBuf;

// Add field to AgentExecutor struct
pub struct AgentExecutor {
    // ... existing fields
    context_manager: Option<ContextManager>,
}

// Update new() to accept optional context manager
impl AgentExecutor {
    pub fn new(/* existing params */, context_path: Option<PathBuf>) -> Self {
        let context_manager = context_path.map(|p| ContextManager::new(p, 10000));
        Self {
            // ... existing fields
            context_manager,
        }
    }
}

// Add method to load context for a task
async fn load_task_context(&self, workspace_id: Option<Uuid>) -> anyhow::Result<Option<Value>> {
    match (&self.context_manager, workspace_id) {
        (Some(manager), Some(id)) => {
            let workspace = manager.load_workspace(id).await?;
            let secrets = manager.load_secrets(id).await?;
            
            // Build context object
            let mut context = serde_json::json!({
                "workspace_id": id.to_string(),
            });
            
            if let Some(ws) = workspace {
                context["workspace_name"] = serde_json::json!(ws.name);
                context["message_count"] = serde_json::json!(
                    ws.conversations.values().map(|c| c.messages.len()).sum::<usize>()
                );
            }
            
            if let Some(sec) = secrets {
                context["secrets_keys"] = serde_json::json!(sec.keys());
            }
            
            Ok(Some(context))
        }
        _ => Ok(None),
    }
}
```

**Step 3: Use context in execute_task**

In `execute_task` method, call `load_task_context` before execution:

```rust
// Before executing the task, load context
let task_context = self.load_task_context(args.workspace_id).await?;
```

**Step 4: Add compile-check test**

Run: `cargo check --lib`
Expected: SUCCESS (or fix any errors)

**Step 5: Commit**

```bash
git add src/agents/executor.rs src/agents/mod.rs
git commit -m "feat(agents): integrate ContextManager with AgentExecutor"
```

---

### Task 3: Final verification

**Step 1: Run full test suite**

Run: `cargo test --lib`
Expected: ALL PASS

**Step 2: Run clippy**

Run: `cargo clippy --lib -- -D warnings`
Expected: NO WARNINGS

**Step 3: Final commit**

```bash
git status
git add -A
git commit -m "feat: complete context management implementation (Phases 3-5)"
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 3 | 3 | Compaction & offloading (token tracking, compact(), offload) |
| 4 | 2 | Secrets handling (masking, env injection) |
| 5 | 3 | Orchestrator integration (ContextManager, executor integration) |

**Total: 8 tasks**

Each task ~2-5 minutes. Run tests after each task. Commit after each task passes.

---

**Plan complete and saved.** Ready to execute using subagent-driven-development — I'll dispatch a fresh subagent per task with two-stage review (spec compliance then code quality). Shall I proceed?