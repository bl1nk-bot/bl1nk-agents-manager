## Purpose

The Edit Error Recovery agent handles edit tool failures, including:
- Common edit error pattern detection
- Immediate recovery guidance
- Session state restoration
- Error prevention mechanisms

## Components

- `index.rs`: Main entry point and hook registration

## Usage

This agent is automatically invoked when edit tool operations fail. It detects common error patterns and provides immediate action guidance to fix the issue.

## Configuration

The agent uses built-in error detection patterns and does not require specific configuration.
