# Plan: Gemini CLI Policy Engine Migration & Restructuring

## 1. Objective
- Migrate the agent permission system to match the exact Gemini CLI Policy Engine standards (Tiers 1-5, Priority 0-999, allow/deny/ask_user rules).
- Eradicate hardcoded models (`model` field) from agent definitions to ensure platform neutrality.
- Restructure configuration and schema files for better clarity and maintainability (removing `.config` ambiguity).

## 2. Key Files & Context
- **Current Data**: `agents/agents.json`
- **Current Schemas**: `.config/schema-agent.json`, `.config/registry-schema.json`, `.config/schema-config.json`
- **Current Rust Structs**: `src/config.rs` (`AgentConfig`)
- **Current Logic**: `src/registry/mod.rs` (`PolicyEvaluator`), `src/agents/router.rs`

## 3. Implementation Steps

### Phase 1: Directory & Naming Restructure
1. Rename the hidden `.config` directory to a visible `config` directory to signify it belongs to the project's core structure, not just a dotfile convention.
2. Rename schema files to be crystal clear about their purpose:
   - `.config/schema-agent.json` (Validates `agents/*.md` frontmatter) -> `config/agent-frontmatter.schema.json`
   - `.config/registry-schema.json` (Validates `agents.json`) -> `config/agent-registry.schema.json`
   - `.config/schema-config.json` (Validates `config.json/toml`) -> `config/app-config.schema.json`
3. Update all Rust code (`src/registry/mod.rs`, `src/config.rs`, `src/system/discovery.rs`) to point to the new `config/` paths.

### Phase 2: Schema Modernization (Gemini Policy Standard)
1. Update `config/agent-registry.schema.json`:
   - Remove required fields: `model`, `permission`, `tool_permissions`.
   - Add new fields: `tier` (integer 1-5), `priority` (integer 0-999).
   - Add `policies` array containing objects with `tool` (string) and `decision` (enum: allow, deny, ask_user).
2. Update `config/agent-frontmatter.schema.json` to reflect that `model` is no longer expected.

### Phase 3: Rust Structs & Deserialization
1. Edit `src/config.rs`:
   - Modify `AgentConfig` struct: remove old fields, add `tier: u8`, `priority: u16`, and `policies: Vec<PolicyRule>`.
   - Create `PolicyRule` struct with `tool: String`, `decision: String`.
2. Fix all compilation errors in tests (e.g., `src/agents/router.rs`) caused by the struct change.

### Phase 4: Policy Evaluator Overhaul
1. Edit `src/registry/mod.rs` (`PolicyEvaluator`):
   - Implement the condition-decision-priority logic.
   - For a given `tool_name`, find the matching rule in `agent.policies`.
   - Return the configured decision (`Allow`, `Deny`, `AskUser`).
   - Implement the Security Guard: If `tier == 2` (Extension) and the tool is dangerous (e.g., `bash`, `write`), force `AskUser` even if the rule says `Allow`.

### Phase 5: Data Migration (`agents.json`)
1. Write a Python script (`scripts/migrate_policies.py`) to read the current `agents/agents.json`.
2. Transform each agent:
   - Delete `model`, `permission`, `permission_policy`, `tool_permissions`.
   - Assign `tier` (e.g., Orchestrator=3, others=2) and `priority` (e.g., 100-900).
   - Generate the new `policies` array (e.g., `[{"tool": "bash", "decision": "deny"}]`).
3. Run the script and overwrite `agents.json`.

## 4. Verification & Testing
- Run `make parallel` to ensure:
  - `cargo fmt` and `cargo clippy` pass cleanly.
  - `cargo test` passes (specifically routing and policy evaluation tests).
  - Schema validation script (`validate_agents.py` updated with new paths) passes against the new `agents.json`.

## 5. Rollback Strategy
- If compilation fails during Phase 3/4 and cannot be easily resolved, run `git reset --hard HEAD` and `git clean -fd` to revert to the stable `v1.7.1` state before attempting a different approach.
