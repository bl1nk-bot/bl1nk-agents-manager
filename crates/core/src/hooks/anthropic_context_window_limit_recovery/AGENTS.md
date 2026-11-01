## Purpose

The Anthropic Context Window Limit Recovery agent handles situations where Anthropic model requests exceed context window limits, providing:
- Automatic token limit error detection
- Intelligent content truncation strategies
- Recovery mechanisms to continue operations
- Preservation of important context information

## Components

- `executor.rs`: Core execution logic for recovery operations
- `index.rs`: Main entry point and hook registration
- `parser.rs`: Token limit error parsing and analysis
- `pruning_deduplication.rs`: Content deduplication and optimization
- `pruning_types.rs`: Type definitions for pruning operations
- `storage.rs`: Storage and retrieval of truncated content
- `types.rs`: Core type definitions and interfaces

## Usage

This agent is automatically invoked when Anthropic model requests encounter context window limit errors. It attempts to recover by intelligently truncating less important content while preserving critical context.

## Configuration

The agent uses built-in recovery strategies and does not require specific configuration. It operates based on detected token limit errors.
