## Purpose

The Background Notification agent handles background notifications, including:
- Background task status updates
- Notification message formatting
- Task completion alerts
- Progress tracking notifications

## Components

- `index.rs`: Main entry point and hook registration
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when background tasks complete or reach significant milestones. It formats and delivers notifications about background task status to users.

## Configuration

The agent uses built-in notification patterns and does not require specific configuration.
