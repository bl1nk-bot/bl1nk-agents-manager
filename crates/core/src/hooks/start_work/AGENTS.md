## Purpose

The Start Work agent handles work session initialization, including:
- Session state management
- Plan selection and initialization
- Workspace setup
- Task queue initialization

## Components

- `constants.rs`: Constant definitions for work session operations
- `index.rs`: Main entry point and hook registration
- `storage.rs`: Session state storage and retrieval
- `types.rs`: Type definitions and interfaces

## Usage

This agent is invoked when starting new work sessions. It handles plan selection, workspace initialization, and sets up the proper session state for work execution.

## Configuration

The agent uses default session initialization settings and does not require specific configuration.
