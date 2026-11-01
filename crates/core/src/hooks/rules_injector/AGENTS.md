## Purpose

The Rules Injector agent handles rules injection, including:
- Rule matching and selection
- Rule injection into sessions
- Constraint enforcement
- Session-specific rule application

## Components

- `constants.rs`: Constant definitions for rule operations
- `finder.rs`: Rule discovery and matching logic
- `index.rs`: Main entry point and hook registration
- `matcher.rs`: Rule matching algorithms
- `mod.rs`: Module definitions and exports
- `parser.rs`: Rule parsing and validation
- `storage.rs`: Rule storage and retrieval
- `types.rs`: Type definitions and interfaces

## Usage

This agent is invoked when rules need to be injected into a session. It matches applicable rules based on session context and injects them appropriately.

## Configuration

The agent uses built-in rule matching patterns and does not require specific configuration.
