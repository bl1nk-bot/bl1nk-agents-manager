#!/usr/bin/env python3
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "docs" / "PROJECT_MAP.md"

KEY_DIRS = [
    "crates/core/src",
    "crates/server/src",
    "agents",
    "skills",
    "commands",
    "config",
    "docs",
]

KEY_FILES = [
    "Cargo.toml",
    "crates/core/Cargo.toml",
    "crates/server/Cargo.toml",
    "config/Config.toml",
    "agents/agents.json",
    "skills/skills.json",
]


def list_dir_tree(path: Path, max_depth: int = 2):
    lines = []
    base_depth = len(path.parts)
    for root, dirs, files in os.walk(path):
        depth = len(Path(root).parts) - base_depth
        if depth > max_depth:
            dirs[:] = []
            continue
        indent = "  " * depth
        lines.append(f"{indent}- {Path(root).name}/")
        for f in sorted(files):
            lines.append(f"{indent}  - {f}")
    return lines


def main():
    lines = []
    lines.append("# Project Map (Auto-Generated)")
    lines.append("")
    lines.append("This file is generated. Run `python3 scripts/update_project_map.py` to refresh.")
    lines.append("")

    lines.append("## Key Paths")
    for d in KEY_DIRS:
        lines.append(f"- `{d}/`")
    lines.append("")

    lines.append("## Key Files")
    for f in KEY_FILES:
        lines.append(f"- `{f}`")
    lines.append("")

    for d in KEY_DIRS:
        path = ROOT / d
        if path.exists():
            lines.append(f"## Tree: `{d}/`")
            lines.extend(list_dir_tree(path))
            lines.append("")

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
