## Purpose

The Auto Slash Command agent handles automatic slash command processing, including:
- Slash command detection in user input
- Automatic command execution
- Command validation and preprocessing
- Context-aware command handling

## Components

- `constants.rs`: Constant definitions for command processing
- `detector.rs`: Command detection logic
- `executor.rs`: Command execution logic
- `mod.rs`: Module definitions and exports
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when user input contains potential slash commands. It detects commands like /plan, /start-work, etc., and processes them automatically.

## Configuration

The agent uses built-in command detection patterns and does not require specific configuration.
