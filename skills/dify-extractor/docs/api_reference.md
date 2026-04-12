# Dify Extractor - API Reference
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

## Classes

### DifyExtractorContext

Manages extraction context and document state.

```python
from dify_extractor import DifyExtractorContext

context = DifyExtractorContext()
```

**Attributes:**
- `current_document` (str): Current document being processed
- `extracted_records` (int): Number of records extracted
- `contacts_registry` (dict): Stored contact information
- `extraction_history` (list): History of all operations
- `metadata` (dict): Additional metadata

**Methods:**

#### `to_dict() -> Dict[str, Any]`
Export context as dictionary.

```python
context_dict = context.to_dict()
# Returns:
# {
#     'current_document': 'file.csv',
#     'extracted_records': 150,
#     'contacts_found': 25,
#     'last_operation': 'extract_csv',
#     'timestamp': '2024-01-10T10:30:00Z',
#     'metadata': {...}
# }
```

#### `add_extraction_history(operation: str, details: Dict[str, Any])`
Record extraction operation.

```python
context.add_extraction_history('extract_csv', {
    'file': 'data.csv',
    'rows': 150,
    'columns': ['name', 'email', 'phone']
})
```

---

### DocumentExtractor

Main document extraction handler.

```python
from dify_extractor import DocumentExtractor, DifyExtractorContext

context = DifyExtractorContext()
extractor = DocumentExtractor(context, verbose=True)
```

**Parameters:**
- `context` (DifyExtractorContext): Extraction context manager
- `verbose` (bool): Enable verbose logging (default: False)

**Methods:**

#### `extract_csv(input_path, output_format='json', preserve_contacts=True, filter_by=None, limit=None) -> Dict[str, Any]`

Extract data from CSV file.

**Parameters:**
- `input_path` (str): Path to CSV file
- `output_format` (str): 'json', 'dict', or 'markdown' (default: 'json')
- `preserve_contacts` (bool): Extract and preserve contacts (default: True)
- `filter_by` (str, optional): Filter column name
- `limit` (int, optional): Maximum rows to extract

**Returns:**
```python
{
    'status': 'success',
    'data': [...],  # Extracted data
    'context': {...}  # Updated context
}
```

**Example:**
```python
result = extractor.extract_csv(
    'customers.csv',
    output_format='json',
    preserve_contacts=True,
    limit=100
)

if result['status'] == 'success':
    print(f"Extracted {result['context']['extracted_records']} records")
```

#### `extract_excel(input_path, sheet_name=None, output_format='json', preserve_contacts=True, header_row=0) -> Dict[str, Any]`

Extract data from Excel file.

**Parameters:**
- `input_path` (str): Path to Excel file
- `sheet_name` (str, optional): Target sheet name
- `output_format` (str): 'json', 'dict', or 'markdown' (default: 'json')
- `preserve_contacts` (bool): Extract and preserve contacts (default: True)
- `header_row` (int): Header row index (default: 0)

**Returns:** Same as `extract_csv`

**Example:**
```python
result = extractor.extract_excel(
    'sales.xlsx',
    sheet_name='Q1 Data',
    output_format='markdown'
)
```

#### `extract_pdf(input_path, mode='text', preserve_contacts=True, page_range=None) -> Dict[str, Any]`

Extract text and structured data from PDF.

**Parameters:**
- `input_path` (str): Path to PDF file
- `mode` (str): 'text', 'table', or 'both' (default: 'text')
- `preserve_contacts` (bool): Extract and preserve contacts (default: True)
- `page_range` (str, optional): Pages to extract (e.g., '1-5')

**Returns:** Same as `extract_csv`

**Example:**
```python
result = extractor.extract_pdf(
    'report.pdf',
    mode='text',
    preserve_contacts=True
)
```

---

### ContactManager

Manages contact information storage and retrieval.

```python
from dify_extractor import ContactManager, DifyExtractorContext

context = DifyExtractorContext()
contacts = ContactManager(context, storage_path='.dify_contacts')
```

**Parameters:**
- `context` (DifyExtractorContext): Extraction context
- `storage_path` (str): Directory for contact storage (default: '.dify_contacts')

**Methods:**

#### `store_contact(contact_id: str, contact_data: Dict[str, Any]) -> Dict[str, Any]`

Store contact information.

**Parameters:**
- `contact_id` (str): Contact identifier (e.g., email address)
- `contact_data` (dict): Contact information

**Returns:**
```python
{
    'status': 'success',
    'message': 'Contact stored successfully',
    'contact_id': 'john@example.com'
}
```

**Example:**
```python
result = contacts.store_contact('john@example.com', {
    'name': 'John Doe',
    'phone': '555-1234',
    'company': 'ACME Corp',
    'department': 'Sales'
})
```

#### `retrieve_contact(contact_id: str) -> Dict[str, Any]`

Retrieve stored contact information.

