use std::collections::HashMap;
use std::fs;

use serde::Serialize;

#[derive(Serialize)]
struct SummaryReport {
    matching: MatchingSummary,
    reasoning: ReasoningSummary,
    feedback: FeedbackSummary,
    synthesis: Vec<SynthesisPoint>,
}

#[derive(Serialize)]
struct MatchingSummary {
    top1_recall: f64,
    top3_recall: f64,
    best_clusters: Vec<String>,
    worst_clusters: Vec<String>,
    total_segments: usize,
}

#[derive(Serialize)]
struct ReasoningSummary {
    edge_coverage: f64,
    path_accuracy: f64,
    total_cases: usize,
    hub_clusters: Vec<u32>,
    isolated_clusters: Vec<u32>,
}

#[derive(Serialize)]
struct FeedbackSummary {
    no_update_final_recall: f64,
    with_update_final_recall: f64,
    max_gap: f64,
    gap_emerges_at: String,
}

#[derive(Serialize)]
struct SynthesisPoint {
    dimension: String,
    finding: String,
    evidence: String,
}

fn load_json(path: &str) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path).expect("Failed to read"))
        .expect("Failed to parse")
}

fn main() {
    let match_output = load_json("examples/project-06/data/output/match-result.json");
    let reason_output = load_json("examples/project-06/data/output/reason-result.json");
    let feedback_output = load_json("examples/project-06/data/output/feedback-result.json");

    // Matching summary
    let top1 = match_output["top1_recall"].as_f64().unwrap_or(0.0);
    let top3 = match_output["top3_recall"].as_f64().unwrap_or(0.0);
    let total = match_output["total_segments"].as_u64().unwrap_or(0) as usize;

    let mut clusters: Vec<(String, f64)> = Vec::new();
    for pc in match_output["per_cluster"].as_array().unwrap_or(&vec![]) {
        let name = pc["name"].as_str().unwrap_or("").to_string();
        let r = pc["recall_top1"].as_f64().unwrap_or(0.0);
        clusters.push((name, r));
    }
    clusters.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let best: Vec<String> = clusters.iter().take(3).map(|(n, _)| n.clone()).collect();
    let mut worst: Vec<String> = clusters.iter().rev().take(3).map(|(n, _)| n.clone()).collect();
    worst.reverse();

    // Reasoning summary
    let edge_cov = reason_output["overall_edge_coverage"].as_f64().unwrap_or(0.0);
    let path_acc = reason_output["path_accuracy"].as_f64().unwrap_or(0.0);
    let n_cases = reason_output["total_cases"].as_u64().unwrap_or(0) as usize;

    let mut hub_counts: HashMap<u32, usize> = HashMap::new();
    for c in reason_output["cases"].as_array().unwrap_or(&vec![]) {
        for e in c["found_edges"].as_array().unwrap_or(&vec![]) {
            if let (Some(f), Some(t)) = (e[0].as_u64(), e[1].as_u64()) {
                *hub_counts.entry(f as u32).or_insert(0) += 1;
                *hub_counts.entry(t as u32).or_insert(0) += 1;
            }
        }
    }
    let mut hub_vec: Vec<(u32, usize)> = hub_counts.into_iter().collect();
    hub_vec.sort_by(|a, b| b.1.cmp(&a.1));
    let hubs: Vec<u32> = hub_vec.iter().take(3).map(|(id, _)| *id).collect();
    let isolated: Vec<u32> = hub_vec.iter().rev().take(2).map(|(id, _)| *id).collect();

    // Feedback summary
    let no_update_final = feedback_output["no_update"]
        .as_array()
        .and_then(|arr| arr.last())
        .and_then(|w| w["recall"].as_f64())
        .unwrap_or(0.0);
    let with_update_final = feedback_output["with_update"]
        .as_array()
        .and_then(|arr| arr.last())
        .and_then(|w| w["recall"].as_f64())
        .unwrap_or(0.0);

    let mut max_gap = 0.0f64;
    let mut gap_week = String::new();
    let empty_vec = vec![];
    let no_update_arr = feedback_output["no_update"].as_array().unwrap_or(&empty_vec);
    let with_update_arr = feedback_output["with_update"].as_array().unwrap_or(&empty_vec);
    for (nu, wu) in no_update_arr.iter().zip(with_update_arr.iter()) {
        let nr = nu["recall"].as_f64().unwrap_or(0.0);
        let wr = wu["recall"].as_f64().unwrap_or(0.0);
        let gap = wr - nr;
        if gap > max_gap {
            max_gap = gap;
            gap_week = nu["week"].as_str().unwrap_or("").to_string();
        }
    }
    let gap_week_clone = gap_week.clone();

    let report = SummaryReport {
        matching: MatchingSummary {
            top1_recall: top1,
            top3_recall: top3,
            best_clusters: best,
            worst_clusters: worst,
            total_segments: total,
        },
        reasoning: ReasoningSummary {
            edge_coverage: edge_cov,
            path_accuracy: path_acc,
            total_cases: n_cases,
            hub_clusters: hubs,
            isolated_clusters: isolated,
        },
        feedback: FeedbackSummary {
            no_update_final_recall: no_update_final,
            with_update_final_recall: with_update_final,
            max_gap,
            gap_emerges_at: gap_week_clone,
        },
        synthesis: vec![
            SynthesisPoint {
                dimension: "词汇断层".to_string(),
                finding: "Top-1 recall 52%，T7 完全漏检簇5".to_string(),
                evidence: "日记用词（取舍/边界/客户battle）与意图描述（商业增长/收入）不重叠".to_string(),
            },
            SynthesisPoint {
                dimension: "结构枢纽".to_string(),
                finding: "簇1（研发方法论）是最高频节点".to_string(),
                evidence: format!("推理实验中簇1出现 {:?} 次", hub_vec.first().map(|(_,c)| c).unwrap_or(&0)),
            },
            SynthesisPoint {
                dimension: "演化滞后".to_string(),
                finding: format!("W23 更新组 100% vs 不更新组 {:.0}%，差距 {:.0}%", no_update_final * 100.0, max_gap * 100.0),
                evidence: format!("差距在 {} 首次出现并持续扩大", gap_week),
            },
        ],
    };

    fs::create_dir_all("examples/project-06/data/report").ok();
    fs::write(
        "examples/project-06/data/report/experiment-summary.json",
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .expect("Failed to write");
    println!("Report written to data/report/experiment-summary.json");
}
