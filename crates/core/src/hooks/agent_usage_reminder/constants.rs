use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::OnceLock;

// Mocking shared data path function (You should implement this in crate::shared)
fn get_opencode_storage_dir() -> PathBuf {
    // In a real implementation, this would use dirs::data_dir() or similar
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        PathBuf::from(home).join(".bl1nk")
    } else {
        PathBuf::from("/tmp/.bl1nk")
    }
}

pub fn opencode_storage() -> &'static PathBuf {
    static STORAGE: OnceLock<PathBuf> = OnceLock::new();
    STORAGE.get_or_init(get_opencode_storage_dir)
}

pub fn agent_usage_reminder_storage() -> &'static PathBuf {
    static STORAGE: OnceLock<PathBuf> = OnceLock::new();
    STORAGE.get_or_init(|| opencode_storage().join("agent-usage-reminder"))
}

pub fn target_tools() -> &'static HashSet<&'static str> {
    static TOOLS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    TOOLS.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert("grep");
        s.insert("safe_grep");
        s.insert("glob");
        s.insert("safe_glob");
        s.insert("webfetch");
        s.insert("context7_resolve-library-id");
        s.insert("context7_query-docs");
        s.insert("websearch_web_search_exa");
        s.insert("context7_get-library-docs");
        s.insert("grep_app_searchgithub");
        s
    })
}

pub fn agent_tools() -> &'static HashSet<&'static str> {
    static TOOLS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    TOOLS.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert("task");
        s.insert("call_omo_agent");
        s.insert("delegate_task");
        s
    })
}

pub const REMINDER_MESSAGE: &str = r#"[Agent Usage Reminder]

You called a search/fetch tool directly without leveraging specialized agents.

RECOMMENDED: Use delegate_task with explore/librarian agents for better results:

```
// Parallel exploration - fire multiple agents simultaneously
delegate_task(agent="explore", prompt="Find all files matching pattern X")
delegate_task(agent="explore", prompt="Search for implementation of Y") 
delegate_task(agent="librarian", prompt="Lookup documentation for Z")

// Then continue your work while they run in background
// System will notify you when each completes
```

WHY:
- Agents can perform deeper, more thorough searches
- Background tasks run in parallel, saving time
- Specialized agents have domain expertise
- Reduces context window usage in main session

ALWAYS prefer: Multiple parallel delegate_task calls > Direct tool calls
"#;
