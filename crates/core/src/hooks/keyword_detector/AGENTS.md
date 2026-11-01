## Purpose

The Keyword Detector agent handles keyword detection, including:
- Pattern matching for specific keywords
- Regular expression-based detection
- Keyword-based action triggering
- Context-aware keyword recognition

## Components

- `constants.rs`: Constant definitions for detection patterns
- `detector.rs`: Core detection logic
- `mod.rs`: Module definitions and exports
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when keyword detection is needed. It scans input for predefined patterns and triggers appropriate actions based on detected keywords.

## Configuration

The agent uses built-in keyword patterns and does not require specific configuration.
