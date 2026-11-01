#!/usr/bin/env python3
import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def run(cmd, cwd=ROOT):
    print(f"\n==> {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        sys.exit(result.returncode)


def validate_agents():
    run(["python3", "scripts/validate_agents.py"])


def audit_skills():
    skill_root = ROOT / "skills"
    registry = skill_root / "skills.json"
    if not registry.exists():
        print("skills/skills.json not found")
        return

    data = json.loads(registry.read_text())
    registered = {entry["source"].rstrip("/") for entry in data.get("skills", [])}
    actual = {
        f"skills/{p.name}"
        for p in skill_root.iterdir()
        if p.is_dir() and (p / "SKILL.md").exists()
    }

    missing = sorted(actual - registered)
    extra = sorted(registered - actual)

    print("\nSkill registry audit")
    if missing:
        print("- Missing in skills.json:")
        for m in missing:
            print(f"  - {m}")
    else:
        print("- Missing in skills.json: none")

    if extra:
        print("- Entries with no folder:")
        for e in extra:
            print(f"  - {e}")
    else:
        print("- Entries with no folder: none")



def main():
    parser = argparse.ArgumentParser(description="Project workflow helper")
    sub = parser.add_subparsers(dest="cmd", required=True)

    sub.add_parser("fmt")
    sub.add_parser("check")
    sub.add_parser("clippy")
    sub.add_parser("agents")
    sub.add_parser("map")
    sub.add_parser("audit-skills")
    sub.add_parser("all")

    args = parser.parse_args()

    if args.cmd == "fmt":
        run(["cargo", "fmt"])
    elif args.cmd == "check":
        run(["cargo", "check"])
    elif args.cmd == "clippy":
        run(["cargo", "clippy", "--all-features"])
    elif args.cmd == "agents":
        validate_agents()
    elif args.cmd == "map":
        run(["python3", "scripts/update_project_map.py"])
    elif args.cmd == "audit-skills":
        audit_skills()
    elif args.cmd == "all":
        run(["python3", "scripts/update_project_map.py"])
        run(["cargo", "fmt"])
        run(["cargo", "clippy", "--all-features"])
        run(["cargo", "check"])
        validate_agents()
        audit_skills()


if __name__ == "__main__":
    main()
