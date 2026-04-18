# Hermes Tool Implementations Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Implement core tools for the Hermes CLI agent in Rust: bash, fetch, glob, grep, read_file, skill, task, todo, write_file

**Architecture:** Create a new `src/tools/` module with separate files per tool. Each tool follows Hermes tool pattern: schema + execute function + tests.

**Tech Stack:** Rust (2021), serde, tokio, existing bl1nk-agents-manager codebase

**Existing:** `src/tools/ask_user_question.rs` (reference pattern), `src/config.rs` (tool permissions)

---

## Task 1: Tool Module Infrastructure

**Objective:** Set up module structure and shared types

**Files:**
- Create: `src/tools/mod.rs` (already exists - extend)
- Create: `src/tools/mod.rs:10` - Add tool exports

**Step 1: Extend tools module**

```rust
// Add to src/tools/mod.rs
pub mod bash;
pub mod fetch;
pub mod file_glob;
pub mod grep;
pub mod read_file;
pub mod skill;
pub mod task;
pub mod todo;
pub mod write_file;

pub use bash::*;
pub use fetch::*;
pub use file_glob::*;
pub use grep::*;
pub use read_file::*;
pub use skill::*;
pub use task::*;
pub use todo::*;
pub use write_file::*;
```

**Step 2: Run test**

Run: `cargo check --lib -p bl1nk-agents-manager`
Expected: ERROR — module not found (will fix in subsequent tasks)

**Step 3: Commit**

```bash
git add src/tools/mod.rs
git commit -m "refactor(tools): add tool module infrastructure"
```

---

## Task 2: Bash Tool

**Objective:** Implement bash execution tool

**Files:**
- Create: `src/tools/bash.rs`

**Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bash_tool_execution() {
        let result = execute_bash("echo hello");
        assert!(result.is_ok());
    }
}
```

**Step 2: Run test**

Run: `cargo test --lib tools::bash -v`
Expected: ERROR — module not found

**Step 3: Create bash tool**

```rust
//! Bash execute tool

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashInput {
    pub command: String,
    pub timeout: Option<u64>,
    pub workdir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Execute a bash command
pub fn execute_bash(input: &BashInput) -> Result<BashOutput, String> {
    // Use std::process::Command
    // Full implementation with timeout support
}
```

**Step 4: Run test**

Run: `cargo test --lib tools::bash -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/tools/bash.rs
git commit -m "feat(tools): add bash execute tool"
```

---

## Task 3: Read file Tool

**Objective:** Implement file reading tool

**Files:**
- Create: `src/tools/read_file.rs`

**Step 1: Write test + implement (similar to Task 2)**

```rust
//! read_file tool - read file contents

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileInput {
    pub path: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileOutput {
    pub content: String,
    pub total_lines: Option<usize>,
}

pub fn read_file(input: &ReadFileInput) -> Result<ReadFileOutput, String> {
    // Use tokio::fs to read file
    // Support offset/limit for pagination
}
```

**Step 2: Run + commit**

---

## Task 4: write_file Tool

**Objective:** Implement file writing tool

**Files:**
- Create: `src/tools/write_file.rs`

**Step 1: write test + implement**

```rust
//! write_file tool

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileInput {
    pub path: String,
    pub content: String,
    pub append: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileOutput {
    pub bytes_written: usize,
}

pub fn write_file(input: &WriteFileInput) -> Result<WriteFileOutput, String> {
    // Use tokio::fs::write
    // Support append mode
}
```

**Step 2: Run + commit**

---

## Task 5: Glob Tool

**Objective:** Implement file glob tool

**Files:**
- Create: `src/tools/file_glob.rs`

**Step 1: write test + implement**

```rust
//! glob tool - find files by pattern

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobInput {
    pub pattern: String,
    pub path: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobOutput {
    pub matches: Vec<String>,
}

pub fn glob(input: &GlobInput) -> Result<GlobOutput, String> {
    // Use glob crate for pattern matching
}
```

**Step 2: Run + commit**

---

## Task 6: Grep Tool

**Objective:** Implement grep search tool

**Files:**
- Create: `src/tools/grep.rs`

**Step 1: write test + implement**

```rust
//! grep tool - search file contents

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepInput {
    pub pattern: String,
    pub path: Option<String>,
    pub context: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepOutput {
    pub matches: Vec<GrepMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepMatch {
    pub file: String,
    pub line: usize,
    pub content: String,
}

pub fn grep_search(input: &GrepInput) -> Result<GrepOutput, String> {
    // Use regex + file iteration
}
```

**Step 2: Run + commit**

---

## Task 7: Fetch Tool

**Objective:** Implement HTTP fetch tool

**Files:**
- Create: `src/tools/fetch.rs`

**Step 1: write test + implement**

```rust
//! fetch tool - HTTP requests

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchInput {
    pub url: String,
    pub method: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOutput {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

pub fn fetch(input: &FetchInput) -> Result<FetchOutput, String> {
    // Use reqwest or ureq for HTTP
}
```

**Step 2: Run + commit**

---

## Task 8: Skill Tool

**Objective:** Implement skill invocation tool (re-use existing)

**Files:**
- Modify: `src/tools/mod.rs` (already has skill discovery)

**Step 1: Check existing implementation**

Run: `search_files("skill_discovery", path="src/")`

**Step 2: Create wrapper if needed + commit**

---

## Task 9: Task Tool

**Objective:** Implement task management tool

**Files:**
- Create: `src/tools/task.rs`

**Step 1: write test + implement**

```rust
//! task tool - manage todos

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInput {
    pub action: String, // "list", "add", "complete", "remove"
    pub id: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutput {
    pub tasks: Vec<TaskItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: String,
    pub content: String,
    pub status: String,
}

pub fn task(input: &TaskInput) -> Result<TaskOutput, String> {
    // Use persistence layer or local state
}
```

**Step 2: Run + commit**

---

## Task 10: Todo Tool

**Objective:** Implement todo list tool (alias or separate)

**Note:** May be identical to Task 9 or separate. Decide during implementation.

---

## Task 11: Integration Tests

**Objective:** Verify all tools work together

**Files:**
- Modify: `src/tools/mod.rs`

**Step 1: Run all tool tests**

Run: `cargo test --lib tools -v`
Expected: All tool tests pass

**Step 2: Commit**

```bash
git add src/tools/
git commit -m "test(tools): add integration tests"
```

---

## Verification Checklist

- [ ] Tools module compiles
- [ ] bash tool executes commands
- [ ] read_file reads with pagination
- [ ] write_file writes with append
- [ ] glob finds files by pattern
- [ ] grep searches content
- [ ] fetch makes HTTP requests
- [ ] skill invokes skills
- [ ] task manages tasks
- [ ] All tests pass