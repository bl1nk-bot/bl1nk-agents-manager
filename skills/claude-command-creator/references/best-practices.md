## 1. Naming Conventions
- **Verb-Noun**: `create-component`, `run-tests`.
- **Categories**: Use prefixes for grouping.
  - `gh:` for GitHub tasks.
  - `ci:` for continuous integration.
  - `sys:` for system maintenance.
  - `gen:` for code generation.

## 2. Tool Permissions (`allowed-tools`)
Restrict tool access to the minimum required for security:
- **Read-only**: `Read, LS`
- **File modification**: `Write`
- **Restricted Bash**: `Bash(git:*), Bash(npm:*)`
- **Full access** (use sparingly): `*`

## 3. Error Handling Workflow
1. **Pre-flight Checks**: Verify arguments and environment before acting.
2. **Graceful Fallback**: If a primary tool fails, suggest an alternative or explain why.
3. **Verification**: Always check if the action was successful (e.g., check if a file was created or a command returned exit code 0).

## 4. Documentation
Include examples in the command file so users know how to call it:
```markdown
## Examples:
- `/gen:component Header`
- `/gen:component Button --variant primary`
```
