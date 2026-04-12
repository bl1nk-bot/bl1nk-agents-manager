## 📌 Project Status (Feb 7, 2026)

Bl1nk Agents Manager is in active development and is not feature‑complete yet.
This repo contains a working extension shell and a Rust core that is being
brought to feature parity with existing TypeScript logic.

**What works now**
- Extension manifest and Gemini CLI scaffolding are present.
- Core Rust modules exist for agents, hooks, MCP/ACP, sessions, and RPC.
- Command and documentation sets are present (currently being refreshed).

**In progress**
- TypeScript → Rust parity for large subsystems (background agents, config,
  ACP normalization).
- End‑to‑end session flows for Gemini/Codex/Qwen within a unified adapter.
- Validation of hook behavior and task orchestration across agents.

**Known gaps**
- Some Rust modules compile but are not fully wired end‑to‑end.
- Configuration loading/migration is still being aligned to actual runtime.
- Authentication flows for some CLIs still require manual steps.

**What to expect right now**
- You can explore the architecture, commands, and agent catalogs.
- Some workflows will still require manual setup or troubleshooting.

For a complete non‑developer overview, see `docs/PROJECT_STATUS.md`.

---
name: dify-plugin-creator
description: Advanced Dify Plugin Generator Framework. Creates production-ready Dify plugins with automated scaffolding, context management, and Claude integration. Extends skill-creator pattern to support Dify plugin ecosystem.
---

# Dify Plugin Creator Framework

An advanced framework for creating production-ready Dify plugins with seamless Claude skill integration. This framework transforms the skill-creator pattern to generate standardized, context-aware Dify plugins.

## Purpose

- Generate production-ready Dify plugin structure
- Automate scaffolding for document processors, API wrappers, and data extractors
- Integrate with Claude skill system for context preservation
- Support batch plugin generation
- Enable plugin packaging and distribution

## Architecture

```
┌─────────────────────────────────────────────────────┐
│       Dify Plugin Creator Framework                 │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────────────────────────────────────┐  │
│  │ Plugin Generator Engine                      │  │
│  │ ├─ init_dify_plugin.py (Scaffolding)         │  │
│  │ ├─ manifest_generator.py (Config builder)    │  │
│  │ └─ package_plugin.py (Distribution)          │  │
│  └──────────────────────────────────────────────┘  │
│                                                     │
│  ┌──────────────────────────────────────────────┐  │
│  │ Template System                              │  │
│  │ ├─ manifest.yaml.template                    │  │
│  │ ├─ tool.yaml.template                        │  │
│  │ ├─ provider.py.template                      │  │
│  │ └─ context_manager.py.template               │  │
│  └──────────────────────────────────────────────┘  │
│                                                     │
│  ┌──────────────────────────────────────────────┐  │
│  │ Context Management System                    │  │
│  │ ├─ Context Preservation                      │  │
│  │ ├─ State Management                          │  │
│  │ └─ Claude Integration                        │  │
│  └──────────────────────────────────────────────┘  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Quick Start

### Create a new Dify plugin

```bash
python /home/user/skills/huynguyen03dev/dify-plugin-creator/scripts/init_dify_plugin.py \
  --name "my-data-processor" \
  --author "your-name" \
  --type "processor" \
  --category "data-extraction"
```

### Generate specific tool

```bash
python /home/user/skills/huynguyen03dev/dify-plugin-creator/scripts/generate_tool.py \
  --plugin-path "./my-data-processor" \
  --tool-name "extract_data" \
  --tool-type "extractor" \
  --input-format "csv,excel,json"
```

### Package for distribution

```bash
python /home/user/skills/huynguyen03dev/dify-plugin-creator/scripts/package_plugin.py \
  --plugin-path "./my-data-processor" \
  --output "./plugins" \
  --include-tests
