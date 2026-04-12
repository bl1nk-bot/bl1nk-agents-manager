#!/usr/bin/env python3
"""
Generate Claude SKILL.md from Dify Plugin
Creates skill documentation that bridges Dify plugins with Claude's skill system
"""

import argparse
import json
import sys
from pathlib import Path
import yaml
import logging

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


class SkillGenerator:
    """Generate Claude SKILL.md from Dify plugin"""
    
    def __init__(self, plugin_path: str, output_dir: str = None):
        self.plugin_path = Path(plugin_path)
        self.output_dir = Path(output_dir) if output_dir else self.plugin_path.parent
        self.manifest = None
        self.plugin_name = self.plugin_path.name
    
    def generate(self) -> bool:
        """Generate SKILL.md"""
        try:
            logger.info(f"Generating SKILL.md for {self.plugin_name}")
            
            # Load manifest
            if not self._load_manifest():
                return False
            
            # Generate skill file
            skill_content = self._generate_skill_content()
            
            # Write to file
            skill_file = self.output_dir / 'SKILL.md'
            with open(skill_file, 'w') as f:
                f.write(skill_content)
            
            logger.info(f"✓ Generated SKILL.md at {skill_file}")
            return True
        
        except Exception as e:
            logger.error(f"Error generating SKILL.md: {str(e)}")
            return False
    
    def _load_manifest(self) -> bool:
        """Load manifest.yaml"""
        manifest_file = self.plugin_path / 'manifest.yaml'
        
        if not manifest_file.exists():
            logger.error(f"manifest.yaml not found at {manifest_file}")
            return False
        
        try:
            with open(manifest_file, 'r') as f:
                self.manifest = yaml.safe_load(f)
            return True
        except Exception as e:
            logger.error(f"Error loading manifest: {str(e)}")
            return False
    
    def _generate_skill_content(self) -> str:
        """Generate SKILL.md content"""
        identity = self.manifest.get('identity', {})
        tools = self.manifest.get('features', [])
        
        skill_name = identity.get('name', self.plugin_name)
        description = identity.get('description', {}).get('en_US', 'A Dify plugin')
        version = identity.get('version', '0.1.0')
        author = identity.get('author', 'Unknown')
        
        # Generate tool documentation
        tools_doc = self._generate_tools_documentation(tools)
        
        # Generate examples
        examples_doc = self._generate_examples(tools)
        
        skill_md = f'''---
name: {skill_name}
description: {description} (Dify Plugin Integration)
version: {version}
author: {author}
---

# {self._humanize(skill_name)}

A Claude-integrated Dify plugin for {description.lower()}.

**Version:** {version}  
**Author:** {author}  
**Plugin Type:** Dify Plugin  
**Integration:** Claude Skill System

## Overview

This skill provides {description.lower()} capabilities with seamless Dify plugin integration. It maintains context across operations and supports batch processing workflows.

### Architecture

This plugin bridges Dify and Claude ecosystems:

```
Claude Conversation
        │
        ├─ @{skill_name} <command> [options]
        │
        ▼
    Dify Plugin API
        │
        ├─ Context Manager (persistent state)
        │
        ▼
    Tool Invocation Engine
        │
        ├─ Parameter Validation
        ├─ Tool Execution
        └─ State Preservation
```

## Available Tools

{tools_doc}

## Quick Start

```bash
@{skill_name} <tool-name> [parameters]
```

## Tool Reference

{self._generate_tool_reference(tools)}

## Context Management

The plugin automatically manages execution context:

- **State Preservation**: Maintains state between operations
- **Error Handling**: Comprehensive error recovery
- **History Tracking**: Logs all operations
- **Metadata**: Preserves operation metadata

### Accessing Context

Context is automatically available within the plugin:

```python
context = PluginContext()
context.set_current_operation('tool_name')
context.add_metadata('key', 'value')
context.save_state()  # Auto-persisted
```

## Common Workflows

{examples_doc}

## Integration Guide

### Using in Claude

Simply invoke with the skill name:

```
@{skill_name} <tool> --param1 value1 --param2 value2
```

### Using in Dify

Add to Dify configuration:

```yaml
tools:
  - name: {skill_name}
    provider: {self._camel_case(skill_name)}Provider
```

### Using in Python

```python
from src.provider import {self._camel_case(skill_name)}Provider

provider = {self._camel_case(skill_name)}Provider()
result = provider.invoke('<tool-name>', {{'param': 'value'}})
```

## Requirements

- Python 3.8+
- dify-plugin library
- Additional dependencies listed in requirements.txt

## Installation

```bash
pip install -r requirements.txt
```

## File Structure

```
{self.plugin_name}/
├── manifest.yaml          # Plugin metadata
├── README.md             # Plugin documentation
├── requirements.txt      # Dependencies
├── src/
│   ├── provider.py       # Main plugin provider
│   ├── context_manager.py # Context handling
│   └── tools/            # Tool implementations
├── tests/                # Test suite
└── docs/                 # Extended documentation
```

## Features

- ✅ Dify plugin ecosystem integration
- ✅ Context preservation across operations
- ✅ Automatic error handling
- ✅ Type-safe parameters
- ✅ Comprehensive logging
- ✅ Production-ready structure

## Error Handling

All operations include comprehensive error handling:

```python
result = provider.invoke('tool_name', parameters)

if result['status'] == 'error':
    error_msg = result['error']
    # Handle error
else:
    data = result['data']
    # Process data
```

## Troubleshooting

**Issue**: Tool not found
- **Solution**: Verify tool name and check manifest.yaml

**Issue**: Parameter validation failed
- **Solution**: Check tool schema and parameter types

**Issue**: Context not persisting
- **Solution**: Ensure PluginContext is initialized with persistent=True

## Advanced Topics

### Custom Tool Implementation

To add new tools:

1. Create tool file in `src/tools/`
2. Implement tool function
3. Add to provider.py
4. Update manifest.yaml

### Performance Optimization

- Use caching for repeated operations
- Batch process large datasets
- Profile tool execution

### Testing

```bash
pytest tests/
```

## Support & Documentation

- **Plugin Repository**: {self.plugin_path}
- **Dify Docs**: https://dify.ai/
- **Claude Skills**: https://claude.ai/help

## License

See LICENSE file in plugin directory.

## Related Skills

- See other plugins in the Dify ecosystem
- Check Claude skill marketplace

## Version History

### v{version}

- Initial release
- Core functionality implemented
- Integration with Claude skill system
'''
        
        return skill_md
    
    def _generate_tools_documentation(self, tools: list) -> str:
        """Generate tools documentation"""
        if not tools:
            return "No tools available."
        
        docs = ""
        for i, tool in enumerate(tools, 1):
            tool_name = self._humanize(tool)
            docs += f"### {i}. {tool_name}\n\n"
            docs += f"Command: `@{self.plugin_name} {tool} [options]`\n\n"
        
        return docs
    
    def _generate_tool_reference(self, tools: list) -> str:
        """Generate detailed tool reference"""
        if not tools:
            return "No tools available."
        
        reference = ""
        for tool in tools:
            reference += f"""## {self._humanize(tool)}

**Command:** `{tool}`

**Description:** TODO: Add description

**Parameters:**

- No parameters defined yet (update tool YAML schema)

**Returns:**

- `status`: Operation status (success/error)
- `data`: Tool output data

**Example:**

```bash
@{self.plugin_name} {tool}
```

---

"""
        
        return reference
    
    def _generate_examples(self, tools: list) -> str:
        """Generate usage examples"""
        if not tools:
            return "No examples available."
        
        examples = ""
        
        if tools:
            first_tool = tools[0]
            examples += f"""### Workflow 1: Basic {self._humanize(first_tool)}

```bash
@{self.plugin_name} {first_tool} --param1 value1
```

"""
        
        if len(tools) > 1:
            second_tool = tools[1]
            examples += f"""### Workflow 2: Multi-step Processing

```bash
# Step 1
@{self.plugin_name} {first_tool} --param1 value1

# Step 2
@{self.plugin_name} {second_tool} --param2 value2
```

"""
        
        examples += """### Workflow 3: Batch Processing

Use the plugin multiple times in sequence to process batches of data.

"""
        
        return examples
    
    @staticmethod
    def _humanize(name: str) -> str:
        """Convert snake_case to Title Case"""
        return ' '.join(word.capitalize() for word in name.split('_'))
    
    @staticmethod
    def _camel_case(name: str) -> str:
        """Convert snake_case to CamelCase"""
        return ''.join(word.capitalize() for word in name.split('_'))


def main():
    parser = argparse.ArgumentParser(description='Generate Claude SKILL.md from Dify Plugin')
    parser.add_argument('--plugin-path', required=True, help='Path to Dify plugin directory')
    parser.add_argument('--output-dir', help='Output directory for SKILL.md')
    
    args = parser.parse_args()
    
    generator = SkillGenerator(args.plugin_path, args.output_dir)
    
    if generator.generate():
        sys.exit(0)
    else:
        sys.exit(1)


if __name__ == '__main__':
    main()
