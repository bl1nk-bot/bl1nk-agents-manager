## Purpose

The Compaction Context Injector agent maintains continuity during session compaction, including:
- Preserving important session context
- Injecting structured context for summarization
- Maintaining work continuity after compaction
- Providing context for seamless session continuation

## Components

- `index.rs`: Main entry point and hook registration

## Usage

This agent is invoked during session compaction to inject structured context that helps maintain work continuity. It provides guidance for summarization and preserves important session information.

## Configuration

The agent uses built-in context injection patterns and does not require specific configuration.