```

## Plugin Types Supported

### 1. **Document Processor**
Process documents (CSV, Excel, PDF, JSON)
```bash
--type "document_processor"
--input-formats "csv,excel,pdf"
--features "extraction,validation,transformation"
```

### 2. **API Wrapper**
Wrap external APIs with Dify interface
```bash
--type "api_wrapper"
--api-endpoint "https://api.example.com"
--auth-type "api_key|oauth|basic"
```

### 3. **Data Transformer**
Transform and enrich data
```bash
--type "data_transformer"
--transformation-type "mapping,aggregation,enrichment"
```

### 4. **Contact Manager**
Manage contact information and relationships
```bash
--type "contact_manager"
--storage-backend "file|database"
--search-enabled true
```

### 5. **RAG Component**
Retrieval-Augmented Generation integration
```bash
--type "rag_component"
--retrieval-method "semantic,keyword,hybrid"
--embedding-model "text-embedding-3-small"
```

## Generated Plugin Structure

```
my-data-processor/
├── manifest.yaml                    # Plugin metadata & configuration
├── README.md                        # Plugin documentation
├── requirements.txt                 # Python dependencies
├── src/
│   ├── __init__.py
│   ├── provider.py                  # Main plugin provider
│   ├── context_manager.py           # Context handling
│   ├── tools/
│   │   ├── __init__.py
│   │   ├── extractor.py             # Tool implementation
│   │   └── schemas.py               # Tool schemas
│   └── models/
│       ├── __init__.py
│       ├── data_models.py           # Data models
│       └── config.py                # Configuration models
├── tests/
│   ├── __init__.py
│   ├── test_provider.py
│   └── test_tools.py
├── scripts/
│   ├── init.sh                      # Initialization script
│   └── install.sh                   # Installation script
└── docs/
    ├── API.md                       # API reference
    ├── integration.md               # Integration guide
    └── examples.md                  # Usage examples
```

## Key Features

### 1. **Progressive Context Loading**

Implements Progressive Disclosure pattern to prevent context bloat:

```python
# Level 1: Metadata only
metadata = {
    'name': 'my-processor',
    'version': '1.0.0',
    'capabilities': [...]
}

# Level 2: Tool schemas (loaded on demand)
schemas = plugin.get_tool_schemas()

# Level 3: Full context (when needed)
context = plugin.get_full_context()
```

### 2. **Automatic Schema Generation**

```bash
python scripts/generate_tool.py \
  --plugin-path "./my-plugin" \
  --analyze-source true  # Auto-generate schema from Python type hints
```

### 3. **Context Manager Integration**

```python
from src.context_manager import PluginContext

context = PluginContext()
context.set_current_operation('extract_data')
context.add_metadata('source_file', 'data.csv')
context.preserve_state()  # Auto-saves to disk
```

### 4. **Claude Skill Auto-Generation**

Automatically creates Claude-compatible SKILL.md:

```bash
python scripts/generate_skill.py \
  --plugin-path "./my-plugin" \
  --output-dir "/home/user/skills/huynguyen03dev"
```

## Integration with Dify

### Step 1: Create Plugin

```bash
python scripts/init_dify_plugin.py \
  --name "my-processor" \
  --author "me" \
  --type "document_processor"
```

### Step 2: Implement Tools

Edit `src/provider.py` to implement your tools:

```python
class MyProcessorProvider(ToolProvider):
    def invoke(self, tool_name: str, parameters: dict) -> dict:
        if tool_name == 'extract_data':
            return self.extract_data(parameters)
```

### Step 3: Generate SKILL.md

```bash
python scripts/generate_skill.py --plugin-path "./my-processor"
```

### Step 4: Register in Dify

Copy to Dify plugins directory:

```bash
cp -r my-processor /path/to/dify/plugins/
```

## Template System

### manifest.yaml Template

```yaml
identity:
  author: {author}
  name: {plugin_name}
  label:
    en_US: {display_name}
  description:
    en_US: {description}
  icon: icon.svg
  version: {version}

tools:
  - tools/{tool_name}.yaml

features:
  - {feature1}
  - {feature2}

extra:
  python:
    source: src/provider.py
```

### Tool YAML Template

```yaml
identity:
  name: {tool_name}
  label: {tool_label}
  description: {tool_description}
  icon: icon.svg

parameters:
  - name: {param_name}
    type: {param_type}
    required: true
    description: {param_description}

