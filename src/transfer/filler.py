#!/usr/bin/env python3
"""Step ④ structured filling: aggregate journal data into schema YAML.

Usage:
    python3 src/transfer/filler.py <domain> [--weeks W19,W21,...] [--annotations annotations.yaml]

Input:
    - data/journal-ingest.json (from src/data/ingest.py)
    - Optional: annotations YAML with causal type assignments

Output:
    - Schema YAML to stdout or --output file
"""

import json
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML is required. Install with: pip install pyyaml", file=sys.stderr)
    sys.exit(1)


def load_journal(path=None):
    if path is None:
        path = Path(__file__).resolve().parents[2] / "data" / "journal-ingest.json"
    with open(path) as f:
        return json.load(f)


def load_annotations(path):
    with open(path) as f:
        return yaml.safe_load(f) or {}


def merge_entities(weeks_data):
    seen = {}
    for wd in weeks_data:
        for s in wd.get("schemas", []):
            for e in s.get("entities", []):
                name = e["name"]
                if name in seen:
                    existing = seen[name]
                    for attr in e.get("attributes", []):
                        if attr not in existing["attributes"]:
                            existing["attributes"].append(attr)
                else:
                    seen[name] = {"name": name, "attributes": list(e.get("attributes", []))}
    return list(seen.values())


def _normalize(text):
    """Strip whitespace/punctuation for fuzzy matching."""
    return text.strip().rstrip("。，.!?").replace(" ", "")


def merge_causals(weeks_data, annotations):
    seen = {}
    type_map = {}
    if annotations:
        for a in annotations.get("causals", []):
            norm_cond = _normalize(a.get("condition", ""))
            type_map[norm_cond] = a

    # Apply annotations to journal data
    for wd in weeks_data:
        for s in wd.get("schemas", []):
            for c in s.get("causals", []):
                norm_cond = _normalize(c["condition"])
                # Check if this condition has a known annotation
                for annotated_cond, a in type_map.items():
                    if annotated_cond in norm_cond or norm_cond in annotated_cond:
                        c["type"] = a.get("type", "保留")
                        if "verify" in a:
                            c["verify"] = a["verify"]
                        break
                else:
                    if "type" not in c:
                        c["type"] = "保留"

    # Deduplicate by normalized condition
    result = {}
    for wd in weeks_data:
        for s in wd.get("schemas", []):
            for c in s.get("causals", []):
                key = _normalize(c["condition"])
                if key not in result:
                    entry = {"condition": c["condition"], "outcome": c["outcome"]}
                    for k in ("type", "verify", "note"):
                        if k in c:
                            entry[k] = c[k]
                    result[key] = entry

    return list(result.values())


def merge_boundaries(weeks_data):
    seen = set()
    for wd in weeks_data:
        for s in wd.get("schemas", []):
            for b in s.get("boundaries", []):
                if isinstance(b, str):
                    seen.add(b)
                elif isinstance(b, dict):
                    seen.add(b.get("description", str(b)))
    return sorted(seen)


def merge_properties(weeks_data):
    seen = {}
    for wd in weeks_data:
        for s in wd.get("schemas", []):
            for p in s.get("properties", []):
                seen[p["key"]] = p["value"]
    return [{"key": k, "value": v} for k, v in seen.items()]


def merge_dynamics(weeks_data):
    seen = {}
    for wd in weeks_data:
        for s in wd.get("schemas", []):
            for d in s.get("dynamics", []):
                seen[d["key"]] = d["value"]
    return [{"key": k, "value": v} for k, v in seen.items()]


def merge_mappings(weeks_data):
    seen = {}
    for wd in weeks_data:
        for s in wd.get("schemas", []):
            for m in s.get("mappings", []):
                intent = m["intent"]
                if intent in seen:
                    existing = seen[intent]
                    if m.get("action") and m["action"] not in existing.get("action", ""):
                        existing["action"] += "；" + m["action"]
                else:
                    seen[intent] = {
                        "intent": intent,
                        "action": m.get("action", ""),
                    }
                    for k in ("type", "caution", "verify"):
                        if k in m:
                            seen[intent][k] = m[k]
    return list(seen.values())


def merge_biases(weeks_data):
    seen = {}
    for wd in weeks_data:
        for s in wd.get("schemas", []):
            for b in s.get("biases", []):
                key = b.get("id") or b.get("belief", "")
                if key not in seen:
                    seen[key] = {"belief": b["belief"], "fact": b["fact"]}
                    for k in ("type", "verify"):
                        if k in b:
                            seen[key][k] = b[k]
    return list(seen.values())


def infer_usage(situations):
    agendas = [s["content"]["agenda"] for s in situations if "content" in s]
    if agendas:
        return "结合多周数据提炼：" + "；".join(agendas)
    return "暂无描述"


def fill(domain, weeks_data, annotations=None):
    """Fill schema for a domain from multi-week data."""
    situations = []
    for wd in weeks_data:
        situations.extend(wd.get("situations", []))

    schema = {
        "usage": infer_usage(situations),
        "entities": merge_entities(weeks_data),
        "causals": merge_causals(weeks_data, annotations),
        "boundaries": merge_boundaries(weeks_data),
        "properties": merge_properties(weeks_data),
        "dynamics": merge_dynamics(weeks_data),
        "mappings": merge_mappings(weeks_data),
        "biases": merge_biases(weeks_data),
    }
    return {"schemas": [schema]}


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Schema filler")
    parser.add_argument("domain", help="Domain name (e.g. think, business)")
    parser.add_argument("--weeks", help="Comma-separated week list (e.g. W19,W21,W22,W23)")
    parser.add_argument("--annotations", help="Path to annotations YAML")
    parser.add_argument("--output", help="Output file path")
    args = parser.parse_args()

    journal = load_journal()
    annotations = load_annotations(args.annotations) if args.annotations else {}

    if args.weeks:
        week_names = [f"2026-{w}" for w in args.weeks.split(",")]
    else:
        week_names = sorted(journal.keys())

    weeks_data = []
    for wn in week_names:
        if wn in journal and args.domain in journal[wn]["domains"]:
            weeks_data.append(journal[wn]["domains"][args.domain])

    if not weeks_data:
        print(f"No data for domain '{args.domain}' in weeks {week_names}", file=sys.stderr)
        sys.exit(1)

    result = fill(args.domain, weeks_data, annotations)

    output = yaml.dump(result, allow_unicode=True, sort_keys=False, default_flow_style=False)
    if args.output:
        Path(args.output).write_text(output, encoding="utf-8")
        print(f"Wrote {args.output}")
    else:
        print(output)


if __name__ == "__main__":
    main()
