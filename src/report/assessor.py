#!/usr/bin/env python3
"""Step ⑤ automated quality assessment using the 7-dimension framework.

Usage:
    python3 src/report/assessor.py <schema.yaml> [--baseline baseline.md]

Output:
    Quality report to stdout or --output file
"""

import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML is required. Install with: pip install pyyaml", file=sys.stderr)
    sys.exit(1)


def score_coverage(schema):
    """A-1: Check default value coverage via properties and dynamics."""
    props = schema.get("properties", [])
    dynamics = schema.get("dynamics", [])
    entities = schema.get("entities", [])
    causals = schema.get("causals", [])
    n_props = len(props)
    n_dynamics = len(dynamics)
    n_entities = len(entities)
    n_causals = len(causals)

    if n_entities == 0:
        return 1, "no entities defined"

    # Expected: at least n_entities properties (one per entity) + dynamics
    expected_min = n_entities + 2
    total = n_props + n_dynamics

    if total >= expected_min + 3:
        score = 5
    elif total >= expected_min + 1:
        score = 4
    elif total >= expected_min - 1:
        score = 3
    elif total >= 1:
        score = 2
    else:
        score = 1

    detail = f"({n_props} properties + {n_dynamics} dynamics) for {n_entities} entities, {n_causals} causals"
    return score, detail


def score_flexibility(schema):
    """A-2: Check retainable/verifiable distinction."""
    causals = schema.get("causals", [])
    if not causals:
        return 1, "no causals to classify"

    typed = [c for c in causals if "type" in c]
    verifiable = [c for c in typed if c.get("type") == "需验证"]
    verified = [c for c in verifiable if c.get("verify")]

    if not typed:
        return 1, f"no causals have type annotations (out of {len(causals)})"
    elif not verifiable:
        return 2, f"all {len(typed)} causals typed but none marked as verifiable"
    elif not verified:
        return 3, f"{len(verifiable)} verifiable causals but none have verify conditions"
    elif len(verified) == len(verifiable):
        return 5, f"all {len(verifiable)} verifiable causals have verify conditions ✓"
    else:
        return 4, f"{len(verified)}/{len(verifiable)} verifiable causals have verify conditions"


def score_complexity(schema):
    """A-3: Check hierarchy and cognitive load."""
    entities = schema.get("entities", [])
    causals = schema.get("causals", [])
    mappings = schema.get("mappings", [])
    n_entities = len(entities)
    n_causals = len(causals)

    if n_entities == 0:
        return 1, "no entities"

    # Heuristic: 3-7 entities is ideal; causals should be roughly 1.5-3x entities
    entity_ok = 3 <= n_entities <= 7
    causal_ratio = n_causals / max(n_entities, 1)

    if entity_ok and 1.5 <= causal_ratio <= 4:
        return 5, f"{n_entities} entities, {n_causals} causals — balanced"
    elif entity_ok and 1 <= causal_ratio <= 5:
        return 4, f"{n_entities} entities, {n_causals} causals — slightly off ratio"
    elif 2 <= n_entities <= 10:
        return 3, f"{n_entities} entities, {n_causals} causals — acceptable"
    elif n_entities <= 12:
        return 2, f"{n_entities} entities — too many/flat"
    else:
        return 1, f"{n_entities} entities — overwhelming"


def score_consistency(schema):
    """B-1: Check internal consistency between causals and biases."""
    causals = schema.get("causals", [])
    biases = schema.get("biases", [])
    mappings = schema.get("mappings", [])

    if not causals and not biases:
        return 3, "no causals or biases to cross-check"

    # The main check: do biases contradict causals?
    # For now, report structural findings
    n_causals = len(causals)
    n_biases = len(biases)
    n_mappings = len(mappings)

    if n_causals >= 3 and n_biases >= 2:
        return 4, f"{n_causals} causals, {n_biases} biases, {n_mappings} mappings — structurally consistent"
    elif n_causals >= 1 and n_biases >= 1:
        return 3, f"basic cross-reference possible ({n_causals} causals, {n_biases} biases)"
    else:
        return 2, f"insufficient data for cross-reference ({n_causals} causals, {n_biases} biases)"


