# Dify Extractor - Integration Guide
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

## Overview

This guide explains how to integrate the Dify Extractor skill with your Dify and Claude workflows.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│          Claude / Dify User Interface                │
└──────────────────┬──────────────────────────────────┘
                   │
        ┌──────────▼──────────┐
        │   Skill Router      │ (Claude context awareness)
        └──────────┬──────────┘
                   │
    ┌──────────────┴──────────────┐
    │                             │
┌───▼────────────────┐    ┌──────▼─────────────┐
│  Dify Plugin API   │    │  Direct Python API │
└───┬────────────────┘    └──────┬─────────────┘
    │                            │
    │     ┌──────────────────────┘
    │     │
┌───▼─────▼─────────────────────────┐
│  DifyExtractorProvider             │
│  ├─ DocumentExtractor             │
│  ├─ ContactManager                │
│  └─ DifyExtractorContext           │
└────────────────────────────────────┘
         │
    ┌────┴─────┬────────┬──────────┐
    │           │        │          │
┌───▼──┐  ┌────▼──┐ ┌───▼─┐  ┌────▼────┐
│ CSV  │  │ Excel │ │ PDF │  │ Contacts│
│Files │  │Files  │ │Files│  │Registry │
└──────┘  └───────┘ └─────┘  └─────────┘
```

## Integration Methods

### Method 1: Dify Plugin Integration

Use the plugin as a native Dify tool.

**Step 1: Copy plugin files**

```bash
cp -r /home/user/next-home/plugins/dify-extractor \
      /your-dify-installation/plugins/
```

**Step 2: Register in Dify**

In your Dify configuration, add:

```yaml
tools:
  - name: dify_extractor
    provider: DifyExtractorProvider
    icon: icon.svg
```

**Step 3: Use in workflows**

```python
# In Dify workflow
result = invoke_tool('dify_extractor', 'extract_csv', {
    'input': 'data.csv',
    'preserve_contacts': True,
    'output': 'json'
})
```

### Method 2: Claude Skill Integration

Use the skill within Claude conversations.

**Activate the skill:**

```
@dify-extractor extract-csv --input customers.csv --preserve-contacts
```

**Available commands:**

- `extract-csv` - Extract CSV data
- `extract-excel` - Extract Excel data
- `extract-pdf` - Extract PDF data
- `manage-contacts` - Manage contact information
- `get-context` - Show current context

### Method 3: Python API

Use directly in Python scripts.

```python
from dify_extractor import (
    DifyExtractorContext,
    DocumentExtractor,
    ContactManager
)

# Initialize
context = DifyExtractorContext()
extractor = DocumentExtractor(context)
contacts = ContactManager(context)

# Extract data
result = extractor.extract_csv('data.csv', preserve_contacts=True)

# Manage contacts
contacts.store_contact('john@example.com', {
    'name': 'John Doe',
    'phone': '555-1234'
})

# Get context
print(context.to_dict())
```

## Context Management

### Context Structure

```python
{
    'current_document': 'filename.csv',
    'extracted_records': 150,
    'contacts_found': 25,
    'last_operation': 'extract_csv',
    'timestamp': '2024-01-10T10:30:00Z',
    'metadata': {...}
}
```

### Preserving Context Across Operations

The skill automatically maintains context:

1. **Document Tracking** - Remembers the last processed document
2. **Contact Registry** - Persistent contact storage
3. **History** - Logs all extraction operations
4. **State** - Maintains current extraction state

```python
# Context is preserved across multiple operations
context = DifyExtractorContext()
extractor = DocumentExtractor(context)

# Operation 1
result1 = extractor.extract_csv('file1.csv')

# Operation 2 - context from operation 1 is preserved
result2 = extractor.extract_excel('file2.xlsx')

# Both operations' information is in context
print(context.extraction_history)  # Shows both operations
```

## Contact Management

### Storing Contacts

```bash
python dify_extractor.py \
  --action "store" \
  --contact-id "john@example.com" \
  --contact-data '{"name": "John Doe", "phone": "555-1234"}'
