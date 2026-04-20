# Plan: Registry Architecture & Policy Redesign (v1.7.5)

## 1. Objective
Refactor the agent registry (`agents.json`) and associated Rust structures to use a scalable, hierarchical, object-based design. This addresses hardcoded limitations (e.g., assuming only 4 tools) and prepares for future agent sourcing (Git, Remote, Built-in).

## 2. Structural Changes

### 2.1 Agent Definition (`agents.json`)
- **`version`**: Add a version string to each agent (e.g., "1.0.0").
- **`source`**: Replace the flat `file` string with a polymorphic object:
  ```json
  "source": {
    "type": "builtin",
    "path": "agents/system-architect.md"
  }
  ```
  *(Future types could be `git`, `local`, `url`)*
- **`policies`**: Change from an array of rules to a nested object indexed by tool name:
  ```json
  "policies": {
    "tools": {
      "AskUserQuestion": "allow",
      "ExitPlanMode": "allow",
      "Glob": "allow",
      "Grep": "allow",
      "ListFiles": "allow",
      "ReadFile": "allow",
      "SaveMemory": "allow",
      "Skill": "allow",
      "TodoWrite": "allow",
      "WebFetch": "allow",
      "WebSearch": "allow",
      "WriteFile": "allow",
      "bash": "deny",
      "rm": "deny"
    }
  }
  ```
  *(Note: The value can be a string "allow"|"deny"|"ask_user" or an object if modes are needed later, but a string is cleanest for now)*

## 3. Implementation Steps

### Phase 1: Update JSON Schemas
1. Modify `config/v1.7/policy-schema.json`:
   - Add `version` to the agent object.
   - Replace `file` with a `source` object.
   - Restructure `policies` to be an object containing a `tools` map (string -> string enum).

### Phase 2: Update Rust Types (`src/registry/schema.rs`)
1. Update `AgentJsonEntry`:
   - Add `pub version: String`.
   - Replace `pub file: String` with `pub source: AgentSource`.
   - Replace `pub policies: Vec<PolicyRuleJson>` with `pub policies: AgentPoliciesJson`.
2. Define `AgentSource` (Enum with `tag = "type"`).
3. Define `AgentPoliciesJson` (Struct containing `tools: HashMap<String, String>`).

### Phase 3: Update Core Configuration (`src/config.rs`)
1. Update `AgentConfig`:
   - Change `pub policies` to a `HashMap<String, String>` mapping tool names to decisions.
2. Update the deserialization/mapping logic in `auto_discover_agents` to handle the new `source` and `policies` structures. Extract the tool list from the discovered YAML frontmatter to dynamically populate the allowed tools in the fallback agent config.

### Phase 4: Refactor Policy Evaluator (`src/registry/mod.rs`)
1. Update `PolicyEvaluator::evaluate`:
   - Perform an O(1) lookup on `agent.policies.get(tool_name)`.
   - Fallback to `agent.policies.get("*")` or default to `Deny`.

### Phase 5: Data Migration
1. Write `scripts/migrate_v1_7_5.py` to rewrite `agents/agents.json`:
   - Set `"version": "1.0.0"` for all agents.
   - Wrap `"file": "..."` into `"source": {"type": "builtin", "path": "agents/..."}`.
   - Convert the old `policies` array into the new `policies: { tools: { ... } }` dictionary. Explicitly list default decisions for all 12 standard tools based on previous tool_permissions logic (e.g. if `skill` was true, set `Skill`: "allow").

## 4. Verification
- `cargo check` and `cargo test --all-features`
- `python3 scripts/validate_agents.py` to verify the new JSON format against the updated schema.
- Commit the changes as `refactor(registry): upgrade to nested tool policies and polymorphic sources (v1.7.5)`
