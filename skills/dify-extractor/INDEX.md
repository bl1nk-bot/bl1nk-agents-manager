## Quick Navigation

### 🚀 Getting Started
- **New to this skill?** Start here: [`SKILL.md`](SKILL.md)
- **Want to install?** Follow: [`SETUP.md`](SETUP.md)
- **Need examples?** Check: [`examples/example_workflow.py`](examples/example_workflow.py)

### 📚 Documentation
- [`SKILL.md`](SKILL.md) - Main feature overview and workflows
- [`SETUP.md`](SETUP.md) - Installation and configuration
- [`docs/integration_guide.md`](docs/integration_guide.md) - Integration patterns
- [`docs/api_reference.md`](docs/api_reference.md) - Complete API documentation

### 💻 Code
- [`scripts/dify_extractor.py`](scripts/dify_extractor.py) - Main CLI script (380 lines)
- [`src/providers/dify_extractor.py`](src/providers/dify_extractor.py) - Dify plugin provider (250 lines)
- [`examples/example_workflow.py`](examples/example_workflow.py) - Working examples

### ⚙️ Configuration
- [`requirements.txt`](requirements.txt) - Python dependencies

---

## What This Skill Does

### Extracts Data From
- 📄 CSV files
- 📊 Excel spreadsheets (.xlsx, .xls)
- 📑 PDF documents

### Manages Contacts
- 📧 Extracts email addresses
- 📱 Extracts phone numbers
- 👤 Extracts names and company info
- 💾 Stores and retrieves contacts

### Integrates With
- **Dify Plugin System** - Native tool integration
- **Claude Skills** - Context-aware workflow
- **Python API** - Direct programmatic access

---

## Quick Start Commands

### 1. Install
```bash
pip install -r /home/user/skills/huynguyen03dev/dify-extractor/requirements.txt
```

### 2. Extract CSV
```bash
python /home/user/skills/huynguyen03dev/dify-extractor/scripts/dify_extractor.py \
  --input data.csv \
  --preserve-contacts \
  --output json
```

### 3. Manage Contacts
```bash
# Store
python scripts/dify_extractor.py \
  --action store \
  --contact-id john@example.com \
  --contact-data '{"name": "John", "phone": "555-1234"}'

# List
python scripts/dify_extractor.py --action list

# Search
python scripts/dify_extractor.py --action list --search-query "ACME"
```

### 4. Test
```bash
python /home/user/skills/huynguyen03dev/dify-extractor/examples/example_workflow.py
```

---

## Documentation Overview

### `SKILL.md` (269 lines)
**Main documentation** covering:
- Purpose and features
- Available commands
- Global options
- Context management
- Dify integration
- File structure
- Requirements
- Common workflows
- Troubleshooting

### `SETUP.md` (Installation guide)
**Setup instructions** covering:
- Prerequisites
- Installation steps
- Directory structure
- Configuration
- Testing
- Integration setup
- Troubleshooting
- Verification

### `docs/integration_guide.md` (351 lines)
**Integration patterns** covering:
- Architecture diagram
- Integration methods (3 ways)
- Context management
- Contact management
- Workflow examples
- Error handling
- Configuration
- Performance tips
- Security notes
- Troubleshooting

### `docs/api_reference.md` (460 lines)
**Complete API documentation** covering:
- All classes (DifyExtractorContext, DocumentExtractor, ContactManager)
- All methods with parameters and returns
- Dify plugin provider API
- CLI interface
- Response formats
- Data types
- Error handling
- Examples

---

## File Structure

```
dify-extractor/
├── INDEX.md                      ← You are here
├── SKILL.md                      (Main guide - START HERE)
├── SETUP.md                      (Installation guide)
├── requirements.txt              (Dependencies)
│
├── scripts/
│   └── dify_extractor.py        (CLI & main classes - 380 lines)
│
├── src/
│   └── providers/
│       └── dify_extractor.py    (Dify plugin provider - 250 lines)
│
├── examples/
│   └── example_workflow.py      (Working examples)
│
└── docs/
    ├── integration_guide.md     (Integration patterns - 351 lines)
    └── api_reference.md         (API docs - 460 lines)
```

