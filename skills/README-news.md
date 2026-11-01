## Directory Structure

### Key Files
- `skills.json` - Central registry of all skills with standardized tags for cross-model compatibility
- `README.md` - This documentation file

### Skill Categories

#### Development Skills
- `agent-development/` - Creating autonomous agents for Claude Code plugins
- `command-development/` - Developing custom commands for the system
- `skill-development/` - Creating and managing new skills
- `extension-architect/` - Designing and building extensions
- `plugin-development/` - Developing plugins for enhanced functionality

#### Engineering Skills
- `action-architect/` - Expert AI assistant for creating GPT Actions and API integrations
- `code-implementer/` - Implements technical plans with rigorous verification
- `code-researcher/` - Conducts technical research on codebase tasks and documentation
- `implementation-planner/` - Creates detailed, atomic, and safe implementation plans
- `ruthless-refactorer/` - Senior Principal Engineering refactoring to eliminate technical debt

#### Creative Skills
- `algorithmic-art/` - Creating algorithmic art and visualizations
- `canvas-design/` - Designing canvas-based visualizations
- `content-research-writer/` - Research and content creation specialist

#### Utility Skills
- `artifacts-builder/` - Building and managing code artifacts
- `auto-dupicate-fixer/` - Automatically detecting and fixing duplicate code
- `brand-guidelines/` - Maintaining brand consistency across outputs
- `changelog-generator/` - Automated changelog generation
- `docs-writer/` - Documentation creation and maintenance
- `ticket-manager/` - Managing Linear tickets locally using Markdown files

#### Analysis & Review Skills
- `code-reviewer/` - Systematic multi-perspective code review with consistent quality gates
- `code-simplifier/` - Simplifying and refining code for clarity and maintainability
- `database-analyst/` - Analyzes database schemas, migrations, and ORM entities
- `research-reviewer/` - Reviews technical research for objectivity and completeness

#### Workflow & Management Skills
- `ai-workflow-orchestrator/` - Orchestrating complex multi-step AI workflows
- `sequential-thinking/` - Structured thinking approach for complex problems
- `multi-format-orchestrator/` - Handling multiple data formats in workflows

#### Specialized Skills
- `building-ai-agent-on-cloudflare/` - Building AI agents on Cloudflare Workers
- `chrome-devtools/` - Chrome DevTools integration and automation
- `mcp-builder/` - Building Model Context Protocol integrations
- `mcp-integration/` - MCP integration and management

## Skills Registry (skills.json)

The `skills.json` file contains a comprehensive registry of all skills with standardized tagging for cross-model compatibility. It includes:

- **Tag Schema**: Standardized tags for domain, complexity, category, input/output types, and dependencies
- **Skill Definitions**: Detailed information about each skill including use cases, version, and related skills
- **Agent Mappings**: Relationships between agents and skills for unified system operation
- **Command Mappings**: Standardized tags for command agents

### Tag Schema
- **Domain**: Primary domain classification (engineering, utility, creative, command, management, etc.)
- **Complexity**: Task complexity level (low, medium, high, expert)
- **Category**: Functional category (code-generation, code-review, architecture, documentation, etc.)
- **Input/Output Types**: Types of data the skill handles (text, code, file, json, yaml, etc.)
- **Dependencies**: External dependencies required by the skill
- **Personality**: Agent personality/tone characteristics

## Usage

Skills can be invoked individually or combined to create complex workflows. Each skill directory contains specific documentation and implementation files that define its capabilities and usage patterns.

## Integration

Skills are integrated with the agent system through the mappings defined in `skills.json`, allowing for seamless orchestration between different capabilities.
