## Purpose

The Task Resume Info agent handles task resumption, including:
- Session ID extraction and tracking
- Resume instruction generation
- Task state preservation
- Continuation guidance

## Components

- `index.rs`: Main entry point and hook registration
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when task resumption is needed. It extracts session IDs from tool outputs and provides guidance for continuing tasks.

## Configuration

The agent uses built-in session tracking and does not require specific configuration.
