## Purpose

The Interactive Bash Session agent handles interactive bash sessions, including:
- TMUX session management
- Interactive command execution
- Session state tracking
- Terminal I/O handling

## Components

- `constants.rs`: Constant definitions for bash session operations
- `index.rs`: Main entry point and hook registration
- `storage.rs`: Session state storage and retrieval
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when interactive bash operations are performed. It manages TMUX sessions and ensures proper state tracking for interactive terminal operations.

## Configuration

The agent uses default session management settings and does not require specific configuration.
