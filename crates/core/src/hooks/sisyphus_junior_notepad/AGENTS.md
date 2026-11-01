## Purpose

The Sisyphus Junior Notepad agent helps maintain work records and plan integrity, including:
- Notepad file location guidance (.sisyphus/notepads/)
- Learning, issue, decision, and problem recording
- Plan file protection (read-only enforcement)
- Work context maintenance

## Components

- `constants.rs`: Constant definitions for notepad operations
- `index.rs`: Main entry point and hook registration

## Usage

This agent is invoked when the Sisyphus Junior agent performs delegate_task operations. It injects notepad guidance to help maintain proper work records and protect plan files from modification.

## Configuration

The agent uses built-in notepad guidance patterns and does not require specific configuration.