output:
  type: object
  properties:
    result:
      type: string
```

## Generated Plugin Capabilities

Each generated plugin includes:

- **Built-in Context Manager**: Automatic state preservation
- **Error Handling**: Comprehensive error recovery
- **Logging**: Structured logging throughout
- **Testing Framework**: Unit test templates
- **Documentation**: Auto-generated docs
- **Type Safety**: Full type hints
- **Async Support**: Async/await patterns

## Usage Examples

### Example 1: Create CSV Processor Plugin

```bash
python scripts/init_dify_plugin.py \
  --name "csv-processor" \
  --author "john-doe" \
  --type "document_processor" \
  --input-formats "csv" \
  --features "parse,validate,transform"
```

### Example 2: Create API Wrapper Plugin

```bash
python scripts/init_dify_plugin.py \
  --name "weather-api" \
  --author "john-doe" \
  --type "api_wrapper" \
  --api-endpoint "https://api.openweathermap.org" \
  --auth-type "api_key"
```

### Example 3: Create Contact Manager Plugin

```bash
python scripts/init_dify_plugin.py \
  --name "contact-manager" \
  --author "john-doe" \
  --type "contact_manager" \
  --storage-backend "file" \
  --search-enabled true
```

## Script Reference

### init_dify_plugin.py

Initialize new plugin with full scaffolding.

**Parameters:**
- `--name`: Plugin name (required)
- `--author`: Author name (required)
- `--type`: Plugin type (required)
- `--category`: Plugin category
- `--version`: Initial version (default: 0.1.0)
- `--output-dir`: Output directory (default: current)

### generate_tool.py

Generate new tool within existing plugin.

**Parameters:**
- `--plugin-path`: Path to plugin directory (required)
- `--tool-name`: Tool name (required)
- `--tool-type`: Tool type (required)
- `--input-format`: Input formats (comma-separated)
- `--output-format`: Output format
- `--analyze-source`: Auto-analyze function signatures

### package_plugin.py

Package plugin for distribution.

**Parameters:**
- `--plugin-path`: Path to plugin directory (required)
- `--output`: Output directory (default: current)
- `--include-tests`: Include test files
- `--include-docs`: Include documentation
- `--create-archive`: Create .skill archive

### generate_skill.py

Generate Claude SKILL.md from plugin.

**Parameters:**
- `--plugin-path`: Path to plugin directory (required)
- `--output-dir`: Output directory for SKILL.md
- `--include-examples`: Include usage examples
- `--include-context`: Include context management docs

## Advanced Features

### Context Preservation Across Sessions

```python
context = PluginContext(persistent=True)
context.load_state()  # Load from disk
# ... perform operations ...
context.save_state()  # Save to disk
```

### Tool Schema Validation

```bash
python scripts/validate_plugin.py --plugin-path "./my-plugin"
```

### Performance Profiling

```bash
python scripts/profile_plugin.py --plugin-path "./my-plugin"
```

### Plugin Testing

```bash
python scripts/test_plugin.py --plugin-path "./my-plugin"
```

## Best Practices

1. **Progressive Disclosure**: Don't load all context at once
2. **Type Safety**: Use type hints for all functions
3. **Error Handling**: Handle and log all errors
4. **Documentation**: Keep docs in sync with code
5. **Testing**: Write tests for all tools
6. **Versioning**: Follow semantic versioning
7. **Dependencies**: Minimize external dependencies

## Troubleshooting

**Issue**: Plugin not recognized by Dify
- **Solution**: Verify manifest.yaml format and location

**Issue**: Tool parameters not appearing
- **Solution**: Check tool YAML schema definitions

**Issue**: Context not persisting
- **Solution**: Ensure PluginContext is initialized with `persistent=True`

## Related Documentation

- Main Skill Documentation
- Dify Plugin Developer Guide
- Claude Skills Integration Guide

## Requirements

- Python 3.8+
- dify-plugin library
- Jinja2 (for template rendering)
- pyyaml (for YAML parsing)

## Installation

```bash
pip install -r /home/user/skills/huynguyen03dev/dify-plugin-creator/requirements.txt
```
