## Purpose

The Directory Readme Injector agent handles directory-based README injection, including:
- Directory context analysis
- README content injection based on directory patterns
- Context-aware content selection
- Directory-specific README configuration

## Components

- `constants.rs`: Constant definitions for injection operations
- `index.rs`: Main entry point and hook registration
- `storage.rs`: Injection state storage and retrieval
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when directory context is detected. It analyzes the current directory and injects appropriate README content based on directory patterns and project structure.

## Configuration

The agent uses built-in directory matching patterns and does not require specific configuration.
