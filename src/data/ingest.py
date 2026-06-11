#!/usr/bin/env python3
"""Journal data ingestion pipeline.

Outputs flat JSON: { week -> { domain -> { schemas, situations, intentions, thoughts } } }

Usage:
    python3 src/data/ingest.py <week>
    python3 src/data/ingest.py --all
"""

import json, os, sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("ERROR: pip install pyyaml", file=sys.stderr)
    sys.exit(1)

JOURNAL_ROOT = Path(__file__).resolve().parents[4] / "data" / "journal" / "quanttide-founder"

def load_yaml(path):
    with open(path) as f:
        return yaml.safe_load(f)

def ingest_week(week_dir):
    week_name = week_dir.name
    result = {}
    for fpath in sorted(week_dir.iterdir()):
        if fpath.suffix == ".yaml":
            domain = fpath.stem
            data = load_yaml(fpath)
            result[domain] = {
                "schemas": data.get("schemas", []),
                "situations": data.get("situations", []),
                "intentions": data.get("intentions", []),
                "thoughts": data.get("thoughts", []),
            }
    return result

def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    if sys.argv[1] == "--all":
        weeks = sorted(JOURNAL_ROOT.iterdir())
    else:
        weeks = [JOURNAL_ROOT / sys.argv[1]]
    all_data = {}
    for wp in weeks:
        if not wp.is_dir():
            continue
        all_data[wp.name] = ingest_week(wp)
    output_dir = Path("data")
    output_dir.mkdir(exist_ok=True)
    with open(output_dir / "journal-ingest.json", "w") as f:
        json.dump(all_data, f, ensure_ascii=False, indent=2)
    print(f"Wrote {len(all_data)} weeks to data/journal-ingest.json")

if __name__ == "__main__":
    main()
