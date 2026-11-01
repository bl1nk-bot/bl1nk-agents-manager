# Roadmap

## Phase 0: Stabilize the core
- Make `cargo check` and `cargo test` clean across the workspace.
- Resolve ACP session lifecycle and adapter consistency in `crates/core`.
- Finish the background agent system port and confirm task completion logic.
- Align the config schema, loader, and runtime wiring so config changes are
  actually applied.

## Phase 1: End-to-end Gemini flow
- Validate Gemini CLI ACP handshake, authentication, session creation, and
  prompting.
- Confirm agent selection, hooks, and permissions behave as documented.
- Document one verified, start-to-finish workflow.

## Phase 2: Second-provider parity
- Normalize provider-specific ACP behavior for Codex or Qwen.
- Ensure tool outputs and session events map consistently into the core.
- Validate the same workflow without additional configuration.

## Phase 3: Documentation and operator readiness
- Refresh documentation to match working behavior only.
- Provide a clear operator guide for setup, auth, and troubleshooting.
- Reduce duplicated or conflicting documentation.

## Risks and dependencies
- ACP behavior differences across providers can cause incompatible event shapes.
- OAuth and local callback flows require UI and port availability.
- Large diff surfaces increase regression risk without tests.

## Definition of done
- Workspace build and test suite pass.
- A Gemini CLI workflow completes reliably.
- A second provider runs the same workflow without reconfiguration.
- Documentation matches actual runtime behavior.
