import os
import yaml
import json
import glob
import sys
from jsonschema import validate, ValidationError

# --- Configuration (v1.7.2 Standard) ---
SCHEMA_CAPABILITY = "config/v1.7/capability-schema.json"
SCHEMA_POLICY = "config/v1.7/policy-schema.json"
REGISTRY_FILE = "agents/agents.json"

def load_schema(path):
    with open(path, 'r', encoding='utf-8') as f:
        return json.load(f)

def validate_md_file(path, schema):
    """Validate a single .md file frontmatter against a schema."""
    with open(path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    if not content.startswith('---'):
        return [f"Missing YAML frontmatter in {path}"]
    
    try:
        parts = content.split('---')
        data = yaml.safe_load(parts[1])
        validate(instance=data, schema=schema)
        return []
    except ValidationError as e:
        return [f"Schema error in {path}: {e.message}"]
    except Exception as e:
        return [f"Validation error in {path}: {e}"]

def main():
    print("🔍 Validating BL1NK Workforce Infrastructure (v1.7.2)...")
    all_errors = []
    
    # 1. Load Schemas
    try:
        cap_schema = load_schema(SCHEMA_CAPABILITY)
        pol_schema = load_schema(SCHEMA_POLICY)
    except Exception as e:
        print(f"❌ Failed to load schemas: {e}")
        sys.exit(1)

    # 2. Validate Registry (agents.json)
    print(f"  - Checking registry: {REGISTRY_FILE}")
    if os.path.exists(REGISTRY_FILE):
        try:
            with open(REGISTRY_FILE, 'r', encoding='utf-8') as f:
                reg_data = json.load(f)
            validate(instance=reg_data, schema=pol_schema)
        except ValidationError as e:
            all_errors.append(f"Registry Policy Violation: {e.message}")
        except Exception as e:
            all_errors.append(f"Registry Read Error: {e}")

    # 3. Validate Agents Frontmatter
    print("  - Checking agent capabilities (.md)...")
    for f in glob.glob("agents/*.md"):
        if os.path.basename(f).upper() in ['README.MD', 'TODO.MD']: continue
        errs = validate_md_file(f, cap_schema)
        if errs: all_errors.extend(errs)

    # 4. Validate Skills Frontmatter
    print("  - Checking skill capabilities (SKILL.md)...")
    if os.path.exists('skills'):
        for skill_dir in os.listdir('skills'):
            skill_file = os.path.join('skills', skill_dir, 'SKILL.md')
            if os.path.exists(skill_file):
                errs = validate_md_file(skill_file, cap_schema)
                if errs: all_errors.extend(errs)

    # Final Result
    if all_errors:
        print("\n🚩 Infrastructure Validation Failed:")
        for err in all_errors: print(f"  ❌ {err}")
        sys.exit(1)
    else:
        print("\n✨ All systems compliant with v1.7.2 Policy & Capability standards!")
        sys.exit(0)

if __name__ == "__main__":
    main()