---

## Key Concepts

### Context Management
- Tracks current document
- Records extraction history
- Stores contact information
- Preserves state across operations

### Document Extraction
- Reads CSV, Excel, PDF files
- Extracts structured data
- Preserves formatting
- Supports multiple output formats

### Contact Preservation
- Auto-detects contact columns
- Extracts emails and phones
- Stores in registry
- Enables search functionality

---

## Integration Methods

### 1. Dify Plugin (Native)
```python
provider = DifyExtractorProvider()
result = provider.invoke('extract_csv', {...})
```
→ See: [`docs/integration_guide.md`](docs/integration_guide.md#method-1-dify-plugin-integration)

### 2. Claude Skill
```
@dify-extractor extract-csv --input data.csv
```
→ See: [`docs/integration_guide.md`](docs/integration_guide.md#method-2-claude-skill-integration)

### 3. Python API
```python
context = DifyExtractorContext()
extractor = DocumentExtractor(context)
result = extractor.extract_csv('data.csv')
```
→ See: [`docs/integration_guide.md`](docs/integration_guide.md#method-3-python-api)

---

## Common Tasks

### Extract and Preserve Contacts
See: [`SKILL.md` - Common Workflows](SKILL.md#common-workflows)

### Store and Retrieve Contacts
See: [`docs/integration_guide.md` - Contact Management](docs/integration_guide.md#contact-management)

### Process Multiple Files
See: [`docs/integration_guide.md` - Workflow 2](docs/integration_guide.md#workflow-2-multi-document-processing)

### Handle Errors
See: [`docs/integration_guide.md` - Error Handling](docs/integration_guide.md#error-handling)

---

## Architecture

```
┌─────────────────────────────┐
│   User Interface            │
│ (Claude Skill / Dify Plugin)│
└────────────┬────────────────┘
             │
    ┌────────▼─────────┐
    │ DifyExtractorProvider
    │ (Plugin interface)
    └────────┬─────────┘
             │
    ┌────────▼──────────────────┐
    │ DifyExtractorContext       │
    │ (State management)         │
    ├────────────────────────────┤
    │ ├─ DocumentExtractor       │
    │ │  ├─ extract_csv()        │
    │ │  ├─ extract_excel()      │
    │ │  └─ extract_pdf()        │
    │ │                          │
    │ └─ ContactManager          │
    │    ├─ store_contact()      │
    │    ├─ retrieve_contact()   │
    │    └─ list_contacts()      │
    └────────┬──────────────────┘
             │
    ┌────────▼──────────┐
    │ File Systems      │
    ├──────────────────┤
    │ CSV, Excel, PDF  │
    │ Contact Registry │
    └───────────────────┘
```

---

## Next Steps

1. **Choose your path:**
   - Installing? → [`SETUP.md`](SETUP.md)
   - Learning? → [`SKILL.md`](SKILL.md)
   - Integrating? → [`docs/integration_guide.md`](docs/integration_guide.md)
   - Coding? → [`docs/api_reference.md`](docs/api_reference.md)

2. **Get started:**
   - Install dependencies
   - Run examples
   - Try extraction
   - Build integration

3. **Need help?**
   - Check troubleshooting sections
   - Review examples
   - Read detailed documentation
   - Check inline code comments

---

## Support

- 📖 Full documentation in each .md file
- 💻 Working code examples in `examples/`
- ⚙️ Configuration in `SETUP.md`
- 🔧 API details in `docs/api_reference.md`
- 🎯 Integration guides in `docs/integration_guide.md`

---

## File Sizes

| File | Lines | Purpose |
|------|-------|---------|
| `scripts/dify_extractor.py` | 380 | Main implementation |
| `docs/api_reference.md` | 460 | Complete API docs |
| `docs/integration_guide.md` | 351 | Integration patterns |
| `SKILL.md` | 269 | Feature overview |
| `src/providers/dify_extractor.py` | 250 | Dify plugin |

**Total Documentation**: 1,710 lines  
**Total Code**: 630 lines

---

Last updated: December 30, 2024  
Status: ✅ Ready to use
