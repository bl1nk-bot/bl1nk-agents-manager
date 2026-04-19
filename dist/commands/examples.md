---
name: blk:examples
description: Show example prompts and use cases for agents
version: 1.0.0
argument-hint: '[arguments]'
---

# Examples Command

First, read the system agent registry file using `run_shell_command` with `cat ${extensionPath}/agents/agents.json`. **Do not use `read_file`.**

Then, attempt to read the custom agent registry file using `run_shell_command` with `cat ${extensionPath}/custom/agents.json`.

Task: Provide example prompts and use cases for the agent '{{args}}' by searching both registries.
If no agent is specified in '{{args}}', pick 3 diverse agents (from both system and custom) to demonstrate.
Use the 'use_cases' field in the JSON as a base to generate specific, copy-pasteable example prompts.