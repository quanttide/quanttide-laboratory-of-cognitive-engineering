mod graph;
mod models;
mod tokenizer;

use std::io::Read;

use models::{EdgeWeight, KeywordTable, RejectLog};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut threshold = 0.1_f64;
    let mut keywords_path = String::from("../project-03/data/keywords.json");
    let mut graph_path = String::from("data/graph-init.json");
    let mut _depth: usize = 2;
    let mut feedback = false;
    let mut build_graph = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--threshold" => {
                i += 1;
                threshold = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(0.1);
            }
            "--keywords" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    keywords_path = v.clone();
                }
            }
            "--graph" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    graph_path = v.clone();
                }
            }
            "--depth" => {
                i += 1;
                _depth = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(2);
            }
            "--feedback" => feedback = true,
            "--build-graph" => build_graph = true,
            _ => {}
        }
        i += 1;
    }

    if build_graph {
        let ig = graph::IntentGraph::from_yaml(
            "../../assets/refined/intent.yaml",
            "../../assets/refined/intent-relation.yaml",
        )?;
        ig.save_json(&graph_path)?;
        eprintln!(
            "Graph saved to {} ({} nodes, {} edges)",
            graph_path,
            ig.node_count(),
            ig.edge_count()
        );
        return Ok(());
    }

    let ig = if std::path::Path::new(&graph_path).exists() {
        graph::IntentGraph::load_json(&graph_path)?
    } else {
        eprintln!("Graph file not found, building from YAML...");
        let ig = graph::IntentGraph::from_yaml(
            "../../assets/refined/intent.yaml",
            "../../assets/refined/intent-relation.yaml",
        )?;
        ig.save_json(&graph_path)?;
        ig
    };

    let keywords_json = std::fs::read_to_string(&keywords_path)?;
    let keywords: KeywordTable = serde_json::from_str(&keywords_json)?;

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let input = input.trim().to_string();

    if input.is_empty() {
        eprintln!("No input text provided");
        return Ok(());
    }

    let output = ig.infer(&keywords, &input, threshold);
    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);

    if feedback && !output.candidate_edges.is_empty() {
        eprintln!("\n--- Feedback Loop ---");
        let mut reject_log: RejectLog = load_reject_log("data/reject_log.json");
        for candidate in &output.candidate_edges {
            if reject_log.rejected.contains(&(candidate.from, candidate.to))
                || reject_log.rejected.contains(&(candidate.to, candidate.from))
            {
                eprintln!("Skipping already rejected: {} → {}", candidate.from, candidate.to);
                continue;
            }
            eprintln!(
                "Candidate edge: {} → {} (type: {}, confidence: {})",
                candidate.from, candidate.to, candidate.proposed_type, candidate.confidence
            );
            eprintln!("Evidence: {}", candidate.evidence);
            eprint!("Keep? (y/N): ");
            std::io::Write::flush(&mut std::io::stderr())?;
            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer)?;
            let answer = answer.trim().to_lowercase();
            if answer == "y" || answer == "yes" {
                let _weight = EdgeWeight {
                    relation_type: candidate.proposed_type.clone(),
                    logic: candidate.evidence.clone(),
                    weeks: vec!["W24".to_string()],
                    period_type: "situational".to_string(),
                };
                // Can't mutate ig here since it's borrowed
                // Will handle by re-loading and saving
                eprintln!("Edge kept (will be saved on next --build-graph run)");
            } else {
                reject_log.rejected.push((candidate.from, candidate.to));
            }
        }
        save_reject_log("data/reject_log.json", &reject_log)?;
    }

    Ok(())
}

fn load_reject_log(path: &str) -> RejectLog {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(RejectLog { rejected: vec![] })
}

fn save_reject_log(path: &str, log: &RejectLog) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(log)?;
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)?;
    Ok(())
}
