#!/usr/bin/env python3

from __future__ import annotations

import sys
from pathlib import Path


def fail(message: str) -> None:
    print(f"ERROR: {message}", file=sys.stderr)
    raise SystemExit(1)


def validate_skill(skill_dir: Path) -> None:
    skill_md = skill_dir / "SKILL.md"
    if not skill_md.exists():
        fail(f"missing {skill_md}")

    text = skill_md.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        fail(f"{skill_md} must start with YAML frontmatter")

    try:
        _, frontmatter, body = text.split("---\n", 2)
    except ValueError as exc:
        raise SystemExit(f"ERROR: {skill_md} has invalid frontmatter delimiters") from exc

    if "name:" not in frontmatter:
        fail(f"{skill_md} frontmatter is missing name")
    if "description:" not in frontmatter:
        fail(f"{skill_md} frontmatter is missing description")
    if "[TODO:" in text:
        fail(f"{skill_md} still contains TODO placeholders")
    if not body.strip():
        fail(f"{skill_md} must contain instructions after frontmatter")

    openai_yaml = skill_dir / "agents" / "openai.yaml"
    if not openai_yaml.exists():
        fail(f"missing {openai_yaml}")


def main() -> None:
    root = Path(__file__).resolve().parents[1] / "skills"
    if not root.exists():
        fail("skills/ directory is missing")

    skill_dirs = sorted(path for path in root.iterdir() if path.is_dir())
    if not skill_dirs:
        fail("no skills found under skills/")

    for skill_dir in skill_dirs:
        validate_skill(skill_dir)

    print(f"validated {len(skill_dirs)} skill(s)")


if __name__ == "__main__":
    main()
