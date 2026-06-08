mod evaluator;
mod models;
mod reporter;

use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let pwd = std::env::current_dir()?;

    let mut bin_a = pwd.join("..").join("project-03").join("target").join("debug").join("project-03");
    let mut bin_b = pwd.join("..").join("project-04").join("target").join("debug").join("project-04");
    let mut keywords = pwd.join("..").join("project-03").join("data").join("keywords.json");
    let mut graph = pwd.join("..").join("project-04").join("data").join("graph-init.json");

    let mut test_set_path = pwd.join("data").join("test-set.json");
    let mut baseline_path = pwd.join("data").join("baseline-w24.json");
    let mut output_json_path = pwd.join("outputs").join("evaluation.json");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--bin-a" => {
                i += 1;
                bin_a = args[i].clone().into();
            }
            "--bin-b" => {
                i += 1;
                bin_b = args[i].clone().into();
            }
            "--keywords" => {
                i += 1;
                keywords = args[i].clone().into();
            }
            "--graph" => {
                i += 1;
                graph = args[i].clone().into();
            }
            "--test-set" => {
                i += 1;
                test_set_path = args[i].clone().into();
            }
            "--baseline" => {
                i += 1;
                baseline_path = args[i].clone().into();
            }
            "--output" => {
                i += 1;
                output_json_path = args[i].clone().into();
            }
            _ => {}
        }
        i += 1;
    }

    let test_set = reporter::read_test_set(&test_set_path.to_string_lossy())?;
    let baselines: Vec<models::BaselineEntry> =
        reporter::read_baselines(&baseline_path.to_string_lossy())?;
    let baseline_map: HashMap<String, models::BaselineEntry> =
        baselines.into_iter().map(|b| (b.id.clone(), b)).collect();

    let mut samples = Vec::new();

    for entry in &test_set {
        let tid = &entry.id;
        let baseline = match baseline_map.get(tid) {
            Some(b) => b,
            None => {
                eprintln!("WARNING: No baseline for {}, skipping", tid);
                continue;
            }
        };

        eprintln!("Processing {}...", tid);

        let common_args: Vec<String> = vec![
            "--keywords".to_string(),
            keywords.to_string_lossy().to_string(),
        ];

        let output_a = evaluator::run_binary_stdin(
            &bin_a.to_string_lossy(),
            &common_args,
            &entry.segment,
        );
        let output_a_val = match output_a {
            Ok(v) => v,
            Err(e) => {
                eprintln!("  Scheme A error: {}", e);
                serde_json::json!({"matched": []})
            }
        };

        let mut b_args = common_args.clone();
        b_args.push("--graph".to_string());
        b_args.push(graph.to_string_lossy().to_string());

        let output_b = evaluator::run_binary_stdin(
            &bin_b.to_string_lossy(),
            &b_args,
            &entry.segment,
        );
        let output_b_val = match output_b {
            Ok(v) => v,
            Err(e) => {
                eprintln!("  Scheme B error: {}", e);
                serde_json::json!({"match_nodes": [], "neighbors": [], "bfs_paths": [], "conflicts": [], "candidate_edges": []})
            }
        };

        let sample = reporter::compute_sample(entry, baseline, &output_a_val, &output_b_val);
        samples.push(sample);
    }

    if samples.is_empty() {
        eprintln!("ERROR: No samples processed");
        std::process::exit(1);
    }

    let summary = reporter::compute_summary(&samples);
    let output = models::EvaluationOutput { samples, summary };

    reporter::write_json(&output, &output_json_path.to_string_lossy())?;
    eprintln!(
        "Evaluation complete: {}",
        output_json_path.to_string_lossy()
    );

    println!("\nQuick summary:");
    println!("  Samples: {}", output.summary.total_samples);
    println!("  Avg recall A: {:.2}%", output.summary.avg_recall_a * 100.0);
    println!("  Avg recall B: {:.2}%", output.summary.avg_recall_b * 100.0);
    println!(
        "  Total incremental nodes: {}",
        output.summary.total_incremental_nodes
    );
    println!(
        "  Total incremental relations: {}",
        output.summary.total_incremental_relations
    );
    println!(
        "  Avg FP A: {:.2}%",
        output.summary.avg_false_positive_a * 100.0
    );
    println!(
        "  Avg FP B: {:.2}%",
        output.summary.avg_false_positive_b * 100.0
    );

    Ok(())
}