```

### Retrieving Contacts

```bash
python dify_extractor.py \
  --action "retrieve" \
  --contact-id "john@example.com"
```

### Searching Contacts

```bash
python dify_extractor.py \
  --action "list" \
  --search-query "ACME"
```

### Contact Auto-Extraction

When using `--preserve-contacts` flag, the skill automatically:
1. Identifies contact columns (email, phone, name, etc.)
2. Extracts contact information
3. Stores in the contact registry
4. Makes contacts searchable

## Workflow Examples

### Workflow 1: Customer Data Import

```python
# 1. Extract customer data
result = extractor.extract_csv('customers.csv', preserve_contacts=True)

# 2. Access extracted contacts
contacts = context.contacts_registry

# 3. Store important contacts
for contact_id, contact_data in contacts.items():
    if contact_data.get('column') == 'email':
        contact_manager.store_contact(
            contact_id,
            {'email': contact_data['value'], 'imported': True}
        )
```

### Workflow 2: Multi-Document Processing

```python
# Process batch of files
files = ['contacts.csv', 'leads.xlsx', 'newsletter.pdf']

context = DifyExtractorContext()
extractor = DocumentExtractor(context)

for file_path in files:
    result = extractor.extract_*(*file_path*, preserve_contacts=True)
    
    # Context automatically updated
    print(f"Processed: {context.current_document}")
    print(f"Records: {context.extracted_records}")
```

### Workflow 3: Contact Lookup and Update

```python
# Retrieve existing contact
contact = contact_manager.retrieve_contact('john@example.com')

if contact['status'] == 'success':
    # Update with new information
    contact_data = contact['data']
    contact_data['updated_department'] = 'Marketing'
    
    # Store updated contact
    contact_manager.store_contact('john@example.com', contact_data)
```

## Error Handling

### File Not Found

```python
try:
    result = extractor.extract_csv('missing.csv')
    if result['status'] == 'error':
        print(f"Error: {result['error']}")
except FileNotFoundError as e:
    print(f"File error: {e}")
```

### Invalid Data Format

```python
try:
    contact_data = json.loads(contact_json)
except json.JSONDecodeError:
    print("Invalid JSON format for contact data")
```

### Contact Not Found

```python
result = contact_manager.retrieve_contact('nonexistent@example.com')
if result['status'] == 'error':
    print(f"Contact not found: {result['error']}")
```

## Configuration

### Output Formats

Supported formats:
- `json` - JSON array of records
- `dict` - Python dictionary format
- `markdown` - Markdown table format

### Contact Storage Path

Default: `.dify_contacts/`

Customize:
```python
contact_manager = ContactManager(context, storage_path='/custom/path')
```

### Logging

Enable verbose logging:

```bash
python dify_extractor.py --input file.csv --verbose
```

## Performance Considerations

- **Large Files**: Use `--limit` parameter for CSV/Excel
- **Batch Processing**: Use `--batch-mode` for multiple files
- **Memory**: Context is kept in memory; clear history if needed

```python
# Clear extraction history if memory is concern
context.extraction_history = []
```

## Security

- File paths are validated before processing
- Contact data is stored locally in `.dify_contacts/`
- No external API calls are made
- Input validation on all parameters

## Troubleshooting

### No Contacts Found

Check that:
1. `--preserve-contacts` flag is enabled
2. File contains columns with contact-related names
3. Data is in recognized format (email, phone, etc.)

### Output Empty

Verify:
1. File path is correct
2. File format is supported
3. File contains data

### Context Not Preserved

Ensure:
1. Using same `context` object across operations
2. Not creating new context instances unexpectedly
3. Saving context if using across sessions

## Related Documentation

- [API Reference](./api_reference.md)
- [Main Skill Guide](../SKILL.md)
- [Dify Plugin Docs](https://dify.ai/)
- [Claude Skills Guide](https://claude.ai/help)
