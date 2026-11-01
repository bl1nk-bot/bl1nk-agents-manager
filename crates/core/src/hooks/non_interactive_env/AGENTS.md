## Purpose

The Non Interactive Environment agent handles non-interactive operations, including:
- Environment detection (CI/CD, headless, etc.)
- Non-interactive command execution
- Environment-specific configuration
- Silent operation modes

## Components

- `index.rs`: Main entry point and hook registration
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when the system detects a non-interactive environment. It adjusts operations to work appropriately in CI/CD pipelines, headless environments, or other non-interactive contexts.

## Configuration

The agent automatically detects environment characteristics and does not require specific configuration.
