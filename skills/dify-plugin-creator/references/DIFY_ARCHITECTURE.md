## Overview

This document explains how the Dify Plugin Creator framework integrates with Dify and Claude ecosystems.

## Key Concepts

### 1. Plugin Types

The framework supports 5 main plugin types:

```
Document Processor  ──┐
                      │
API Wrapper         ──┤───> Plugin Provider ──> Tool Invocation
                      │
Data Transformer    ──┤
                      │
Contact Manager     ──┤
                      │
RAG Component       ──┘
```

### 2. Plugin Lifecycle

```
1. CREATION (init_dify_plugin.py)
   ├─ Create directory structure
   ├─ Generate manifest.yaml
   ├─ Generate provider.py
   └─ Generate context manager

2. DEVELOPMENT
   ├─ Implement tools in src/tools/
   ├─ Update tool schemas
   └─ Add business logic

3. INTEGRATION
   ├─ Generate Claude SKILL.md (generate_skill.py)
   ├─ Register in Dify (manifest.yaml)
   └─ Deploy to production

4. DEPLOYMENT
   ├─ Package plugin (package_plugin.py)
   ├─ Distribute .skill archive
   └─ Install in Dify instance
```

### 3. Context Management Pattern

```
Operation Start
      │
      ▼
Load Context ──> Execute Tool ──> Save State
      │                │               │
      └─ Preserve ─────┴────────────────┘
         Metadata
```

## Progressive Disclosure Pattern

The framework implements Progressive Disclosure to manage context efficiently:

**Level 1: Minimal (Metadata)**
```yaml
name: my-plugin
version: 0.1.0
type: document_processor
```

**Level 2: Schema (Tool Definitions)**
```yaml
tools:
  - tools/extract.yaml
  - tools/transform.yaml
```

**Level 3: Full (Complete Context)**
```python
# Full provider with all context loaded
context = PluginContext(persistent=True)
context.load_state()
```

## Generated Plugin Structure

```
my-plugin/
├── manifest.yaml              # [Level 1] Metadata
├── tools/
│   ├── extract.yaml          # [Level 2] Schema
│   ├── transform.yaml
│   └── validate.yaml
├── src/
│   ├── provider.py           # [Level 3] Full impl
│   ├── context_manager.py
│   └── tools/
│       ├── extract.py        # [Level 3] Logic
│       ├── transform.py
│       └── validate.py
└── docs/
    ├── API.md
    └── INTEGRATION.md
```

## Integration Points

### Dify Integration

```python
# Dify configuration
tools:
  - name: my-plugin
    provider: MyPluginProvider
    enabled: true
    context_preservation: true
```

### Claude Integration

```bash
# In Claude conversation
@my-plugin extract --file "data.csv"

# This triggers:
# 1. Dify Plugin API
# 2. Tool Invocation
# 3. Context Management
# 4. Response formatting
```

### Direct Python Integration

```python
from src.provider import MyPluginProvider

provider = MyPluginProvider()
result = provider.invoke('extract', {'file': 'data.csv'})
```

## Tool Schema Mapping

### Dify Tool Definition (YAML)

```yaml
identity:
  name: extract
  description: Extract data

parameters:
  - name: file
    type: string
    required: true
```

### Generated Tool Implementation (Python)

```python
def extract(self, parameters):
    file = parameters.get('file')
    # Process file
    return {'status': 'success', 'data': result}
```

### Claude Skill Command

```bash
@my-plugin extract --file "data.csv"
```

## Context Preservation

### How It Works

1. **Operation Starts**: Tool invocation begins
2. **Context Loaded**: Previous state loaded from disk
3. **Tool Executes**: With full context available
4. **State Updated**: New metadata and state added
5. **State Saved**: Persisted to disk

### Implementation

```python
class PluginContext:
    def __init__(self, persistent=True):
        self.persistent = persistent
        if persistent:
            self.load_state()
    
    def save_state(self):
        # Persists to disk
        pass
    
    def load_state(self):
        # Loads from disk
        pass
```

## Error Handling Flow

```
Tool Invocation
      │
      ├─ Validate Parameters
      │  └─ Error → Return Error Response
      │
      ├─ Execute Tool
      │  └─ Error → Log & Recover
      │
      ├─ Validate Output
      │  └─ Error → Return Error Response
      │
      └─ Save State
         └─ Error → Log (don't fail operation)
```

## Performance Considerations

### Context Size

- **Metadata Only**: ~1KB (Level 1)
- **With Schemas**: ~5KB (Level 2)
- **Full Context**: ~50KB (Level 3)

### Progressive Loading

```python
# Fast - loads only metadata
provider = DifyExtractorProvider()

# On demand - loads schemas
schemas = provider.get_tool_schemas()

# Full context - loads everything
context = provider.context.to_dict()
```

## Security Considerations

1. **Input Validation**: All parameters validated
2. **Type Safety**: Full type hints throughout
3. **Error Isolation**: Errors don't break other tools
4. **State Isolation**: Each plugin instance isolated
5. **Credential Management**: Through Dify credential system

## Best Practices

1. **Use Type Hints**
   ```python
   def extract(self, parameters: Dict[str, Any]) -> Dict[str, Any]:
       pass
   ```

2. **Comprehensive Logging**
   ```python
   logger.info(f"Processing {file_name}")
   logger.error(f"Failed: {error_msg}")
   ```

3. **Graceful Error Handling**
   ```python
   try:
       result = process()
   except Exception as e:
       logger.error(str(e))
       return {'status': 'error', 'error': str(e)}
   ```

4. **Context Management**
   ```python
   self.context.set_current_operation(tool_name)
   self.context.add_metadata('key', value)
   self.context.save_state()
   ```

5. **Documentation**
   - Keep docstrings updated
   - Document parameters and return values
   - Provide usage examples

## Troubleshooting

### Plugin not recognized

Check manifest.yaml:
- Valid YAML syntax
- Required fields present
- Correct provider path

### Tool not available

Check in manifest.yaml:
- Tool YAML file listed
- Tool file exists at specified path
- Tool name matches

### Context not persisting

Ensure:
- PluginContext initialized with `persistent=True`
- Storage directory writable
- Sufficient disk space

### Parameter validation fails

Check:
- Parameter names match schema
- Parameter types correct
- Required parameters provided

## Advanced Topics

### Custom Tool Types

Create new tool type:
1. Add to PLUGIN_TYPES in init_dify_plugin.py
2. Create template files
3. Update documentation

### Plugin Distribution

```bash
python package_plugin.py \
  --plugin-path "./my-plugin" \
  --output "./dist" \
  --create-archive
```

### Plugin Testing

```bash
pytest tests/
```

### Performance Optimization

- Cache tool results
- Batch process operations
- Profile with context size monitoring

## Related Documentation

- [Main SKILL.md](../SKILL.md)
- [Integration Guide](./INTEGRATION_GUIDE.md)
- [Dify Official Docs](https://dify.ai/)
- [Claude Skills Guide](https://claude.ai/)
