# Rust Code Style Guide

## 1. Project-Specific Conventions (bl1nk-agents-manager)

### 1.1 Module Structure
```
src/
├── main.rs           # Entry point, CLI parsing
├── config.rs         # Configuration parsing
├── mcp/              # MCP protocol implementation
│   ├── mod.rs        # Module exports + orchestrator
│   └── protocol.rs   # Protocol types
├── agents/           # Agent management
│   ├── mod.rs        # Module exports
│   ├── register.rs   # Agent registry
│   ├── router.rs     # Task routing
│   ├── extractor.rs  # Agent executor
│   └── creator.rs    # Agent creation
├── rate_limit.rs     # Rate limiting logic
├── system/           # System discovery
├── persistence/      # Data persistence
├── hooks/            # Hook system
└── permissions/      # Permission management
```

### 1.2 Naming Conventions
- **Modules:** `snake_case` filenames (`rate_limit.rs`, `agent_registry.rs`)
- **Structs/Enums:** `PascalCase` (`AgentConfig`, `RoutingTier`)
- **Functions/Methods:** `snake_case` (`load_default()`, `get_agents_by_capability()`)
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Type Aliases:** `PascalCase` with `Type` suffix if ambiguous

### 1.3 Module Exports
ใช้ explicit re-exports ใน `mod.rs`:
```rust
pub mod register;
pub mod router;
pub use register::AgentRegistry;
pub use router::AgentRouter;
```

---

## 2. Formatting Rules (rustfmt)

### 2.1 Base Configuration
```toml
edition = "2024"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
indent_style = "Block"
```

### 2.2 Import Organization
```rust
// เรียงตามกลุ่ม: std → external → crate
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::agents::AgentRegistry;
```

### 2.3 Code Style Rules
- **Line length:** 100 characters max
- **Where clauses:** Mixed style (prefer single-line when short)
- **Match arms:** Wrap when complex, inline when simple
- **Struct literals:** Max width 100 before multi-line

---

## 3. Documentation & Comments (ภาษาไทย)

### 3.1 Rustdoc Comments
ใช้ `///` สำหรับ public API:
```rust
/// โหลดค่ากำหนดจากพาธ
///
/// # Arguments
/// * `path` - พาธไปยังไฟล์ config
///
/// # Returns
/// * `Ok(Config)` - ถ้าโหลดสำเร็จ
/// * `Err` - ถ้าไฟล์ไม่พบหรือ parse ไม่ได้
///
/// # Examples
/// ```no_run
/// let config = Config::load("./config.toml")?;
/// ```
pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
```

### 3.2 Internal Comments
ใช้ `//` พร้อมภาษาไทยสำหรับ logic ภายใน:
```rust
// ตรวจสอบว่ามี agent ID ซ้ำหรือไม่
let mut seen_ids = std::collections::HashSet::new();
for agent in &self.agents {
    if !seen_ids.insert(&agent.id) {
        anyhow::bail!("Duplicate agent ID: {}", agent.id);
    }
}

// TODO: เพิ่มการ validate rate limiting แบบ distributed
```

### 3.3 TODO Convention
```rust
// TODO: <รายละเอียดสิ่งที่ต้องทำ>
// TODO: ย้าย logic การ routing ไปใช้ BM25 แทน keyword matching
// TODO: เพิ่มการเชื่อมต่อ modal.com sandbox
```

---

## 4. Error Handling

### 4.1 Use `anyhow` for Application Code
```rust
use anyhow::{Context, Result};

pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
    let content = fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

    let config: Config = toml::from_str(&content)
        .context("Failed to parse TOML config")?;

    Ok(config)
}
```

### 4.2 Use `thiserror` for Library Code
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent '{0}' not found")]
    NotFound(String),

    #[error("Rate limit exceeded for agent '{0}'")]
    RateLimitExceeded(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

### 4.3 Error Messages (Verbose + Recovery)
```rust
tracing::error!(
    agent_id = %agent.id,
    error = %e,
    "❌ Failed to execute agent - ตรวจสอบ command path และ permissions"
);
```

---

## 5. Async & Concurrency

### 5.1 Tokio Patterns
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // ใช้ Arc<RwLock<T>> สำหรับ shared state
    let registry = Arc::new(RwLock::new(AgentRegistry::new()));

    // Clone Arc ก่อนเข้า closure
    let registry_clone = registry.clone();
    tokio::spawn(async move {
        // ใช้งาน registry_clone
    });

    Ok(())
}
```

### 5.2 Read/Write Lock Usage
```rust
// อ่านข้อมูล (multiple readers)
let config = self.config.read().await;

