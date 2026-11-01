## Purpose

The Prometheus MD Only agent enforces file operation restrictions for the Prometheus agent, including:
- Blocking non-Markdown file operations
- Enforcing .sisyphus directory restrictions
- Providing planning workflow guidance
- Preventing unauthorized file modifications

## Components

- `constants.rs`: Constant definitions for file operation restrictions
- `index.rs`: Main entry point and hook registration

## Usage

This agent is automatically invoked when the Prometheus agent attempts file operations. It validates file paths and extensions, allowing only .md files within the .sisyphus directory.

## Configuration

The agent uses built-in file restriction rules and does not require specific configuration.
