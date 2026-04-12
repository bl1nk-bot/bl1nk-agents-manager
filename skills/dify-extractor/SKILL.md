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
name: dify-extractor
description: Dify Plugin Extractor with Document Context Management. Extracts structured data from documents (CSV, Excel, PDF) and manages contact information with Dify plugin architecture.
---

# Dify Extractor Skill

A comprehensive data extraction and document management skill designed for Dify plugin ecosystem integration with Claude's context management capabilities.

## Purpose

- Extract structured data from multiple document formats (CSV, Excel, PDF)
- Manage and preserve contact information
- Integrate seamlessly with Dify plugin architecture
- Maintain conversation context and document state
- Support batch processing and multi-document workflows

## Quick Start

Run the main extraction script:

```bash
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py <document_path> [options]
```

## Available Commands

### extract-csv

Extract structured data from CSV files.

```bash
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py \
  --input "path/to/file.csv" \
  --output "json" \
  --preserve-contacts
```

**Parameters:**
- `--input`: CSV file path (required)
- `--output`: Output format: `json` | `dict` | `markdown`
- `--preserve-contacts`: Keep contact information (boolean)
- `--filter-by`: Column name to filter data
- `--limit`: Maximum rows to extract

### extract-excel

Extract structured data from Excel files with sheet selection.

```bash
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py \
  --input "path/to/file.xlsx" \
  --sheet-name "Sheet1" \
  --output "json" \
  --preserve-contacts
```

**Parameters:**
- `--input`: Excel file path (required)
- `--sheet-name`: Target sheet (default: first sheet)
- `--output`: Output format: `json` | `dict` | `markdown`
- `--preserve-contacts`: Keep contact information
- `--header-row`: Header row index (default: 0)

### extract-pdf

Extract text and structured data from PDF files.

```bash
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py \
  --input "path/to/file.pdf" \
  --mode "text|table" \
  --preserve-contacts
```

**Parameters:**
- `--input`: PDF file path (required)
- `--mode`: Extraction mode: `text` | `table` | `both`
- `--preserve-contacts`: Keep contact information
- `--page-range`: Pages to extract (e.g., "1-5")

### manage-contacts

Store, retrieve, and manage extracted contact information.

```bash
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py \
  --action "store|retrieve|update|delete" \
  --contact-id "email@example.com" \
  --contact-data "{name: 'John', phone: '123456'}"
```

**Parameters:**
- `--action`: Contact action: `store` | `retrieve` | `update` | `delete` | `list`
- `--contact-id`: Contact identifier (email or custom ID)
- `--contact-data`: Contact information (JSON format)
- `--search-query`: Search contacts by name or email

## Global Options

- `-v, --verbose`: Enable verbose output
- `-o, --output-dir`: Output directory (default: current directory)
- `--format`: Output format: `json` | `markdown` | `dict`
- `--context-preserve`: Maintain document context in memory
- `--batch-mode`: Process multiple files sequentially

## Context Management

The skill maintains document context across operations:

- **Document State**: Tracks current document, extraction progress, and metadata
- **Contact Registry**: Persistent storage of extracted contact information
- **Extraction History**: Maintains log of all extraction operations
- **Conversation Context**: Preserves extraction context within Claude conversation

### Example Context Usage

```python
# Context is automatically managed through Dify plugin system
# Access current context in extraction operations
context = {
    'current_document': 'file.csv',
    'extracted_records': 150,
    'contacts_found': 25,
    'last_operation': 'extract_csv',
    'timestamp': '2024-01-10T10:30:00Z'
}
```

## Integration with Dify Plugin

The skill is designed as a Dify plugin provider:

```yaml
identity:
  author: huynguyen03dev
  name: dify_extractor
  label:
    en_US: Dify Extractor
  description:
    en_US: Extract and manage document data with contact preservation
  tags:
    - rag
    - data-extraction
    - contact-management
```

## File Structure

```
dify-extractor/
├── SKILL.md                    # This file
├── scripts/
│   ├── dify_extractor.py      # Main extraction script
│   ├── contact_manager.py     # Contact management utilities
│   └── document_parser.py     # Format-specific parsers
├── src/
│   ├── providers/
│   │   └── dify_extractor.py  # Dify plugin provider
│   ├── extractors/
│   │   ├── csv_extractor.py
│   │   ├── excel_extractor.py
│   │   └── pdf_extractor.py
│   └── models/
│       ├── contact.py
│       └── document.py
├── examples/
│   ├── sample.csv
│   ├── sample.xlsx
│   └── example_workflow.py
└── docs/
    ├── api_reference.md
    └── integration_guide.md
```

## Requirements

- Python 3.8+
- pandas (CSV/Excel extraction)
- openpyxl (Excel support)
- PyPDF2 (PDF support)
- dify-plugin (Dify integration)

## Installation

```bash
pip install -r /home/user/skills/huynguyen03dev/dify-extractor/requirements.txt
```

## Common Workflows

### 1. Extract Customer Data from CSV

```bash
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py \
  --input "customers.csv" \
  --preserve-contacts \
  --output "json"
```

### 2. Process Excel Reports with Contact Preservation

```bash
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py \
  --input "sales_report.xlsx" \
  --sheet-name "Contacts" \
  --preserve-contacts \
  --output "markdown"
```

### 3. Extract and Store Contacts

```bash
# Extract from document
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py \
  --input "document.pdf" \
  --mode "text"

# Store contacts
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py \
  --action "store" \
  --contact-id "john@example.com" \
  --contact-data '{"name": "John Doe", "phone": "555-1234", "company": "ACME"}'

# Retrieve stored contacts
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py \
  --action "list" \
  --search-query "ACME"
```

## Dify Integration

To use this skill as a Dify plugin:

1. Copy plugin files to next-home/plugins/dify-extractor/
2. Register in Dify tool configuration
3. Call via Dify tool interface:

```python
from dify_plugin import ToolProvider

class DifyExtractorProvider(ToolProvider):
    def invoke(self, tool_name: str, parameters: dict) -> dict:
        # Skill execution through Dify plugin interface
        return extract_and_manage_documents(parameters)
```

## Claude Skill Features

- **Context Awareness**: Maintains extraction state across conversations
- **Document Memory**: Remembers previously processed documents
- **Contact Preservation**: Extracts and stores contact information
- **Batch Processing**: Handle multiple documents in sequence
- **Error Handling**: Graceful degradation and error reporting

## Troubleshooting

**Issue**: File not found
- **Solution**: Use absolute path or ensure file is in correct directory

**Issue**: Encoding errors on CSV/Excel
- **Solution**: Specify encoding with `--encoding utf-8`

**Issue**: Contact not stored
- **Solution**: Verify contact format and ensure preservation flag is enabled

## Examples

See `/home/user/skills/huynguyen03dev/dify-extractor/examples/` for complete working examples.