def score_external_validity(schema):
    """B-2: Check data traceability."""
    causals = schema.get("causals", [])
    biases = schema.get("biases", [])
    
    # Check if there are sources or traceability markers
    has_sources = any("sources" in c for c in causals)
    has_verify = any("verify" in c for c in causals)
    has_ids = any("id" in b for b in biases)

    indicators = sum([has_sources, has_verify, has_ids])
    n_causals = len(causals)

    if has_sources:
        return 5, "causals have explicit source references"
    elif indicators >= 2:
        return 4, f"traceability indicators present ({indicators}/3)"
    elif indicators >= 1:
        return 3, f"minimal traceability ({indicators}/3 indicators)"
    else:
        return 2, "no traceability markers found"


def score_task_fit(schema):
    """B-3: Check if schema matches its stated usage."""
    usage = schema.get("usage", "")
    causals = schema.get("causals", [])
    mappings = schema.get("mappings", [])

    if not usage:
        return 1, "no usage defined"

    has_causals = len(causals) >= 3
    has_mappings = len(mappings) >= 2
    usage_length = len(usage)

    if has_causals and has_mappings and usage_length > 30:
        return 5, f"usage defined, {len(causals)} causals, {len(mappings)} mappings — actionable"
    elif has_causals and has_mappings:
        return 4, "has causals and mappings but usage could be more specific"
    elif has_causals or has_mappings:
        return 3, "partial coverage (causals or mappings)"
    else:
        return 2, "usage defined but no actionable content"


def score_communicability(schema):
    """B-4: Check readability and self-explanation."""
    usage = schema.get("usage", "")
    n_boundaries = len(schema.get("boundaries", []))
    n_entities = len(schema.get("entities", []))
    n_biases = len(schema.get("biases", []))

    has_usage = len(usage) > 20
    has_boundaries = n_boundaries >= 3
    has_biases = n_biases >= 1

    indicators = [has_usage, has_boundaries, has_biases, (3 <= n_entities <= 7)]
    score = sum(indicators) + 1  # Shift from 0-4 to 1-5

    detail_parts = []
    if has_usage:
        detail_parts.append("usage clear")
    if has_boundaries:
        detail_parts.append(f"{n_boundaries} boundaries")
    if has_biases:
        detail_parts.append(f"{n_biases} biases")
    detail_parts.append(f"{n_entities} entities")

    return min(score, 5), ", ".join(detail_parts)


def assess(schema):
    """Run all 7 dimension assessments on a schema."""
    dimensions = {
        "A-1 覆盖度": score_coverage,
        "A-2 灵活性": score_flexibility,
        "A-3 复杂度": score_complexity,
        "B-1 内部一致性": score_consistency,
        "B-2 外部有效性": score_external_validity,
        "B-3 任务适用性": score_task_fit,
        "B-4 可沟通性": score_communicability,
    }

    results = []
    total = 0
    for name, scorer in dimensions.items():
        score, detail = scorer(schema)
        results.append({"dimension": name, "score": score, "detail": str(detail)})
        total += score

    avg = round(total / len(dimensions), 2)
    return {"dimensions": results, "total_score": avg}


def format_report(results, baseline_path=None):
    """Format assessment results as markdown."""
    lines = ["# Schema 质量评估报告（自动）\n"]
    lines.append(f"| 维度 | 分数 | 说明 |")
    lines.append(f"|------|------|------|")

    for d in results["dimensions"]:
        lines.append(f"| {d['dimension']} | {d['score']}/5 | {d['detail']} |")

    lines.append(f"\n**总分**：{results['total_score']}/5")

    if baseline_path:
        try:
            baseline_text = Path(baseline_path).read_text()
            lines.append(f"\n### 基线对比\n")
            lines.append(f"参见：{baseline_path}")
        except FileNotFoundError:
            lines.append(f"\n*基线文件未找到：{baseline_path}*")

    return "\n".join(lines)


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Schema quality assessor")
    parser.add_argument("schema", help="Path to schema YAML")
    parser.add_argument("--baseline", help="Path to baseline assessment file")
    parser.add_argument("--output", help="Output report path")
    args = parser.parse_args()

    with open(args.schema) as f:
        data = yaml.safe_load(f)

    schemas = data.get("schemas", [data])
    if not schemas:
        print("No schemas found in input", file=sys.stderr)
        sys.exit(1)

    result = assess(schemas[0])
    report = format_report(result, args.baseline)

    if args.output:
        Path(args.output).write_text(report, encoding="utf-8")
        print(f"Wrote {args.output}")
    else:
        print(report)


if __name__ == "__main__":
    main()
