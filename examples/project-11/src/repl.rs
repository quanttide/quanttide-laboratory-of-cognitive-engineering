use std::io::{self, BufRead};

use crate::report::ReportGenerator;
use crate::query::QueryEngine;

pub struct Repl {
    engine: QueryEngine,
}

impl Repl {
    pub fn new(engine: QueryEngine) -> Self {
        Self { engine }
    }

    pub fn run(&self) -> Result<(), String> {
        let reporter = ReportGenerator::new(QueryEngine::new(
            "/home/iguo/repos/quanttide/domains/quanttide-think/docs/gallery",
        ));

        println!("=== Project 11: Situation Engine ===");
        println!("Commands:");
        println!("  weeks                    - list available weeks");
        println!("  show <week>              - show week summary");
        println!("  landscape <week>         - show week landscape (compact)");
        println!("  explore <name>           - track situation evolution across weeks");
        println!("  registry                 - show situation registry");
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
                "weeks" => {
                    match self.engine.list_weeks() {
                        Ok(weeks) => {
                            println!("Available weeks:");
                            for w in weeks {
                                println!("  {}", w);
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "registry" => {
                    match self.engine.registry() {
                        Ok(reg) => {
                            println!("Situation Registry:");
                            for r in reg {
                                println!("  {}: {}", r.name, r.label);
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "show" => {
                    if parts.len() < 2 {
                        println!("Usage: show <week>");
                        continue;
                    }
                    let week = parts[1];
                    match reporter.summary(week) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "landscape" => {
                    if parts.len() < 2 {
                        println!("Usage: landscape <week>");
                        continue;
                    }
                    let week = parts[1];
                    match reporter.landscape(week) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "explore" => {
                    if parts.len() < 2 {
                        println!("Usage: explore <name>");
                        continue;
                    }
                    let name = parts[1];
                    match reporter.evolution(name) {
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
