## Purpose

The Agent Usage Reminder agent handles agent usage reminders, including:
- Agent usage pattern tracking
- Reminder injection at appropriate intervals
- Usage statistics collection
- Session-based reminder management

## Components

- `constants.rs`: Constant definitions for reminder operations
- `index.rs`: Main entry point and hook registration
- `storage.rs`: Usage state storage and retrieval
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked periodically during sessions to provide reminders about appropriate agent usage patterns and best practices.

## Configuration

The agent uses built-in reminder intervals and does not require specific configuration.
