mod data;
mod transfer;
mod report;

use std::path::PathBuf;
use data::{load_journal, load_annotations, JournalSchema};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage:");
        eprintln!("  lab fill <domain> [--weeks W19,W21,...] [--annotations file.yaml]");
        eprintln!("  lab assess <schema.yaml>");
        std::process::exit(1);
    }

    let journal_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("journal-ingest.json");

    match args[1].as_str() {
        "fill" => cmd_fill(&args, &journal_path),
        "assess" => cmd_assess(&args),
        _ => eprintln!("Unknown command: {}", args[1]),
    }
}

fn cmd_fill(args: &[String], journal_path: &PathBuf) {
    let domain = &args[2];

    // Parse optional flags
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
        } else {
            i += 1;
        }
    }

    let journal = match load_journal(journal_path) {
        Ok(j) => j,
        Err(e) => { eprintln!("Failed to load journal: {}", e); std::process::exit(1); }
    };

    let annotations = annotations_path.as_ref().and_then(|p| load_annotations(p).ok());

    // Collect weeks data for domain
    let mut weeks_data: Vec<&data::JournalDomain> = Vec::new();
    for (week_name, domains) in &journal.weeks {
        if let Some(ref wf) = weeks_filter {
            if !wf.contains(week_name) {
                continue;
            }
        }
        if let Some(jd) = domains.get(domain) {
            weeks_data.push(jd);
        }
    }

    if weeks_data.is_empty() {
        eprintln!("No data for domain '{}'", domain);
        std::process::exit(1);
    }

    let schema = transfer::fill_schema(&weeks_data, annotations.as_ref());
    let output = serde_yaml::to_string(&JournalSchemaOutput {
        schemas: vec![schema],
    }).expect("serialization failed");
    println!("{}", output);
}

fn cmd_assess(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: lab assess <schema.yaml>");
        std::process::exit(1);
    }
    let path = PathBuf::from(&args[2]);
    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => { eprintln!("Failed to open {}: {}", path.display(), e); std::process::exit(1); }
    };
    let wrapper: JournalSchemaOutput = match serde_yaml::from_reader(file) {
        Ok(w) => w,
        Err(e) => { eprintln!("YAML parse error: {}", e); std::process::exit(1); }
    };
    let schema = wrapper.schemas.into_iter().next()
        .unwrap_or_else(|| JournalSchema {
            usage: None, entities: None, causals: None, boundaries: None,
            properties: None, dynamics: None, mappings: None, biases: None,
        });
    let assessment = report::assess(&schema);
    println!("{}", report::format_report(&assessment));
}

/// Output wrapper for schema YAML.
#[derive(serde::Serialize, serde::Deserialize)]
struct JournalSchemaOutput {
    schemas: Vec<JournalSchema>,
}
