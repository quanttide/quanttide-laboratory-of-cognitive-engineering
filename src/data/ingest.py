#!/usr/bin/env python3
"""Journal data ingestion pipeline.

Reads quanttide-founder journal YAML files and outputs structured JSON
for downstream schema migration.

Usage:
    python3 src/data/ingest.py <week>  # single week
    python3 src/data/ingest.py --all   # all weeks
"""

import json
import os
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML is required. Install with: pip install pyyaml", file=sys.stderr)
    sys.exit(1)


JOURNAL_ROOT = Path(__file__).resolve().parents[4] / "data" / "journal" / "quanttide-founder"


def load_yaml(path):
    with open(path) as f:
        return yaml.safe_load(f)


def ingest_week(week_dir):
    """Ingest a single week directory, return structured dict."""
    week_name = week_dir.name
    result = {
        "week": week_name,
        "domains": {},
        "thoughts": {},
    }

    for fpath in sorted(week_dir.iterdir()):
        if fpath.suffix == ".yaml":
            domain = fpath.stem  # e.g. "think", "business"
            data = load_yaml(fpath)
            result["domains"][domain] = {
                "schemas": data.get("schemas", []),
                "situations": data.get("situations", []),
                "intentions": data.get("intentions", []),
                "thoughts": data.get("thoughts", []),
            }

        elif fpath.name == "thoughts" and fpath.is_dir():
            for thought_file in sorted(fpath.iterdir()):
                if thought_file.suffix == ".md":
                    result["thoughts"][thought_file.stem] = thought_file.read_text(encoding="utf-8")

    return result


def validate_week(data):
    """Validate that a week's data is structurally complete."""
    issues = []
    if not data["domains"]:
        issues.append("No domain YAML files found")

    for domain_name, domain_data in data["domains"].items():
        if not domain_data["schemas"]:
            issues.append(f"{domain_name}: empty schemas")
        if not domain_data["situations"]:
            issues.append(f"{domain_name}: empty situations")
        if not domain_data["intentions"]:
            issues.append(f"{domain_name}: empty intentions")

    return issues


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    if sys.argv[1] == "--all":
        weeks = sorted(JOURNAL_ROOT.iterdir())
    else:
        week_name = sys.argv[1]
        weeks = [JOURNAL_ROOT / week_name]

    all_data = {}
    total_issues = 0

    for week_path in weeks:
        if not week_path.is_dir():
            print(f"SKIP {week_path.name}: not a directory", file=sys.stderr)
            continue

        print(f"INGEST {week_path.name}...", end=" ")
        data = ingest_week(week_path)
        issues = validate_week(data)

        if issues:
            print(f"{len(issues)} issue(s)")
            for issue in issues:
                print(f"  - {issue}")
            total_issues += len(issues)
        else:
            domains = list(data["domains"].keys())
            thought_count = len(data["thoughts"])
            print(f"OK ({len(domains)} domains, {thought_count} thoughts)")

        all_data[week_path.name] = data

    # Write output
    output_dir = Path("data")
    output_dir.mkdir(exist_ok=True)
    output_path = output_dir / "journal-ingest.json"
    with open(output_path, "w") as f:
        json.dump(all_data, f, ensure_ascii=False, indent=2)

    print(f"\nOUTPUT {output_path.resolve()}")
    if total_issues:
        print(f"WARNING: {total_issues} issue(s) found")
        sys.exit(1)


if __name__ == "__main__":
    main()
