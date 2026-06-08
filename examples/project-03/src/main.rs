mod keyword_builder;
mod matcher;
mod models;
mod tokenizer;

use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut threshold = 0.1_f64;
    let mut keywords_path = String::from("data/keywords.json");
    let mut yaml_path: Option<String> = None;

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
            "--yaml" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    yaml_path = Some(v.clone());
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(yaml) = yaml_path {
        let table = keyword_builder::build_keyword_table(&yaml)?;
        keyword_builder::save_keyword_table(&table, &keywords_path)?;
        eprintln!("Keywords saved to {}", keywords_path);
        return Ok(());
    }

    let keywords_json = std::fs::read_to_string(&keywords_path)?;
    let keywords: models::KeywordTable = serde_json::from_str(&keywords_json)?;

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let input = input.trim();

    let result = matcher::match_text(input, &keywords, threshold);
    let output = serde_json::to_string_pretty(&result)?;
    println!("{}", output);

    Ok(())
}
