## Purpose

The Session Recovery agent handles session recovery, including:
- Session error detection and handling
- Recovery state management
- Session state restoration
- Error prevention mechanisms

## Components

- `constants.rs`: Constant definitions for recovery operations
- `index.rs`: Main entry point and hook registration
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when session errors occur. It manages recovery operations and helps restore session state to a working condition.

## Configuration

The agent uses built-in recovery patterns and does not require specific configuration.
