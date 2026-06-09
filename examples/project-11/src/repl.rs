use std::io::{self, BufRead};

use crate::report::ReportGenerator;
use crate::query::QueryEngine;

pub struct Repl {
    engine: QueryEngine,
    gallery_path: String,
}

impl Repl {
    pub fn new(engine: QueryEngine, gallery_path: String) -> Self {
        Self { engine, gallery_path }
    }

    pub fn run(&self) -> Result<(), String> {
        let reporter = ReportGenerator::new(QueryEngine::new(&self.gallery_path));

        println!("=== Project 11: Situation Engine ===");
        println!("Gallery: {}", self.gallery_path);
        println!("Commands:");
        println!("  weeks                    - list available weeks");
        println!("  show <week>              - show week summary");
        println!("  landscape <week>         - show week landscape (compact)");
        println!("  explore <name>           - track situation evolution across weeks");
        println!("  registry                 - show situation registry");
        println!("  report <week>            - generate structured weekly report");
        println!("  diff <weekA> <weekB>      - compare two weeks");
        println!("  relate <week>            - LLM infer situation relations");
        println!("  exit                     - quit\n");

        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.map_err(|e| e.to_string())?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts[0] {
                "exit" | "quit" => break,
                "help" => {
                    println!("Commands:");
                    println!("  weeks                    - list available weeks");
                    println!("  show <week>              - show week summary");
                    println!("  landscape <week>         - show week landscape (compact)");
                    println!("  explore <name>           - track situation evolution across weeks");
                    println!("  registry                 - show situation registry");
                    println!("  report <week>            - generate structured weekly report");
                    println!("  diff <weekA> <weekB>      - compare two weeks");
                    println!("  relate <week>            - LLM infer situation relations");
                    println!("  exit                     - quit");
                }
                "weeks" => match self.engine.list_weeks() {
                    Ok(weeks) => {
                        println!("Available weeks:");
                        for w in weeks {
                            println!("  {}", w);
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                },
                "registry" => match self.engine.registry() {
                    Ok(reg) => {
                        println!("Situation Registry:");
                        for r in reg {
                            println!("  {}: {}", r.name, r.label);
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                },
                "show" => {
                    if parts.len() < 2 {
                        println!("Usage: show <week>");
                        continue;
                    }
                    match reporter.summary(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "landscape" => {
                    if parts.len() < 2 {
                        println!("Usage: landscape <week>");
                        continue;
                    }
                    match reporter.landscape(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "explore" => {
                    if parts.len() < 2 {
                        println!("Usage: explore <name>");
                        continue;
                    }
                    match reporter.evolution(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "report" => {
                    if parts.len() < 2 {
                        println!("Usage: report <week>");
                        continue;
                    }
                    match reporter.report(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "diff" => {
                    if parts.len() < 3 {
                        println!("Usage: diff <weekA> <weekB>");
                        continue;
                    }
                    match reporter.diff(parts[1], parts[2]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "relate" => {
                    if parts.len() < 2 {
                        println!("Usage: relate <week>");
                        continue;
                    }
                    match reporter.relate_llm(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                _ => {
                    println!("Unknown command: {}", line);
                }
            }
        }
        Ok(())
    }
}
