use std::collections::{HashMap, HashSet};
use std::fs;

use intent_graph::graph::IntentGraph;
use intent_graph::models::{EdgeWeight, NodeWeight};
use serde::Serialize;

use project_06::{
    FeedbackData, FeedbackWeek, FeedbackSegment, IncrementalEdge, WeeklySnapshot,
};

#[derive(Serialize)]
struct WeeklyResult {
    week: String,
    condition: String,
    total_expected: usize,
    total_found: usize,
    recall: f64,
    segment_details: Vec<SegmentDetail>,
}

#[derive(Serialize)]
struct SegmentDetail {
    id: String,
    clusters: Vec<u32>,
    expected_relations: Vec<(u32, u32, String)>,
    found_relations: Vec<(u32, u32, String)>,
    found_count: usize,
    total_count: usize,
}

#[derive(Serialize)]
struct ComparisonReport {
    no_update: Vec<WeeklyResult>,
    with_update: Vec<WeeklyResult>,
}

fn load_cluster_names(intent_yaml: &str) -> HashMap<u32, String> {
    let content = fs::read_to_string(intent_yaml).expect("Failed to read YAML");
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

fn build_weekly_graph(snapshot: &WeeklySnapshot, names: &HashMap<u32, String>) -> IntentGraph {
    let mut graph = IntentGraph::new();
    for &cid in &snapshot.active_clusters {
        let name = names.get(&cid).cloned().unwrap_or_else(|| format!("Cluster {}", cid));
        graph.add_node(NodeWeight {
            id: cid,
            name,
            r#type: String::new(),
            evolution: String::new(),
            per_week_intents: Vec::new(),
        });
    }
    for edge in &snapshot.stable_relations {
        graph.add_edge(
            edge.from,
            edge.to,
            EdgeWeight {
                relation_type: edge.relation_type.clone(),
                logic: String::new(),
                weeks: vec![snapshot.week.clone()],
                period_type: "stable".to_string(),
            },
        );
    }
    for edge in &snapshot.periodic_relations {
        graph.add_edge(
            edge.from,
            edge.to,
            EdgeWeight {
                relation_type: edge.relation_type.clone(),
                logic: String::new(),
                weeks: vec![snapshot.week.clone()],
                period_type: "periodic".to_string(),
            },
        );
    }
    for edge in &snapshot.situational_relations {
        graph.add_edge(
            edge.from,
            edge.to,
            EdgeWeight {
                relation_type: edge.relation_type.clone(),
                logic: String::new(),
                weeks: vec![snapshot.week.clone()],
                period_type: "situational".to_string(),
            },
        );
    }
    graph
}

fn apply_incremental(
    graph: &mut IntentGraph,
    new_clusters: &[u32],
    incremental_relations: &[IncrementalEdge],
    names: &HashMap<u32, String>,
) {
    for &cid in new_clusters {
        if graph.neighbors(cid).is_empty() && graph.node_count() < 20 {
            let name = names
                .get(&cid)
                .cloned()
                .unwrap_or_else(|| format!("Cluster {}", cid));
            graph.add_node(NodeWeight {
                id: cid,
                name,
                r#type: String::new(),
                evolution: String::new(),
                per_week_intents: Vec::new(),
            });
        }
    }
    for edge in incremental_relations {
        graph.add_edge(
            edge.from,
            edge.to,
            EdgeWeight {
                relation_type: edge.relation_type.clone(),
                logic: String::new(),
                weeks: vec!["incremental".to_string()],
                period_type: "incremental".to_string(),
            },
        );
    }
}

fn evaluate_segment(
    graph: &IntentGraph,
    segment: &FeedbackSegment,
) -> SegmentDetail {
    let expected: Vec<(u32, u32, String)> = segment
        .expected_relations
        .iter()
        .map(|er| (er.from, er.to, er.relation_type.clone()))
        .collect();

    let mut found_set: HashSet<(u32, u32, String)> = HashSet::new();
    for &cid in &segment.clusters {
        let nbs = graph.neighbors(cid);
        for nb in &nbs {
            found_set.insert((nb.from, nb.to, nb.relation.clone()));
        }
    }

    let found_count = expected
        .iter()
        .filter(|k| found_set.contains(k))
        .count();
    let found_relations: Vec<(u32, u32, String)> = expected
        .iter()
        .filter(|k| found_set.contains(k))
        .cloned()
        .collect();

    SegmentDetail {
        id: segment.id.clone(),
        clusters: segment.clusters.clone(),
        expected_relations: expected.clone(),
        found_relations,
        found_count,
        total_count: expected.len(),
    }
}

fn evaluate_week(
    graph: &IntentGraph,
    week: &FeedbackWeek,
) -> WeeklyResult {
    let mut segment_details = Vec::new();
    let mut total_expected = 0usize;
    let mut total_found = 0usize;

    for segment in &week.segments {
        let detail = evaluate_segment(graph, segment);
        total_expected += detail.total_count;
        total_found += detail.found_count;
        segment_details.push(detail);
    }

    let recall = if total_expected > 0 {
        total_found as f64 / total_expected as f64
    } else {
        0.0
    };

    WeeklyResult {
        week: week.week.clone(),
        condition: String::new(), // filled by caller
        total_expected,
        total_found,
        recall,
        segment_details,
    }
}

fn main() {
    let data_path = "examples/project-06/data/input/feedback-weekly-data.json";
    let intent_yaml = "assets/refined/intent.yaml";

    let feedback: FeedbackData =
        serde_json::from_str(&fs::read_to_string(data_path).expect("Failed to read feedback data"))
            .expect("Failed to parse feedback data");
    let names = load_cluster_names(intent_yaml);

    let snapshots: HashMap<String, &WeeklySnapshot> = feedback
        .weekly_snapshots
        .iter()
        .map(|s| (s.week.clone(), s))
        .collect();

    let w19_snapshot = snapshots.get("2026-W19").expect("W19 snapshot not found");

    // ---- Condition A: No Update (W19-only graph for all weeks) ----
    let seed_graph = build_weekly_graph(w19_snapshot, &names);
    let mut no_update_results = Vec::new();
    for week in &feedback.timeline {
        let mut result = evaluate_week(&seed_graph, week);
        result.condition = "no_update".to_string();
        no_update_results.push(result);
    }

    // ---- Condition B: With Update (incremental) ----
    let mut current_graph = build_weekly_graph(w19_snapshot, &names);
    let mut with_update_results = Vec::new();
    for week in &feedback.timeline {
        let mut result = evaluate_week(&current_graph, week);
        result.condition = "with_update".to_string();
        with_update_results.push(result);

        if let Some(snapshot) = snapshots.get(week.week.as_str()) {
            apply_incremental(
                &mut current_graph,
                &snapshot.new_clusters,
                &snapshot.incremental_relations,
                &names,
            );
        }
    }

    let report = ComparisonReport {
        no_update: no_update_results,
        with_update: with_update_results,
    };

    fs::create_dir_all("examples/project-06/data/output").ok();
    fs::write(
        "examples/project-06/data/output/feedback-result.json",
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .expect("Failed to write output");

    println!("=== Feedback Experiment (6.3) ===");
    println!();
    println!("Condition A: No Update (W19 seed graph only)");
    for r in &report.no_update {
        println!(
            "  {} : {}/{} = {:.1}%",
            r.week, r.total_found, r.total_expected, r.recall * 100.0
        );
    }
    println!();
    println!("Condition B: With Update (incremental weekly)");
    for r in &report.with_update {
        println!(
            "  {} : {}/{} = {:.1}%",
            r.week, r.total_found, r.total_expected, r.recall * 100.0
        );
    }
    println!();
    println!("Comparison:");
    for (nu, wu) in report.no_update.iter().zip(report.with_update.iter()) {
        let diff = wu.recall - nu.recall;
        let sign = if diff > 0.0 { "+" } else { "" };
        println!(
            "  {} : no_update={:.1}%  with_update={:.1}%  Δ={}{:.1}%",
            nu.week,
            nu.recall * 100.0,
            wu.recall * 100.0,
            sign,
            diff * 100.0
        );
    }
}
