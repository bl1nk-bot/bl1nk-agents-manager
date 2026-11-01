## Purpose

The Atlas Agent handles orchestration operations, including:
- Multi-agent workflow coordination
- Task delegation management
- Session state orchestration
- Complex operation sequencing

## Components

- `index.rs`: Main entry point and hook registration
- `types.rs`: Type definitions and interfaces

## Usage

This agent is invoked when Atlas orchestrator operations are performed. It coordinates complex workflows involving multiple agents and manages the overall session state during orchestration.

## Configuration

The agent uses built-in orchestration patterns and does not require specific configuration.
