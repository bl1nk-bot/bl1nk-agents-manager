## Purpose

The Thinking Block Validator agent handles thinking block validation, including:
- Thinking block structure validation
- Format compliance checking
- Proper placement verification
- Content validation within thinking blocks

## Components

- `index.rs`: Main entry point and hook registration
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when thinking blocks are processed. It validates that thinking blocks follow the required structure and format according to project standards.

## Configuration

The agent uses built-in validation patterns and does not require specific configuration.
