# SKILL.md Template Files Documentation

## Overview
The `SKILL.md` files are standardized template files located in each skill subdirectory within the skills/ directory. These files serve as structured guides for defining and implementing specific skills within the Bl1nk Agents Manager system.

## Purpose and Function

### Primary Role
- Define the structure and behavior of individual skills
- Provide standardized YAML frontmatter for skill identification and triggering
- Contain detailed instructions for implementing and using each skill
- Serve as templates for creating new skills following established patterns

### Structure
Each `SKILL.md` file follows a consistent format:

```
[Project Status Header]
---
YAML frontmatter (name, description, optional fields)
---
# Skill Title
[Detailed skill instructions and documentation]
```

### YAML Frontmatter
The YAML frontmatter is critical for skill identification and triggering:

- **name** (required): Unique identifier for the skill
- **description** (required): Comprehensive description that helps Claude determine when to use the skill
- **optional fields**: Additional metadata like license, version, etc.

Example:
```yaml
---
name: code-reviewer
description: Use this skill to review code. It supports both local changes (staged or working tree) and remote Pull Requests (by ID or URL). It focuses on correctness, maintainability, and adherence to project standards.
---
```

## Directory Organization

Each skill resides in its own subdirectory under `skills/`:

```
skills/
├── skill-name/
│   ├── SKILL.md (required template file)
│   ├── README.md (optional documentation)
│   ├── scripts/ (optional executable resources)
│   ├── references/ (optional reference materials)
│   └── assets/ (optional output resources)
```

## Content Guidelines

### Frontmatter Requirements
- Name must be unique and descriptive
- Description must clearly explain when and how to use the skill
- Description serves as the primary triggering mechanism

### Body Content
- Provide essential procedural knowledge
- Include workflow instructions
- Reference bundled resources (scripts, references, assets)
- Keep under 500 lines for efficiency

## Progressive Disclosure Design

Skills use a three-level loading system to manage context efficiently:

1. **Metadata** (name + description) - Always in context (~100 words)
2. **SKILL.md body** - When skill triggers (<5k words)
3. **Bundled resources** - As needed by Claude (Unlimited)

## Bundled Resources

Skills may include optional resource directories:

### Scripts (`scripts/`)
- Executable code for deterministic tasks
- Used when operations are fragile or consistency is critical

### References (`references/`)
- Documentation loaded as needed
- Database schemas, API docs, domain knowledge

### Assets (`assets/`)
- Files used in output (templates, logos, etc.)
- Not intended to be loaded into context

## Common Patterns

### Skill Creation Process
1. Understand the skill with concrete examples
2. Plan reusable skill contents (scripts, references, assets)
3. Initialize the skill (run init_skill.py)
4. Edit the skill (implement resources and write SKILL.md)
5. Package the skill (run package_skill.py)
6. Iterate based on real usage

### Progressive Disclosure Patterns
- Keep SKILL.md body to essentials
- Split content into separate reference files when approaching limits
- Organize by domain or framework when supporting multiple variants
- Link to advanced content conditionally

## Best Practices

### Conciseness
- The context window is a shared resource
- Only add context Claude doesn't already have
- Prefer concise examples over verbose explanations

### Appropriate Freedom Levels
- **High freedom**: Text-based instructions for flexible tasks
- **Medium freedom**: Pseudocode or scripts with parameters
- **Low freedom**: Specific scripts with few parameters for fragile operations

### Proper Triggering Information
- Include specific contexts for when to use the skill in the description
- Don't put "when to use" information in the body (it's loaded after triggering)

## Relationship to skills.json

The `SKILL.md` files complement the central `skills.json` registry by providing detailed implementation instructions, while `skills.json` provides standardized metadata and cross-references between agents, skills, and commands.

## Maintenance

- Regularly update SKILL.md files to reflect changes in skill functionality
- Ensure bundled resources remain synchronized with instructions
- Test skills after modifications to verify proper functioning
