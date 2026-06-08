use std::fs;

use crate::models::*;

pub fn compute_sample(
    entry: &TestSegment,
    baseline: &BaselineEntry,
    output_a: &serde_json::Value,
    output_b: &serde_json::Value,
) -> SampleOutput {
    let expected_clusters = &baseline.clusters;
    let expected_relations = &baseline.relations;

    let matched_a = crate::evaluator::get_matched_a(output_a);
    let matched_b = crate::evaluator::get_matched_b(output_b);

    let recall_a = crate::evaluator::recall(&matched_a, expected_clusters);
    let recall_b = crate::evaluator::recall(&matched_b, expected_clusters);
    let inc_nodes = crate::evaluator::incremental_nodes(&matched_a, &matched_b, expected_clusters);
    let inc_rels = crate::evaluator::incremental_relations(output_b, expected_clusters);
    let fp_a = crate::evaluator::false_positive_rate(&matched_a, expected_clusters);
    let fp_b = crate::evaluator::false_positive_rate(&matched_b, expected_clusters);

    let neighbors = crate::evaluator::parse_neighbors(output_b);
    let bfs_paths = crate::evaluator::parse_bfs_paths(output_b);
    let conflicts = crate::evaluator::parse_conflicts(output_b);
    let candidates = crate::evaluator::parse_candidates(output_b);
    let path_grades = crate::evaluator::build_path_grades(output_b);

    SampleOutput {
        id: entry.id.clone(),
        baseline: BaselineSummary {
            clusters: expected_clusters.clone(),
            relations: expected_relations.clone(),
        },
        approach_a: ApproachAOutput {
            matched: matched_a,
        },
        approach_b: ApproachBOutput {
            matched: matched_b,
            neighbors,
            bfs_paths,
            conflicts,
            candidate_edges: candidates,
        },
        metrics: Metrics {
            recall_a: (recall_a * 10000.0).round() / 10000.0,
            recall_b: (recall_b * 10000.0).round() / 10000.0,
            incremental_nodes: inc_nodes,
            incremental_relations: inc_rels,
            false_positive_a: (fp_a * 10000.0).round() / 10000.0,
            false_positive_b: (fp_b * 10000.0).round() / 10000.0,
        },
        path_grades,
    }
}

pub fn compute_summary(samples: &[SampleOutput]) -> Summary {
    let total = samples.len();
    let avg_recall_a = samples.iter().map(|s| s.metrics.recall_a).sum::<f64>() / total as f64;
    let avg_recall_b = samples.iter().map(|s| s.metrics.recall_b).sum::<f64>() / total as f64;
    let total_inc_nodes = samples.iter().map(|s| s.metrics.incremental_nodes).sum();
    let total_inc_rels = samples.iter().map(|s| s.metrics.incremental_relations).sum();
    let avg_fp_a = samples.iter().map(|s| s.metrics.false_positive_a).sum::<f64>() / total as f64;
    let avg_fp_b = samples.iter().map(|s| s.metrics.false_positive_b).sum::<f64>() / total as f64;

    let all_grades: Vec<u32> = samples
        .iter()
        .flat_map(|s| s.path_grades.iter())
        .filter_map(|pg| pg.grade)
        .collect();
    let avg_grade = if all_grades.is_empty() {
        None
    } else {
        Some(all_grades.iter().sum::<u32>() as f64 / all_grades.len() as f64)
    };

    Summary {
        total_samples: total,
        avg_recall_a: (avg_recall_a * 10000.0).round() / 10000.0,
        avg_recall_b: (avg_recall_b * 10000.0).round() / 10000.0,
        total_incremental_nodes: total_inc_nodes,
        total_incremental_relations: total_inc_rels,
        avg_false_positive_a: (avg_fp_a * 10000.0).round() / 10000.0,
        avg_false_positive_b: (avg_fp_b * 10000.0).round() / 10000.0,
        avg_path_grade: avg_grade.map(|g| (g * 10000.0).round() / 10000.0),
        caveat: "Test segments from W23 (used for graph construction), not truly unseen W24 data. Baseline is best-effort from intent analysis.".to_string(),
    }
}

pub fn write_json(output: &EvaluationOutput, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(output)?;
    if let Some(parent) = std::path::Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, json)?;
    Ok(())
}

pub fn read_test_set(path: &str) -> Result<Vec<TestSegment>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn read_baselines(path: &str) -> Result<Vec<BaselineEntry>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}
