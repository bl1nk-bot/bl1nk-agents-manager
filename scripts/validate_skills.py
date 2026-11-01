#!/usr/bin/env python3
"""
Comprehensive validation script for skills that accounts for the project status header
"""

import sys
import os
import re
import yaml
from pathlib import Path

def validate_skill(skill_path):
    """Comprehensive validation of a skill that handles the project status header"""
    skill_path = Path(skill_path)

    # Check SKILL.md exists
    skill_md = skill_path / 'SKILL.md'
    if not skill_md.exists():
        return False, "SKILL.md not found"

    # Read the file content
    content = skill_md.read_text()

    # Look for YAML frontmatter anywhere in the file (not just at the beginning)
    # This regex finds YAML frontmatter blocks (--- to ---)
    frontmatter_pattern = r'^---\n(.*?)\n---'
    match = re.search(frontmatter_pattern, content, re.DOTALL | re.MULTILINE)
    
    if not match:
        return False, "No YAML frontmatter found (look for --- ... --- block)"

    frontmatter_text = match.group(1)

    # Parse YAML frontmatter
    try:
        frontmatter = yaml.safe_load(frontmatter_text)
        if not isinstance(frontmatter, dict):
            return False, "Frontmatter must be a YAML dictionary"
    except yaml.YAMLError as e:
        return False, f"Invalid YAML in frontmatter: {e}"

    # Define allowed properties
    ALLOWED_PROPERTIES = {'name', 'description', 'license', 'allowed-tools', 'metadata'}

    # Check for unexpected properties (excluding nested keys under metadata)
    unexpected_keys = set(frontmatter.keys()) - ALLOWED_PROPERTIES
    if unexpected_keys:
        return False, (
            f"Unexpected key(s) in SKILL.md frontmatter: {', '.join(sorted(unexpected_keys))}. "
            f"Allowed properties are: {', '.join(sorted(ALLOWED_PROPERTIES))}"
        )

    # Check required fields
    if 'name' not in frontmatter:
        return False, "Missing 'name' in frontmatter"
    if 'description' not in frontmatter:
        return False, "Missing 'description' in frontmatter"

    # Extract name for validation
    name = frontmatter.get('name', '')
    if not isinstance(name, str):
        return False, f"Name must be a string, got {type(name).__name__}"
    name = name.strip()
    if name:
        # Check naming convention (hyphen-case: lowercase with hyphens)
        if not re.match(r'^[a-z0-9-]+$', name):
            return False, f"Name '{name}' should be hyphen-case (lowercase letters, digits, and hyphens only)"
        if name.startswith('-') or name.endswith('-') or '--' in name:
            return False, f"Name '{name}' cannot start/end with hyphen or contain consecutive hyphens"
        # Check name length (max 64 characters per spec)
        if len(name) > 64:
            return False, f"Name is too long ({len(name)} characters). Maximum is 64 characters."

    # Extract and validate description
    description = frontmatter.get('description', '')
    if not isinstance(description, str):
        return False, f"Description must be a string, got {type(description).__name__}"
    description = description.strip()
    if description:
        # Check for angle brackets
        if '<' in description or '>' in description:
            return False, "Description cannot contain angle brackets (< or >)"
        # Check description length (max 1024 characters per spec)
        if len(description) > 1024:
            return False, f"Description is too long ({len(description)} characters). Maximum is 1024 characters."

    return True, "Skill is valid!"

def validate_all_skills(base_path):
    """Validate all skills in the skills directory"""
    base_path = Path(base_path)
    skill_dirs = [d for d in base_path.iterdir() if d.is_dir()]
    
    results = []
    for skill_dir in skill_dirs:
        print(f"Validating {skill_dir.name}...")
        valid, message = validate_skill(skill_dir)
        results.append((skill_dir.name, valid, message))
        if valid:
            print(f"  ✅ {message}")
        else:
            print(f"  ❌ {message}")
        print()
    
    return results

if __name__ == "__main__":
    if len(sys.argv) == 1:
        # Validate all skills in the skills directory
        base_path = Path(__file__).parent.parent / "skills"
        print(f"Validating all skills in {base_path}")
        print("=" * 50)
        results = validate_all_skills(base_path)
        
        # Summary
        total = len(results)
        valid_count = sum(1 for _, valid, _ in results if valid)
        invalid_count = total - valid_count
        
        print("=" * 50)
        print(f"Summary: {valid_count}/{total} skills are valid")
        if invalid_count > 0:
            print(f"{invalid_count} skills have issues that need to be fixed")
    elif len(sys.argv) == 2:
        # Validate a specific skill
        skill_path = sys.argv[1]
        valid, message = validate_skill(skill_path)
        print(message)
        sys.exit(0 if valid else 1)
    else:
        print("Usage: python validate_skills.py [skill_directory]")
        print("       python validate_skills.py  # Validates all skills")
        sys.exit(1)