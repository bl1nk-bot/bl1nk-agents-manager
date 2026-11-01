## Purpose

The Category Skill Reminder agent assists orchestrator agents with proper delegation, including:
- Category and skill system reminders
- Delegation guidance based on work type
- Proper tool usage recommendations
- Session state tracking for reminders

## Components

- `index.rs`: Main entry point and hook registration
- `types.rs`: Type definitions and interfaces

## Usage

This agent is invoked when orchestrator agents (like sisyphus, atlas) perform work that could potentially be delegated. It monitors tool usage and provides reminders after a threshold of non-delegation tool calls.

## Configuration

The agent uses built-in target agent detection and does not require specific configuration.