// แก้ไขข้อมูล (exclusive writer)
let mut config = self.config.write().await;
```

---

## 6. Testing Standards

### 6.1 Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config_str = r#"
            [server]
            host = "127.0.0.1"
            port = 3000
        "#;
        let config: Config = toml::from_str(config_str).unwrap();
        assert_eq!(config.server.max_concurrent_tasks, 5);
    }
}
```

### 6.2 Async Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_routing() {
        let registry = AgentRegistry::new(vec![], None);
        let result = registry.route_task("code-gen").await;
        assert!(result.is_ok());
    }
}
```

### 6.3 Coverage Requirement
- **Minimum:** 80% line coverage
- **Critical paths:** 100% (routing, rate limiting, permissions)
- **Test types:** unit + integration + property-based (proptest)

---

## 7. CLI Conventions

### 7.1 Output Format
```rust
// ใช้ emoji สำหรับ status
tracing::info!("🚀 Starting BL1NK Agents Manager");
tracing::info!("✅ Loaded {} agents", config.agents.len());
tracing::error!("❌ Failed to load config: {}", e);

// Progress indicators สำหรับ long operations
tracing::info!("🔍 Scanning system resources...");
```

### 7.2 Command Structure
```rust
#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Delegate a task to an agent (CLI interactive mode)
    Delegate {
        #[arg(short, long)]
        task_type: String,

        #[arg(short, long)]
        prompt: String,
    },
}
```

---

## 8. Configuration Standards

### 8.1 TOML Config Structure
```toml
[server]
host = "127.0.0.1"
port = 3000
max_concurrent_tasks = 5

[main_agent]
name = "gemini"
type = "gemini-cli"

[[agents]]
id = "code-generator"
name = "Code Generator"
type = "cli"
command = "gemini"
capabilities = ["code-generation"]
enabled = true
priority = 100

[routing]
tier = "default"
rules = []

[rate_limiting]
strategy = "round-robin"
track_usage = true

[logging]
level = "info"
output = "stdout"
```

### 8.2 Config Path Resolution (Priority Order)
1. `--config <path>` (CLI flag)
2. `./config.toml` (current directory)
3. `./.bl1nk-agents-manager.toml` (current directory, hidden)
4. `~/.config/bl1nk-agents-manager/config.toml` (user config)
5. `~/.bl1nk-agents-manager.toml` (user home, hidden)

---

## 9. Security Guidelines

### 9.1 No `unsafe` Blocks
- **ห้ามใช้** `unsafe` ยกเว้นมีเหตุผลชัดเจน
- ต้องผ่าน review และคอมเมนต์อธิบายว่าทำไมจำเป็น

### 9.2 Secrets Management
```rust
// ❌ อย่าเก็บ API keys ใน config ตรงๆ
// ✅ ใช้ environment variables หรือ secret managers
let token = std::env::var("AGENT_TOKEN")
    .context("AGENT_TOKEN environment variable not set")?;
```

### 9.3 Permission Sandboxing
```rust
// ตรวจสอบ commands ที่ agent รันได้
pub fn is_command_allowed(command: &str) -> bool {
    ALLOWED_COMMANDS.contains(&command)
}
```

---

## 10. Linting & CI

### 10.1 Required Checks
```bash
# ก่อน commit ทุกครั้ง
cargo fmt --check
cargo clippy -- -D warnings
cargo test --all-features
```

### 10.2 Clippy Pedantic (Optional)
```bash
cargo clippy -- -W clippy::pedantic -A clippy::missing-errors-doc
```

---

## 11. Platform-Specific Code

### 11.1 Conditional Compilation
```rust
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
```

### 11.2 Android/Termux Support
```rust
// ใช้ path ที่รองรับ Termux
let config_dir = if cfg!(target_os = "android") {
    PathBuf::from("/data/data/com.termux/files/home/.config")
} else {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"))
};
```
