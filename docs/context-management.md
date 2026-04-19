# Context Management

ระบบจัดการ conversation context แบบ cross-project สำหรับ BL1NK Agents Manager

## Storage Structure

### Global (`~/.bl1nk/`)

```
~/.bl1nk/
├── sessions/              # Context ข้าม sessions
│   ├── workspaces.json    # Index ของ workspaces ทั้งหมด
│   ├── {uuid}.json        # Workspace data (flat structure)
│   └── secrets/          # API keys (แยกเก็บ - ไม่ git)
├── memory/                # Long-term memory
├── hooks/                 # Global hooks
├── skills/                # Global skills
├── extensions/            # Extensions
├── cache/                 # Model cache
└── config.yaml            # Global config
```

### Per-project (`.bl1nk/`)

```
.bl1nk/
├── todo.md               # Project todo
└── checkpoints/         # Checkpoints per project
```

## Usage

### Initialize Global Store

```rust
use bl1nk_agents_manager::context::{ContextStore, JsonContextStore, context_dir};

let store = JsonContextStore::new(context_dir());
// หรือ custom path:
// let store = JsonContextStore::new(PathBuf::from("/custom/path"));
```

### Save/Load Workspace

```rust
use bl1nk_agents_manager::context::{Workspace, ContextStore, JsonContextStore};
use uuid::Uuid;

let store = JsonContextStore::new(context_dir());

let workspace = Workspace::new("My Project".to_string());
store.save_workspace(&workspace).await?;

let loaded = store.load_workspace(workspace.id).await?;
```

### Secrets Management

```rust
use bl1nk_agents_manager::context::{Secrets, ContextStore};

let secrets = Secrets::new();
secrets.set("openai_api_key", "sk-xxx");

store.save_secrets(workspace.id, &secrets).await?;
let loaded_secrets = store.load_secrets(workspace.id).await?;
```

## Data Structures

### Message

```rust
pub struct Message {
    pub role: MessageRole,    // User, Assistant, System
    pub content: String,      // ข้อความ
    pub timestamp: DateTime<Utc>,
}
```

### Conversation

```rust
pub struct Conversation {
    pub id: Uuid,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
}
```

### Workspace

```rust
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub conversations: Vec<Conversation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Secrets

```rust
pub struct Secrets {
    data: HashMap<String, String>,
}
```

## ContextStore Trait

```rust
#[async_trait]
pub trait ContextStore: Send + Sync {
    async fn save_workspace(&self, workspace: &Workspace) -> anyhow::Result<()>;
    async fn load_workspace(&self, id: Uuid) -> anyhow::Result<Option<Workspace>>;
    async fn list_workspaces(&self) -> anyhow::Result<Vec<Workspace>>;
    async fn delete_workspace(&self, id: Uuid) -> anyhow::Result<()>;
    async fn save_secrets(&self, workspace_id: Uuid, secrets: &Secrets) -> anyhow::Result<()>;
    async fn load_secrets(&self, workspace_id: Uuid) -> anyhow::Result<Option<Secrets>>;
}
```

## Testing

```bash
# Run all context tests
cargo test context -- --test-threads=1

# Run specific test
cargo test json_store::tests::test_save_and_load_workspace
```

## Related

- [Context Compaction](./context-compaction.md) - Token optimization
- [Tool Compaction](./tool-compaction.md) - Minimize tool call overhead
