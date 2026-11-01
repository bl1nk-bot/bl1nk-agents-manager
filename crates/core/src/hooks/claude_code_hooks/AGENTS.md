## Purpose

The Claude Code Hooks agent handles various code-related hooks and operations specific to the Claude environment, including:
- Tool execution before/after processing
- Context window monitoring
- Session state management
- Error recovery mechanisms

## Components

- `config.rs`: Configuration management for Claude code hooks
- `config_loader.rs`: Dynamic configuration loading and validation
- `executor.rs`: Core execution logic for Claude-specific operations
- `index.rs`: Main entry point and hook registration
- `parser.rs`: Input/output parsing and validation
- `plugin_config.rs`: Plugin-specific configuration settings
- `post_tool_use.rs`: Post-tool execution processing
- `pre_compact.rs`: Pre-compaction operations
- `pre_tool_use.rs`: Pre-tool execution processing
- `stop.rs`: Session stopping and cleanup operations
- `todo.rs`: Task management and tracking
- `tool_input_cache.rs`: Caching for tool inputs
- `transcript.rs`: Conversation transcript management
- `types.rs`: Type definitions and interfaces
- `user_prompt_submit.rs`: User prompt submission handling

## Usage

This agent is automatically invoked when Claude-specific code operations are performed. It ensures proper handling of code-related tasks and maintains session state consistency.

## Configuration

The agent can be configured through the main configuration system. See the main documentation for configuration options.
