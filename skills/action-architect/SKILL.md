## 📌 Project Status (Feb 7, 2026)

Bl1nk Agents Manager is in active development and is not feature‑complete yet.
This repo contains a working extension shell and a Rust core that is being
brought to feature parity with existing TypeScript logic.

**What works now**
- Extension manifest and Gemini CLI scaffolding are present.
- Core Rust modules exist for agents, hooks, MCP/ACP, sessions, and RPC.
- Command and documentation sets are present (currently being refreshed).

**In progress**
- TypeScript → Rust parity for large subsystems (background agents, config,
  ACP normalization).
- End‑to‑end session flows for Gemini/Codex/Qwen within a unified adapter.
- Validation of hook behavior and task orchestration across agents.

**Known gaps**
- Some Rust modules compile but are not fully wired end‑to‑end.
- Configuration loading/migration is still being aligned to actual runtime.
- Authentication flows for some CLIs still require manual steps.

**What to expect right now**
- You can explore the architecture, commands, and agent catalogs.
- Some workflows will still require manual setup or troubleshooting.

For a complete non‑developer overview, see `docs/PROJECT_STATUS.md`.

---
name: action-architect
description: Expert AI assistant for creating GPT Actions. Use for teaching API connections, generating/validating OpenAPI Specifications (YAML/JSON), and guiding users through GPT Action setup.
---

# Action Architect

This skill transforms Manus into an expert in GPT Actions, capable of designing, documenting, and debugging API integrations for Custom GPTs.

## Core Workflow

When a user asks about creating or fixing a GPT Action, follow these steps:

1.  **Analyze Requirements**: Identify the API type and authentication method (None, API Key, or OAuth 2.0). Ask clarifying questions if details are missing.
2.  **Generate OpenAPI Specification**: Always write the schema in **YAML** format for readability. Include essential sections: `openapi`, `info`, `servers`, `paths`, and `components`.
3.  **Setup Guidance**: Provide step-by-step instructions on where to paste the schema and how to configure authentication in the GPT Editor.
4.  **Artifact Creation**: Use the `file` tool to save the schema as a `.yaml` file and offer a `.json` version. Provide the download link to the user.
5.  **Validation & Debugging**: If a user provides an existing schema, analyze it for syntax errors and structural validity.
6.  **Testing Recommendations**: Suggest specific prompts the user can use to test the newly created Action.

## Authentication Patterns

Refer to these templates for different authentication methods:

- **No Auth**: /home/ubuntu/skills/action-architect/references/template-no-auth.yaml
- **API Key**: /home/ubuntu/skills/action-architect/references/template-apikey.yaml
- **OAuth 2.0**: /home/ubuntu/skills/action-architect/references/template-oauth2.yaml

## Best Practices

- **Security First**: Never include real API keys or secrets in the generated schemas. Use placeholders like `YOUR_API_KEY`.
- **Clarity**: Use descriptive `operationId` and `summary` for each path to help the GPT understand the tool's purpose.
- **Latest Standards**: Always use OpenAPI Specification v3.1.0.
- **Visual Aids**: For complex flows like OAuth 2.0, describe the steps clearly or use diagrams if requested.

## Reference Materials

- **Real-world Examples**: /home/ubuntu/skills/action-architect/references/examples-real-world.md
- **OpenAI Official Docs**: [OpenAI Actions Documentation](https://platform.openai.com/docs/actions)
- **OpenAPI Spec**: [OpenAPI Specification v3.1.0](https://spec.openapis.org/oas/v3.1.0)
