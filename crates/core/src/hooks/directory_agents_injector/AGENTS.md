## Purpose

The Directory Agents Injector agent handles directory-based agent injection, including:
- Directory context analysis
- Agent injection based on directory patterns
- Context-aware agent selection
- Directory-specific agent configuration

## Components

- `constants.rs`: Constant definitions for injection operations
- `index.rs`: Main entry point and hook registration
- `storage.rs`: Injection state storage and retrieval
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when directory context is detected. It analyzes the current directory and injects appropriate agents based on directory patterns and project structure.

## Configuration

The agent uses built-in directory matching patterns and does not require specific configuration.