**Parameters:**
- `contact_id` (str): Contact identifier

**Returns:**
```python
{
    'status': 'success',
    'data': {
        'name': 'John Doe',
        'phone': '555-1234',
        ...
    }
}
```

**Example:**
```python
result = contacts.retrieve_contact('john@example.com')
if result['status'] == 'success':
    contact = result['data']
    print(contact['name'])
```

#### `list_contacts(search_query: Optional[str] = None) -> Dict[str, Any]`

List all stored contacts with optional search.

**Parameters:**
- `search_query` (str, optional): Search query (searches all fields)

**Returns:**
```python
{
    'status': 'success',
    'count': 5,
    'contacts': [...]
}
```

**Example:**
```python
# List all contacts
result = contacts.list_contacts()

# Search for contacts
result = contacts.list_contacts(search_query='ACME')
```

---

## Dify Plugin Provider

### DifyExtractorProvider

Dify plugin provider for tool integration.

```python
from dify_extractor_provider import DifyExtractorProvider

provider = DifyExtractorProvider()
```

**Methods:**

#### `invoke(tool_name: str, parameters: dict) -> dict`

Invoke extraction tool.

**Parameters:**
- `tool_name` (str): Tool identifier
- `parameters` (dict): Tool parameters

**Available Tools:**
- `extract_csv`
- `extract_excel`
- `extract_pdf`
- `store_contact`
- `retrieve_contact`
- `list_contacts`
- `get_context`

**Example:**
```python
# Extract CSV via Dify plugin
result = provider.invoke('extract_csv', {
    'input': 'data.csv',
    'preserve_contacts': True,
    'output': 'json'
})

# Store contact
result = provider.invoke('store_contact', {
    'contact_id': 'john@example.com',
    'contact_data': {'name': 'John Doe', 'phone': '555-1234'}
})

# Get current context
result = provider.invoke('get_context', {})
```

#### `get_tool_schema() -> dict`

Get schema for all available tools.

```python
schema = provider.get_tool_schema()
for tool in schema['tools']:
    print(f"{tool['name']}: {tool['description']}")
```

---

## Command Line Interface

### Main Script

```bash
python dify_extractor.py [options]
```

**Options:**

#### File Extraction

```bash
# Extract CSV
python dify_extractor.py \
  --input file.csv \
  --output json \
  --preserve-contacts \
  --limit 100

# Extract Excel
python dify_extractor.py \
  --input file.xlsx \
  --sheet-name "Sheet1" \
  --output markdown \
  --preserve-contacts

# Extract PDF
python dify_extractor.py \
  --input file.pdf \
  --mode text \
  --preserve-contacts
```

#### Contact Management

```bash
# Store contact
python dify_extractor.py \
  --action store \
  --contact-id john@example.com \
  --contact-data '{"name": "John", "phone": "555-1234"}'

# Retrieve contact
python dify_extractor.py \
  --action retrieve \
  --contact-id john@example.com

# List contacts
python dify_extractor.py \
  --action list \
  --search-query "ACME"
```

**Global Options:**
- `-v, --verbose`: Verbose output
- `-o, --output-dir`: Output directory

---

## Response Format

### Success Response

```python
{
    'status': 'success',
    'data': {...},  # Extracted data or operation result
    'context': {
        'current_document': 'file.csv',
        'extracted_records': 150,
        'contacts_found': 25,
        'last_operation': 'extract_csv',
        'timestamp': '2024-01-10T10:30:00Z'
    }
}
```

### Error Response

```python
{
    'status': 'error',
    'error': 'Error message',
    'context': {...}
}
```

---

## Data Types

### Contact Data

```python
{
    'name': 'John Doe',
    'email': 'john@example.com',
    'phone': '555-1234',
    'company': 'ACME Corp',
    'department': 'Sales',
    'address': '123 Main St',
    # ... additional fields
}
```

### Extraction Result

```python
{
    'id': 1,
    'name': 'John Doe',
    'email': 'john@example.com',
    'phone': '555-1234',
    # ... record data
}
```

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `FileNotFoundError` | File path incorrect | Verify absolute or relative path |
| `JSONDecodeError` | Invalid JSON in contact data | Ensure valid JSON format |
| `KeyError` | Missing required parameter | Check required parameters |
| `UnicodeDecodeError` | File encoding issue | Try with `--encoding utf-8` |

### Exception Handling

```python
from dify_extractor import DocumentExtractor, DifyExtractorContext

context = DifyExtractorContext()
extractor = DocumentExtractor(context)

try:
    result = extractor.extract_csv('file.csv')
    if result['status'] == 'error':
        print(f"Extraction error: {result['error']}")
except Exception as e:
    print(f"Unexpected error: {e}")
    print(f"Context: {context.to_dict()}")
```

---

## Examples

See `/home/user/skills/huynguyen03dev/dify-extractor/examples/` for complete working examples.
