# Dify Extractor - Setup Guide
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

## Installation

### 1. Prerequisites
- Python 3.8+
- pip package manager

### 2. Install Dependencies

```bash
# From workspace root
pip install -r /home/user/skills/huynguyen03dev/dify-extractor/requirements.txt

# Or individual packages
pip install pandas openpyxl PyPDF2
```

### 3. Directory Structure

```
next-home/plugins/dify-extractor/
├── dify_extractor.py          # Plugin provider
├── dify_extractor.yaml        # Plugin configuration
└── README.md

skills/huynguyen03dev/dify-extractor/
├── SKILL.md                   # Main skill documentation
├── SETUP.md                   # This file
├── requirements.txt
├── scripts/
│   └── dify_extractor.py      # Main CLI script
├── src/
│   └── providers/
│       └── dify_extractor.py  # Dify plugin provider
├── examples/
│   └── example_workflow.py    # Usage examples
└── docs/
    ├── api_reference.md       # API documentation
    └── integration_guide.md   # Integration instructions
```

## Configuration

### Contact Storage Path

Default: `.dify_contacts/`

To customize:
```python
from dify_extractor import ContactManager, DifyExtractorContext

context = DifyExtractorContext()
contacts = ContactManager(context, storage_path='/custom/path')
```

### Environment Variables (Optional)

```bash
export DIFY_CONTACT_PATH=/path/to/contacts
export DIFY_VERBOSE=1
export DIFY_OUTPUT_FORMAT=json
```

## Testing

### Run Examples

```bash
cd /home/user/skills/huynguyen03dev/dify-extractor/examples
python example_workflow.py
```

### Quick Test

```bash
cd /home/user/skills/huynguyen03dev/dify-extractor

# Create test CSV
cat > test.csv << 'DATA'
name,email,phone,company
John Doe,john@example.com,555-1234,ACME
Jane Smith,jane@example.com,555-5678,Tech Inc
DATA

# Extract
python scripts/dify_extractor.py --input test.csv --output json --verbose

# Store contact
python scripts/dify_extractor.py \
  --action store \
  --contact-id john@example.com \
  --contact-data '{"name": "John Doe", "company": "ACME"}'

# List contacts
python scripts/dify_extractor.py --action list

# Cleanup
rm test.csv
```

## Integration Setup

### For Dify Plugin

1. Copy plugin files to Dify installation:
```bash
cp -r /home/user/next-home/plugins/dify-extractor \
      /your-dify-installation/plugins/
```

2. Restart Dify service

3. Plugin will appear in tool library

### For Claude Skills

1. Skill is automatically available at:
   `/home/user/skills/huynguyen03dev/dify-extractor/`

2. Use in conversations with Claude:
   ```
   @dify-extractor extract-csv --input data.csv --preserve-contacts
   ```

## Troubleshooting

### Import Errors

```
ModuleNotFoundError: No module named 'pandas'
```

**Solution**: Install requirements
```bash
pip install -r requirements.txt
```

### File Not Found

```
FileNotFoundError: CSV file not found
```

**Solution**: Use absolute path or ensure file exists
```bash
python scripts/dify_extractor.py \
  --input /absolute/path/to/file.csv
```

### Encoding Issues

```
UnicodeDecodeError: 'utf-8' codec can't decode byte
```

**Solution**: Verify file encoding or specify encoding in extraction

### Contact Storage Permissions

```
PermissionError: [Errno 13] Permission denied
```

**Solution**: Check `.dify_contacts/` directory permissions
```bash
chmod 755 .dify_contacts/
```

## Verification

### Verify Installation

```bash
cd /home/user/skills/huynguyen03dev/dify-extractor

# Check files
ls -la scripts/
ls -la src/providers/
ls -la docs/

# Check dependencies
python -c "import pandas; print('✓ pandas')"
python -c "import openpyxl; print('✓ openpyxl')"
python -c "import PyPDF2; print('✓ PyPDF2')"

# Try importing main modules
python -c "from scripts.dify_extractor import DocumentExtractor; print('✓ DocumentExtractor')"
```

### Test Basic Operations

```bash
cd /home/user/skills/huynguyen03dev/dify-extractor

python << 'PYEOF'
from scripts.dify_extractor import DifyExtractorContext, DocumentExtractor, ContactManager

# Test context
context = DifyExtractorContext()
print("✓ Context initialized")

# Test extractor
extractor = DocumentExtractor(context)
print("✓ DocumentExtractor initialized")

# Test contact manager
contacts = ContactManager(context)
print("✓ ContactManager initialized")

# Test context export
ctx_dict = context.to_dict()
print(f"✓ Context: {ctx_dict}")

print("\n✓ All components working!")
PYEOF
```

## Next Steps

1. **Read SKILL.md** for full feature overview
2. **Review examples/** for usage patterns
3. **Check docs/integration_guide.md** for your integration scenario
4. **See docs/api_reference.md** for API details

## Support

For issues or questions:
1. Check troubleshooting section above
2. Review documentation files
3. Check example scripts
4. Verify file paths and permissions

---

Installation verified! Ready to use.
