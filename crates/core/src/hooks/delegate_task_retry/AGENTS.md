## Purpose

The Delegate Task Retry agent handles delegate task failures, including:
- Error pattern detection for delegate tasks
- Automatic retry guidance generation
- Failure recovery mechanisms
- Parameter correction suggestions

## Components

- `index.rs`: Main entry point and hook registration
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when delegate_task operations fail. It detects common error patterns and provides immediate retry guidance to fix the parameters.

## Configuration

The agent uses built-in error pattern detection and does not require specific configuration.
