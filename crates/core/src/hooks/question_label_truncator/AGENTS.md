## Purpose

The Question Label Truncator agent handles long question labels, including:
- Automatic label truncation to specified lengths
- Preserving label meaning during truncation
- Maintaining UI consistency with truncated labels
- Preventing UI overflow issues

## Components

- `index.rs`: Main entry point and hook registration
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when question labels exceed the maximum allowed length. It truncates labels while preserving their essential meaning and adds ellipsis indicators.

## Configuration

The agent uses built-in truncation parameters and does not require specific configuration.
