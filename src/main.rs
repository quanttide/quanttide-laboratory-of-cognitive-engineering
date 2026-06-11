mod data;
mod transfer;
mod report;

use std::path::PathBuf;
use qtcloud_think_cli::repo::Repo;
use crate::data::{load_annotations, SchemaFile};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage:");
        eprintln!("  lab fill <domain> [--weeks W19,W21,...] [--annotations file.yaml]");
        eprintln!("  lab assess <schema.yaml>");
        std::process::exit(1);
    }

    // Determine journal path: try env var, then default
    let journal_path = std::env::var("JOURNAL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../data/journal")
                .canonicalize()
                .unwrap()
        });

    match args[1].as_str() {
        "fill" => cmd_fill(&args, &journal_path),
        "assess" => cmd_assess(&args),
        _ => eprintln!("Unknown command: {}", args[1]),
    }
}

fn cmd_fill(args: &[String], journal_path: &PathBuf) {
    let domain = &args[2];

    let mut weeks_filter: Option<Vec<String>> = None;
    let mut annotations_path: Option<PathBuf> = None;
    let mut i = 3;
    while i < args.len() {
        if args[i] == "--weeks" && i + 1 < args.len() {
            weeks_filter = Some(args[i+1].split(',').map(|w| format!("2026-{}", w)).collect());
            i += 2;
        } else if args[i] == "--annotations" && i + 1 < args.len() {
            annotations_path = Some(PathBuf::from(&args[i+1]));
            i += 2;
        } else { i += 1; }
    }

    let repo = Repo::open(journal_path);
    let world = "quanttide-founder";
    let annotations = annotations_path.as_ref().and_then(|p| load_annotations(p).ok());

    // Collect weeks
    let all_weeks = repo.periods(world).unwrap_or_default();
    let selected_weeks: Vec<&str> = if let Some(ref wf) = weeks_filter {
        all_weeks.iter().filter(|w| wf.contains(w)).map(|s| s.as_str()).collect()
    } else {
        all_weeks.iter().map(|s| s.as_str()).collect()
    };

    let mut weeks_data: Vec<qtcloud_think_cli::repo::DomainFile> = Vec::new();
    for week in &selected_weeks {
        if let Ok(df) = repo.load(world, week, domain) {
            weeks_data.push(df);
        }
    }

    if weeks_data.is_empty() {
        eprintln!("No data for domain '{}' in selected weeks", domain);
        std::process::exit(1);
    }

    let refs: Vec<&qtcloud_think_cli::repo::DomainFile> = weeks_data.iter().collect();
    let schema = transfer::fill_schema(&refs, annotations.as_ref());
    let output = serde_yaml::to_string(&SchemaFile { schemas: vec![schema] })
        .expect("serialization failed");
    println!("{}", output);
}

fn cmd_assess(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: lab assess <schema.yaml>");
        std::process::exit(1);
    }
    let path = PathBuf::from(&args[2]);
    let file = std::fs::File::open(&path).unwrap_or_else(|e| {
        eprintln!("Failed to open {}: {}", path.display(), e);
        std::process::exit(1);
    });
    let wrapper: SchemaFile = serde_yaml::from_reader(file).unwrap_or_else(|e| {
        eprintln!("YAML parse error: {}", e);
        std::process::exit(1);
    });
    let schema = wrapper.schemas.into_iter().next().unwrap_or_default();
    let assessment = report::assess(&schema);
    println!("{}", report::format_report(&assessment));
}
