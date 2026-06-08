#!/usr/bin/env python3
"""A/B comparison evaluation for intent inference (project-03 vs project-04)."""

import json
import os
import subprocess
import sys
from datetime import datetime

BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
PROJECT_03_DIR = os.path.abspath(os.path.join(BASE_DIR, "..", "project-03"))
PROJECT_04_DIR = os.path.abspath(os.path.join(BASE_DIR, "..", "project-04"))


def resolve_binary(project_dir, name):
    candidates = [
        os.path.join(project_dir, "target", "release", name),
        os.path.join(project_dir, "target", "debug", name),
    ]
    for c in candidates:
        if os.path.isfile(c):
            return c
    sys.exit(f"Binary not found: {name} in {project_dir}")


BIN_A = resolve_binary(PROJECT_03_DIR, "project-03")
BIN_B = resolve_binary(PROJECT_04_DIR, "project-04")
KEYWORDS_PATH = os.path.abspath(os.path.join(PROJECT_03_DIR, "data", "keywords.json"))
GRAPH_PATH = os.path.abspath(os.path.join(PROJECT_04_DIR, "data", "graph-init.json"))


def run_binary(bin_path, args, input_text):
    try:
        proc = subprocess.run(
            [bin_path] + args,
            input=input_text.encode("utf-8"),
            capture_output=True,
            timeout=30,
        )
        if proc.returncode != 0:
            return None, f"exit code {proc.returncode}: {proc.stderr.decode()}"
        output = json.loads(proc.stdout.decode("utf-8"))
        return output, None
    except json.JSONDecodeError as e:
        return None, f"invalid JSON: {e}"
    except FileNotFoundError:
        return None, "binary not found"
    except subprocess.TimeoutExpired:
        return None, "timeout"
    except Exception as e:
        return None, str(e)


def get_matched_ids_a(output):
    if output is None:
        return set()
    matched = output.get("matched", [])
    return {m["id"] for m in matched}


def get_matched_ids_b(output):
    if output is None:
        return set()
    nodes = output.get("match_nodes", [])
    return {n["id"] for n in nodes}


def count_incremental_relations(output, expected_clusters):
    if output is None:
        return 0
    count = 0
    for n in output.get("neighbors", []):
        if n["from"] in expected_clusters or n["to"] in expected_clusters:
            count += 1
    for path in output.get("bfs_paths", []):
        for step in path:
            if step["from"] in expected_clusters or step["to"] in expected_clusters:
                count += 1
                break
    return count


def false_positive_rate(matched, expected):
    if len(matched) == 0:
        return 0.0
    fp = len(matched - expected)
    return fp / len(matched)


def recall(matched, expected):
    if len(expected) == 0:
        return 0.0
    tp = len(matched & expected)
    return tp / len(expected)


def compute_sample_metrics(baseline_entry, output_a, output_b):
    expected_clusters = set(baseline_entry["clusters"])
    matched_a = get_matched_ids_a(output_a)
    matched_b = get_matched_ids_b(output_b)

    recall_a = recall(matched_a, expected_clusters)
    recall_b = recall(matched_b, expected_clusters)

    incremental_nodes = len(matched_b - matched_a)
    incremental_relations = count_incremental_relations(output_b, expected_clusters)

    fp_a = false_positive_rate(matched_a, expected_clusters)
    fp_b = false_positive_rate(matched_b, expected_clusters)

    path_grades = []
    if output_b:
        for n in output_b.get("neighbors", []):
            path_grades.append({
                "from": n["from"],
                "to": n["to"],
                "depth": 1,
                "relation": n["relation"],
                "grade": None,
            })
        for path in output_b.get("bfs_paths", []):
            for step in path:
                path_grades.append({
                    "from": step["from"],
                    "to": step["to"],
                    "depth": len(path),
                    "relation": step["relation"],
                    "grade": None,
                })

    sample = {
        "id": baseline_entry["id"],
        "baseline": {
            "clusters": baseline_entry["clusters"],
            "relations": baseline_entry.get("relations", []),
        },
        "approach_a": {
            "matched": sorted(matched_a),
            "raw": output_a,
        },
        "approach_b": {
            "matched": sorted(matched_b),
            "neighbors": output_b.get("neighbors", []) if output_b else [],
            "bfs_paths": output_b.get("bfs_paths", []) if output_b else [],
            "conflicts": output_b.get("conflicts", []) if output_b else [],
            "candidate_edges": output_b.get("candidate_edges", []) if output_b else [],
            "raw": output_b,
        },
        "metrics": {
            "recall_a": round(recall_a, 4),
            "recall_b": round(recall_b, 4),
            "incremental_nodes": incremental_nodes,
            "incremental_relations": incremental_relations,
            "false_positive_a": round(fp_a, 4),
            "false_positive_b": round(fp_b, 4),
        },
        "path_grades": path_grades,
    }
    return sample


