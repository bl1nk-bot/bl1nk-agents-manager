# Product

## Overview
Bl1nk Agents Manager is a Gemini CLI extension plus a Rust backend that provides
multi-agent orchestration through MCP and ACP. It is intended to normalize how
different agent CLIs (Gemini, Codex, Qwen, and others) are run, so users can
switch providers without reconfiguring every workflow.

## Who this is for
- Teams that need repeatable, shared agent workflows across projects.
- Operators who want a central place to manage hooks, permissions, and safety
  rules.
- Developers who need multiple specialized agents without rebuilding tool
  integrations each time.

## Problems this solves
- Fragmented setup and inconsistent behavior across agent CLIs.
- No single control plane for hooks, permissions, and agent selection.
- Difficulty reusing workflows when switching providers.

## Product scope
The product is a collection of aligned components, not a single binary.

- **Extension manifest**: `gemini-extension.json` is the entry point that
  registers the backend with Gemini CLI.
- **Rust core** (`crates/core`): provides agents, hooks, MCP/ACP adapters,
  sessions, RPC, filesystem, search, and project utilities.
- **Server crate** (`crates/server`): the HTTP server shell for remote or
  service-based deployments.
- **Front-end and CLI helpers** (`frontend/`, `commands/`): command definitions
  and supporting tooling.

## Current state (February 7, 2026)
- The extension manifest is present and points to the Rust binary.
- Core Rust modules exist for agents, hooks, adapters, sessions, and RPC, but
  not all subsystems are fully wired end-to-end.
- Large TypeScript-to-Rust ports are still in progress for configuration and
  background task orchestration.

## Constraints
- Authentication is provider-specific and still requires manual steps in some
  CLIs.
- ACP behavior varies by provider and needs explicit normalization.
- Some modules compile but do not yet deliver a complete end-to-end flow.

## Non-goals right now
- A fully stable consumer product across all providers.
- Zero-configuration setup for every provider.

## Success metrics
- `cargo check` and tests pass across the workspace without manual patches.
- A verified end-to-end Gemini CLI workflow runs with hooks and permissions.
- One additional provider (Codex or Qwen) runs the same workflow without new
  configuration.

## Primary user journey
1. Install the extension and build the Rust backend.
2. Start a Gemini CLI session and select a specialized agent.
3. Run a task that exercises hooks and permissions.
4. Switch to another provider and repeat the same workflow.
