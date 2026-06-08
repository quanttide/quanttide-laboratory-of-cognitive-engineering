use std::collections::HashMap;
use std::fs;

use intent_graph::graph::IntentGraph;
use intent_graph::builder::GraphBuilder;
use serde::Serialize;

use project_06::TestSegment;

#[derive(Serialize)]
struct SegmentResult {
    id: String,
    week: String,
    expected: Vec<u32>,
    matched_ids: Vec<u32>,
    matched_rank_of_expected: HashMap<u32, Option<usize>>,
    hit_top1: bool,
    hit_top3: bool,
    hit_any: bool,
}

#[derive(Serialize)]
struct ClusterRecall {
    cluster: u32,
    name: String,
    total: usize,
    top1_hits: usize,
    top3_hits: usize,
    recall_top1: f64,
    recall_top3: f64,
}

#[derive(Serialize)]
struct Report {
    total_segments: usize,
    top1_recall: f64,
    top3_recall: f64,
    any_recall: f64,
    per_cluster: Vec<ClusterRecall>,
    results: Vec<SegmentResult>,
}

fn parse_cluster_names_from_yaml(path: &str) -> HashMap<u32, String> {
    let content = fs::read_to_string(path).expect("Failed to read YAML");
    let mut map = HashMap::new();
    let mut current_id: Option<u32> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("- id: ") {
            current_id = stripped.trim().parse::<u32>().ok();
        } else if let Some(stripped) = trimmed.strip_prefix("name: ") {
            if let Some(id) = current_id {
                map.insert(id, stripped.trim().to_string());
            }
            current_id = None;
        }
    }
    map
}

fn main() {
    let data_path = "examples/project-06/data/input/match-test-set.json";
    let intent_yaml = "assets/refined/intent.yaml";
    let relation_yaml = "assets/refined/intent-relation.yaml";

    let test_data: Vec<TestSegment> =
        serde_json::from_str(&fs::read_to_string(data_path).expect("Failed to read test data"))
            .expect("Failed to parse test data");

    let graph = IntentGraph::load(intent_yaml, relation_yaml).expect("Failed to build graph");
    let kw_table =
        GraphBuilder::build_keyword_table_from_yaml(intent_yaml).expect("Failed to build keyword table");
    let cluster_names = parse_cluster_names_from_yaml(intent_yaml);

    let mut results = Vec::new();
    let mut cluster_stats: HashMap<u32, (usize, usize, usize)> = HashMap::new();

    for segment in &test_data {
        let matched = graph.match_nodes(&kw_table, &segment.segment, 0.0);
        let matched_ids: Vec<u32> = matched.iter().map(|m| m.id).collect();
        let top3_ids: Vec<u32> = matched.iter().take(3).map(|m| m.id).collect();

        let mut rank_map = HashMap::new();
        for c in &segment.clusters {
            let rank = matched.iter().position(|m| m.id == *c);
            rank_map.insert(*c, rank);
        }

        let hit_any = segment.clusters.iter().any(|c| matched_ids.contains(c));
        let hit_top1 = segment.clusters.iter().any(|c| top3_ids.get(0) == Some(c));
        let hit_top3 = segment.clusters.iter().any(|c| top3_ids.contains(c));

        results.push(SegmentResult {
            id: segment.id.clone(),
            week: segment.week.clone(),
            expected: segment.clusters.clone(),
            matched_ids,
            matched_rank_of_expected: rank_map,
            hit_top1,
            hit_top3,
            hit_any,
        });

        for c in &segment.clusters {
            let entry = cluster_stats.entry(*c).or_insert((0, 0, 0));
            entry.0 += 1;
            if hit_top1 {
                entry.1 += 1;
            }
            if hit_top3 {
                entry.2 += 1;
            }
        }
    }

    let total = results.len();
    let top1_hits = results.iter().filter(|r| r.hit_top1).count();
    let top3_hits = results.iter().filter(|r| r.hit_top3).count();
    let any_hits = results.iter().filter(|r| r.hit_any).count();

    let mut per_cluster: Vec<ClusterRecall> = cluster_stats
        .into_iter()
        .map(|(c, (total, t1, t3))| ClusterRecall {
            cluster: c,
            name: cluster_names.get(&c).cloned().unwrap_or_default(),
            total,
            top1_hits: t1,
            top3_hits: t3,
            recall_top1: if total > 0 { t1 as f64 / total as f64 } else { 0.0 },
            recall_top3: if total > 0 { t3 as f64 / total as f64 } else { 0.0 },
        })
        .collect();
    per_cluster.sort_by_key(|c| c.cluster);

    let report = Report {
        total_segments: total,
        top1_recall: if total > 0 { top1_hits as f64 / total as f64 } else { 0.0 },
        top3_recall: if total > 0 { top3_hits as f64 / total as f64 } else { 0.0 },
        any_recall: if total > 0 { any_hits as f64 / total as f64 } else { 0.0 },
        per_cluster,
        results,
    };

    fs::create_dir_all("examples/project-06/data/output").ok();
    fs::write(
        "examples/project-06/data/output/match-result.json",
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .expect("Failed to write output");

    println!("=== Match Experiment (6.1) ===");
    println!("Segments:    {}", report.total_segments);
    println!("Top-1 recall: {:.1}%", report.top1_recall * 100.0);
    println!("Top-3 recall: {:.1}%", report.top3_recall * 100.0);
    println!("Any recall:   {:.1}%", report.any_recall * 100.0);
    println!();
    for pc in &report.per_cluster {
        println!(
            "  Cluster {:2} ({:12}) : top1={}/{}={:.0}%  top3={}/{}={:.0}%",
            pc.cluster,
            pc.name,
            pc.top1_hits,
            pc.total,
            pc.recall_top1 * 100.0,
            pc.top3_hits,
            pc.total,
            pc.recall_top3 * 100.0
        );
    }
    println!();
    for r in &report.results {
        let rank_str: Vec<String> = r
            .expected
            .iter()
            .map(|c| {
                if let Some(Some(pos)) = r.matched_rank_of_expected.get(c) {
                    format!("#{}", pos + 1)
                } else {
                    "✗".to_string()
                }
            })
            .collect();
        println!(
            "  {} expected={:?} rank=[{}]",
            r.id, r.expected, rank_str.join(",")
        );
    }
}