def generate_md_report(samples, summary, caveat=True):
    lines = []
    lines.append("# A/B Comparison Evaluation Report")
    lines.append("")
    lines.append(f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M')}")
    lines.append(f"**Total samples**: {summary['total_samples']}")
    lines.append("")
    if caveat:
        lines.append("> ⚠️ **Caveat**: Baseline annotations are best-effort based on W23 intent analysis, ")
        lines.append("> not human-verified for these segments. Metrics should be interpreted as indicative, not absolute.")
        lines.append("")
    lines.append("---")
    lines.append("")

    for s in samples:
        lines.append(f"## {s['id']}")
        lines.append("")
        lines.append("### Segment")
        lines.append("")
        seg = None
        for b in summary.get("_baselines", []):
            if b["id"] == s["id"]:
                seg = b.get("segment", "")
                break
        if seg:
            lines.append(f"> {seg}")
        lines.append("")
        lines.append("### Cluster Match Comparison")
        lines.append("")
        lines.append("| Scheme | Matched Clusters | Recall | False Positive |")
        lines.append("|--------|-----------------|--------|---------------|")
        exp_str = str(s["baseline"]["clusters"])
        a_str = str(s["approach_a"]["matched"])
        b_str = str(s["approach_b"]["matched"])
        lines.append(f"| **Baseline** | {exp_str} | — | — |")
        lines.append(f"| **Scheme A** (text match) | {a_str} | {s['metrics']['recall_a']:.2%} | {s['metrics']['false_positive_a']:.2%} |")
        lines.append(f"| **Scheme B** (graph inference) | {b_str} | {s['metrics']['recall_b']:.2%} | {s['metrics']['false_positive_b']:.2%} |")
        lines.append("")

        nb = s["approach_b"]["neighbors"]
        bfs = s["approach_b"]["bfs_paths"]
        cf = s["approach_b"]["conflicts"]
        ce = s["approach_b"]["candidate_edges"]
        lines.append("### Scheme B Additional Discoveries")
        lines.append("")
        lines.append(f"- **Incremental nodes**: {s['metrics']['incremental_nodes']}")
        lines.append(f"- **Incremental relations**: {s['metrics']['incremental_relations']}")
        lines.append(f"- **L1 neighbors**: {len(nb)}")
        lines.append(f"- **L2 BFS paths**: {len(bfs)}")
        lines.append(f"- **L3 conflicts**: {len(cf)}")
        lines.append(f"- **L4 candidate edges**: {len(ce)}")
        lines.append("")

        if nb:
            lines.append("| From | To | Relation | Direction |")
            lines.append("|------|----|----------|-----------|")
            for n in nb:
                lines.append(f"| {n['from']} | {n['to']} | {n['relation']} | {n.get('direction', '—')} |")
            lines.append("")

        if bfs:
            lines.append("#### BFS Paths")
            lines.append("")
            for i, path in enumerate(bfs):
                steps = " → ".join(f"{s['from']}→{s['to']}({s['relation']})" for s in path)
                lines.append(f"{i+1}. {steps}")
            lines.append("")

        if cf:
            lines.append("#### Conflicts")
            lines.append("")
            lines.append("| Node A | Node B | Type | Via |")
            lines.append("|--------|--------|------|-----|")
            for c in cf:
                lines.append(f"| {c['node_a']} | {c['node_b']} | {c.get('relation_type', '')} | {c.get('via', [])} |")
            lines.append("")

        if ce:
            lines.append("#### Candidate Edges")
            lines.append("")
            lines.append("| From | To | Proposed Type | Confidence |")
            lines.append("|------|----|--------------|------------|")
            for e in ce:
                lines.append(f"| {e['from']} | {e['to']} | {e['proposed_type']} | {e['confidence']} |")
            lines.append("")

        pgs = s["path_grades"]
        if pgs:
            lines.append("### Path Quality Review")
            lines.append("")
            lines.append("| From | To | Depth | Relation | Grade | Note |")
            lines.append("|------|----|-------|----------|-------|------|")
            for p in pgs:
                g = p.get("grade", "")
                if g is None:
                    g = "—"
                lines.append(f"| {p['from']} | {p['to']} | {p['depth']} | {p['relation']} | {g} | |")
            lines.append("")

        lines.append("---")
        lines.append("")

    lines.append("## Summary Metrics")
    lines.append("")
    lines.append("| Metric | Value |")
    lines.append("|--------|-------|")
    lines.append(f"| Total samples | {summary['total_samples']} |")
    lines.append(f"| Avg recall (Scheme A) | {summary['avg_recall_a']:.2%} |")
    lines.append(f"| Avg recall (Scheme B) | {summary['avg_recall_b']:.2%} |")
    lines.append(f"| Total incremental nodes | {summary['total_incremental_nodes']} |")
    lines.append(f"| Total incremental relations | {summary['total_incremental_relations']} |")
    lines.append(f"| Avg false positive (Scheme A) | {summary['avg_false_positive_a']:.2%} |")
    lines.append(f"| Avg false positive (Scheme B) | {summary['avg_false_positive_b']:.2%} |")
    lines.append(f"| Avg path grade | {summary['avg_path_grade']:.2f} |" if summary.get("avg_path_grade") is not None else "| Avg path grade | — |")
    lines.append("")
    lines.append("### Notes")
    lines.append("")
    lines.append("- **Limitation**: Test segments are from W23 (not W24), which was used for graph construction.")
    lines.append("  This means both schemes may score higher on recall than on truly unseen data.")
    lines.append("- **Baseline**: Generated from W23 intent analysis, not human-verified per segment.")
    lines.append("- **Path grades**: To be filled in by human reviewer against original text.")
    lines.append("")
    return "\n".join(lines)


def main():
    test_set_path = os.path.join(BASE_DIR, "data", "test-set.json")
    baseline_path = os.path.join(BASE_DIR, "data", "baseline-w24.json")
    output_json_path = os.path.join(BASE_DIR, "outputs", "evaluation.json")
    output_md_path = os.path.join(BASE_DIR, "outputs", "evaluation.md")

    with open(test_set_path, "r") as f:
        test_set = json.load(f)
    with open(baseline_path, "r") as f:
        baselines = json.load(f)

    baseline_map = {b["id"]: b for b in baselines}

    samples = []
    for entry in test_set:
        tid = entry["id"]
        segment = entry["segment"]
        baseline = baseline_map.get(tid)
        if baseline is None:
            print(f"WARNING: No baseline for {tid}, skipping", file=sys.stderr)
            continue

        print(f"Processing {tid}...", file=sys.stderr)
        common_args = ["--keywords", KEYWORDS_PATH]
        output_a, err_a = run_binary(BIN_A, common_args, segment)
        if err_a:
            print(f"  Scheme A error: {err_a}", file=sys.stderr)

        output_b, err_b = run_binary(
            BIN_B, common_args + ["--graph", GRAPH_PATH], segment
        )
        if err_b:
            print(f"  Scheme B error: {err_b}", file=sys.stderr)

        sample = compute_sample_metrics(baseline, output_a, output_b)
        samples.append(sample)

    total = len(samples)
    if total == 0:
        print("ERROR: No samples processed", file=sys.stderr)
        sys.exit(1)

    avg_recall_a = sum(s["metrics"]["recall_a"] for s in samples) / total
    avg_recall_b = sum(s["metrics"]["recall_b"] for s in samples) / total
    total_inc_nodes = sum(s["metrics"]["incremental_nodes"] for s in samples)
    total_inc_rels = sum(s["metrics"]["incremental_relations"] for s in samples)
    avg_fp_a = sum(s["metrics"]["false_positive_a"] for s in samples) / total
    avg_fp_b = sum(s["metrics"]["false_positive_b"] for s in samples) / total

    all_grades = []
    for s in samples:
        for pg in s["path_grades"]:
            if pg.get("grade") is not None:
                all_grades.append(pg["grade"])
    avg_grade = sum(all_grades) / len(all_grades) if all_grades else None

    summary = {
        "total_samples": total,
        "avg_recall_a": round(avg_recall_a, 4),
        "avg_recall_b": round(avg_recall_b, 4),
        "total_incremental_nodes": total_inc_nodes,
        "total_incremental_relations": total_inc_rels,
        "avg_false_positive_a": round(avg_fp_a, 4),
        "avg_false_positive_b": round(avg_fp_b, 4),
        "avg_path_grade": round(avg_grade, 4) if avg_grade is not None else None,
        "_baselines": [{"id": b["id"], "segment": b.get("segment", "")} for b in baselines],
        "caveat": "Test segments from W23 (used for graph construction), not truly unseen W24 data. Baseline is best-effort from intent analysis.",
    }

    output = {"samples": samples, "summary": summary}

    os.makedirs(os.path.dirname(output_json_path), exist_ok=True)
    with open(output_json_path, "w") as f:
        json.dump(output, f, ensure_ascii=False, indent=2)

    md = generate_md_report(samples, summary)
    with open(output_md_path, "w") as f:
        f.write(md)

    print(f"Evaluation complete: {output_json_path}, {output_md_path}", file=sys.stderr)

    print("\nQuick summary:")
    print(f"  Samples: {total}")
    print(f"  Avg recall A: {avg_recall_a:.2%}")
    print(f"  Avg recall B: {avg_recall_b:.2%}")
    print(f"  Total incremental nodes: {total_inc_nodes}")
    print(f"  Total incremental relations: {total_inc_rels}")
    print(f"  Avg FP A: {avg_fp_a:.2%}")
    print(f"  Avg FP B: {avg_fp_b:.2%}")


if __name__ == "__main__":
    main()
