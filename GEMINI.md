## 🌟 What are System Agents?

System Agents are specialized agent definitions that define how the Gemini CLI behaves. Instead of a generic assistant, you can load a specific persona with expert knowledge, unique speech patterns, or strict behavioral constraints.

This extension provides:

1. **A Curated Library**: 48+ high-quality, pre-tested agents for engineering, writing, research, and entertainment.
2. **Management Commands**: CLI tools to list, inspect, switch, and create agents.
3. **Extensibility**: A structure to add your own custom agents.
4. **Hooks System**: 35+ hooks for automation and context injection.

---

## 🎭 Available Agents

The extension comes with a built-in library of agents located in the `agents/` directory.

### 🛠️ Engineering & Development

| Agent | ID | Description |
|-------|-----|-------------|
| Architect | `architect` | Software architecture and design planning |
| Code Generator | `code-generator` | Rapid, clean code generation |
| Code Reviewer | `code-reviewer` | Bug detection and code quality |
| Code Explorer | `code-explorer` | Deep codebase analysis |
| Code Architect | `code-architect` | Feature architecture design |
| Cloudflare | `cloudflare` | Cloudflare Workers and Agents |
| Fullstack Dev | `fullstack-dev` | Full-stack application development |
| Orchestrator | `orchestrator` | Task delegation and routing |

### 🔍 Research & Analysis

| Agent | ID | Description |
|-------|-----|-------------|
| Codebase Analyzer | `codebase-analyzer` | Implementation detail analysis |
| Codebase Locator | `codebase-locator` | File and component discovery |
| Codebase Pattern Finder | `codebase-pattern-finder` | Similar implementation search |
| Research Analyzer | `research-analyzer` | Research document analysis |
| Thoughts Analyzer | `thoughts-analyzer` | Deep research on topics |
| Web Search Researcher | `web-search-researcher` | Web content research |

### 📝 Documentation & Planning

| Agent | ID | Description |
|-------|-----|-------------|
| Docbot Pro | `docbot-pro` | Enterprise documentation |
| Docs Researcher | `docs-researcher` | Library documentation |
| Insight Documenter | `insight-documenter` | Technical breakthrough docs |
| Plan Implementation Reviewer | `plan-implementation-reviewer` | Plan validation |

### 🛠️ Utilities & Tools

| Agent | ID | Description |
|-------|-----|-------------|
| Agent Creator | `agent-creator` | Create new agents |
| Command Creator | `command-creator` | Create Claude Code commands |
| Skill Creator | `skill-creator` | Create new skills |
| Skill Reviewer | `skill-reviewer` | Skill quality review |
| Plugin Validator | `plugin-validator` | Plugin structure validation |
| Task Management | `task-management` | Task tracking and context |
| UI Engineer | `ui-engineer` | Frontend/UI development |

### 🎪 Entertainment & Comedy

| Agent | ID | Description |
|-------|-----|-------------|
| Creative Writer | `creative-writer` | Poetry, prose, storytelling |
| Pirate | `pirate` | Pirate dialect assistant |
| Yoda | `yoda` | Yoda-speak assistant |
| Shakespeare | `shakespeare` | Shakespearean English |
| Cowboy | `cowboy` | Western dialect |
| Gen Z | `gen-z` | Gen Z slang |
| Comedian | `comedian` | Dad jokes |

---

## 🚀 Commands

This extension registers the `/system-agent` command namespace.

| Command | Description | Usage |
|---------|-------------|-------|
| `/system-agent` | List all available agents | `/system-agent` |
| `/system-agent:info` | Get detailed metadata for an agent | `/system-agent:info <agent_id>` |
| `/system-agent:switch` | Get instructions to switch agents | `/system-agent:switch <agent_id>` |
| `/system-agent:examples` | Show example prompts for an agent | `/system-agent:examples <agent_id>` |
| `/system-agent:new` | Interactive wizard to create a new agent | `/system-agent:new` |

---

## 📌 Gemini CLI Cheatsheet

For day-to-day CLI usage and options, see the local cheatsheet:

- `docs/GEMINI_CLI_CHEATSHEET.md`

---

## 💡 How to Switch Agents

**Important:** You cannot switch agents in the *middle* of a running session because the system prompt is loaded at startup. To switch agents, you must set the `GEMINI_SYSTEM_MD` environment variable and start a new session.

The `/system-agent:switch` command will generate the exact commands you need.

**Common Methods:**

1. **Temporary (One-off session):**

   ```bash
   GEMINI_SYSTEM_MD=~/.gemini/extensions/agents-manager/agents/pirate.md gemini
   ```

2. **Persistent (Until shell exit):**

   ```bash
   export GEMINI_SYSTEM_MD=~/.gemini/extensions/agents-manager/agents/architect.md
   gemini
   ```

3. **Aliases (Recommended):**
   Add these to your `.bashrc` or `.zshrc`:

   ```bash
   alias gemini-pirate="GEMINI_SYSTEM_MD=~/.gemini/extensions/agents-manager/agents/pirate.md gemini"
   alias gemini-code="GEMINI_SYSTEM_MD=~/.gemini/extensions/agents-manager/agents/code-generator.md gemini"
   ```

---

## 📂 Project Structure

For developers extending this project:

```
/
├── gemini-extension.json   # Extension manifest
├── agents/                 # Built-in agent definitions
│   ├── agents.json         # Registry of 48+ agents
│   ├── *.md               # Agent system prompt files
│   └── README.md          # Agent documentation
├── commands/              # Command definitions (.toml)
│   ├── system-agent.toml  # Main /system-agent command
│   └── agent/             # Subcommands (:switch, :info, etc.)
├── crates/
│   ├── core/              # Core library (16 agent modules, 35+ hooks)
│   └── server/            # HTTP/Rocket server
├── hooks/                 # Hook definitions (35+ hooks)
├── skills/                # AI skill definitions
└── docs/                  # Documentation
```

### Adding a New Agent

1. Create a new `.md` file in `agents/`
2. Add the agent's metadata to `agents/agents.json`
3. (Optional) If it's a "custom" user agent, the logic in `system-agent.toml` looks for a `custom/agents.json` file relative to the extension path

### Creating Custom Hooks

1. Create a new module in `crates/core/src/hooks/`
2. Implement the hook interface
3. Register in `crates/core/src/hooks/mod.rs`

---

## 🧩 Hooks System

Bl1nk includes 35+ hooks for advanced automation:

### Context Injection

- `directory_agents_injector` - Inject agent context from directory
- `directory_readme_injector` - Inject README context
- `compaction_context_injector` - Context compaction
- `ralph_loop` - Loop detection and recovery

### Monitoring & Recovery

- `context_window_monitor` - Memory usage monitoring
- `session_recovery` - Session recovery
- `empty_task_response_detector` - Detect empty responses
- `edit_error_recovery` - Recover from edit errors

### Task Management

- `todo_continuation_enforcer` - Enforce TODO completion
- `category_skill_reminder` - Category/skill reminders
- `task_resume_info` - Task resumption info
