use std::collections::HashSet;
use std::fs;

use intent_graph::graph::IntentGraph;
use serde::Serialize;

use project_06::{ExpectedEdge, ExpectedPath, ReasonTestCase};

#[derive(Serialize)]
struct CaseResult {
    id: String,
    input_clusters: Vec<u32>,
    expected_edges: Vec<ExpectedEdge>,
    found_edges: Vec<(u32, u32, String)>,
    edge_coverage: f64,
    expected_path: Option<ExpectedPath>,
    found_path: bool,
    false_positives: Vec<(u32, u32, String)>,
}

#[derive(Serialize)]
struct Report {
    total_cases: usize,
    overall_edge_coverage: f64,
    overall_false_positive_rate: f64,
    path_accuracy: f64,
    cases: Vec<CaseResult>,
}

fn relation_key(from: u32, to: u32, rtype: &str) -> (u32, u32, String) {
    (from, to, rtype.to_string())
}

fn neighbor_relation_key(n: &intent_graph::models::NeighborInfo) -> String {
    n.relation.clone()
}

fn main() {
    let data_path = "examples/project-06/data/input/reason-test-set.json";
    let intent_yaml = "assets/refined/intent.yaml";
    let relation_yaml = "assets/refined/intent-relation.yaml";

    let test_data: Vec<ReasonTestCase> =
        serde_json::from_str(&fs::read_to_string(data_path).expect("Failed to read test data"))
            .expect("Failed to parse test data");

    let graph = IntentGraph::load(intent_yaml, relation_yaml).expect("Failed to build graph");

    let mut case_results = Vec::new();
    let mut total_expected = 0usize;
    let mut total_found = 0usize;
    let mut total_false_positives = 0usize;
    let mut total_paths_expected = 0usize;
    let mut total_paths_found = 0usize;

    for case in &test_data {
        let expected_set: HashSet<(u32, u32, String)> = case
            .expected_direct_edges
            .iter()
            .map(|e| relation_key(e.from, e.to, &e.relation_type))
            .collect();

        let mut found_edges: Vec<(u32, u32, String)> = Vec::new();
        let mut found_set: HashSet<(u32, u32, String)> = HashSet::new();

        match case.query_type.as_str() {
            "neighbors" => {
                for &cid in &case.input_clusters {
                    let nbs = graph.neighbors(cid);
                    for nb in &nbs {
                        let rel = neighbor_relation_key(nb);
                        let key = relation_key(nb.from, nb.to, &rel);
                        if found_set.insert(key.clone()) {
                            found_edges.push(key);
                        }
                    }
                }
            }
            "path" => {
                if case.input_clusters.len() >= 2 {
                    let a = case.input_clusters[0];
                    let b = case.input_clusters[1];
                    let paths_a = graph.bfs(a, 5);
                    let paths_b = graph.bfs(b, 5);

                    for path in &paths_a {
                        for step in path {
                            let key = relation_key(step.from, step.to, &step.relation);
                            if found_set.insert(key.clone()) {
                                found_edges.push(key);
                            }
                        }
                    }
                    for path in &paths_b {
                        for step in path {
                            let key = relation_key(step.from, step.to, &step.relation);
                            if found_set.insert(key.clone()) {
                                found_edges.push(key);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        let found_expected = expected_set
            .iter()
            .filter(|k| found_set.contains(k))
            .count();
        let false_positives: Vec<(u32, u32, String)> = found_edges
            .iter()
            .filter(|k| !expected_set.contains(k))
            .cloned()
            .collect();

        // Check if expected path exists
        let path_found = if let Some(ep) = &case.expected_path {
            if case.input_clusters.len() < 2 {
                true
            } else {
                let a = case.input_clusters[0];
                let b = case.input_clusters[1];
                // Try BFS from both directions (graph is directed, but relations are undirected)
                let paths_a = graph.bfs(a, 5);
                let paths_b = graph.bfs(b, 5);
                let reaches = |paths: &[Vec<intent_graph::models::PathStep>], target: u32| -> bool {
                    for path in paths {
                        if let Some(last) = path.last() {
                            if last.to == target {
                                return true;
                            }
                        }
                    }
                    false
                };
                reaches(&paths_a, b) || reaches(&paths_b, a)
            }
        } else {
            true
        };

        let edge_coverage = if !expected_set.is_empty() {
            found_expected as f64 / expected_set.len() as f64
        } else {
            1.0
        };

        total_expected += expected_set.len();
        total_found += found_expected;
        total_false_positives += false_positives.len();
        if case.expected_path.is_some() {
            total_paths_expected += 1;
            if path_found {
                total_paths_found += 1;
            }
        }

        case_results.push(CaseResult {
            id: case.id.clone(),
            input_clusters: case.input_clusters.clone(),
            expected_edges: case.expected_direct_edges.clone(),
            found_edges: found_edges.clone(),
            edge_coverage,
            expected_path: case.expected_path.clone(),
            found_path: path_found,
            false_positives,
        });
    }

    let overall_coverage = if total_expected > 0 {
        total_found as f64 / total_expected as f64
    } else {
        1.0
    };
    let total_actual_found: usize = case_results.iter().map(|c| c.found_edges.len()).sum();
    let fpr = if total_actual_found > 0 {
        total_false_positives as f64 / total_actual_found as f64
    } else {
        0.0
    };
    let path_accuracy = if total_paths_expected > 0 {
        total_paths_found as f64 / total_paths_expected as f64
    } else {
        1.0
    };

    let report = Report {
        total_cases: test_data.len(),
        overall_edge_coverage: overall_coverage,
        overall_false_positive_rate: fpr,
        path_accuracy,
        cases: case_results,
    };

    fs::write(
        "examples/project-06/data/output/reason-result.json",
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .expect("Failed to write output");

    println!("=== Reason Experiment (6.2) ===");
    println!("Cases:          {}", report.total_cases);
    println!("Edge coverage:  {:.1}%", report.overall_edge_coverage * 100.0);
    println!("False positive: {:.1}%", report.overall_false_positive_rate * 100.0);
    println!("Path accuracy:  {:.1}%", report.path_accuracy * 100.0);
    println!();
    for c in &report.cases {
        let found_ids: Vec<(u32, u32)> = c.found_edges.iter().map(|(f, t, _)| (*f, *t)).collect();
        println!(
            "  {} input={:?} edge_cov={:.0}% path={} fp={}",
            c.id, c.input_clusters, c.edge_coverage * 100.0,
            if c.found_path { "✓" } else { "✗" },
            c.false_positives.len()
        );
    }
}
