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
        println!("  intentions [week] [name]  - list intentions");
        println!("  intention <id>           - show intention detail");
        println!("  filter <options>         - filter intentions");
        println!("  trace <title>            - find intention across weeks");
        println!("  drift <weekA> <weekB> <name> - compare priority/risk shift");
        println!("  evolve <name>            - intention evolution table");
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
                    println!("  intentions [week] [name]  - list intentions");
                    println!("  intention <id>           - show intention detail");
                    println!("  filter <options>         - filter intentions");
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
                "intentions" => {
                    let first = parts.get(1).map(|s| *s);
                    // if 1st arg is a week pattern, use as week; otherwise as situation name
                    let (w, n) = if let Some(v) = first {
                        if v.starts_with("2026-") {
                            (Some(v), parts.get(2).map(|s| *s))
                        } else {
                            (None, Some(v))
                        }
                    } else {
                        (None, None)
                    };
                    if let Some(w) = w {
                        match reporter.list_intentions(w, n) {
                            Ok(s) => println!("{}", s),
                            Err(e) => println!("Error: {}", e),
                        }
                    } else {
                        // no args: list all intentions across all weeks
                        match reporter.filter_intentions(None, None, None, None, None, None) {
                            Ok(s) => println!("{}", s),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                }
                "intention" => {
                    if parts.len() < 2 {
                        println!("Usage: intention <id>");
                        continue;
                    }
                    match reporter.show_intention(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "trace" => {
                    if parts.len() < 2 {
                        println!("Usage: trace <title>");
                        continue;
                    }
                    let title = parts[1..].join(" ");
                    match reporter.trace(&title) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "drift" => {
                    if parts.len() < 3 {
                        println!("Usage: drift <weekA> <weekB> <sit_name>");
                        continue;
                    }
                    let sit_name = parts.get(3).map(|s| *s).unwrap_or("");
                    match reporter.drift(parts[1], parts[2], sit_name) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "evolve" => {
                    if parts.len() < 2 {
                        println!("Usage: evolve <sit_name>");
                        continue;
                    }
                    match reporter.evolution_table(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "relate-intentions" | "ri" => {
                    if parts.len() < 2 {
                        println!("Usage: relate-intentions <week>");
                        continue;
                    }
                    match reporter.relate_intentions_llm(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "intention-report" | "ir" => {
                    if parts.len() < 2 {
                        println!("Usage: intention-report <week>");
                        continue;
                    }
                    match reporter.intention_report(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "coverage" => {
                    if parts.len() < 2 {
                        println!("Usage: coverage <week>");
                        continue;
                    }
                    match reporter.coverage(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "tension" => {
                    if parts.len() < 2 {
                        println!("Usage: tension <week>");
                        continue;
                    }
                    match reporter.tension(parts[1]) {
                        Ok(s) => println!("{}", s),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "filter" => {
                    let mut week = None;
                    let mut sit_name = None;
                    let mut priority = None;
                    let mut risk = None;
                    let mut level = None;
                    let mut agent = None;
                    let mut i = 1;
                    while i < parts.len() {
                        match parts[i] {
                            "--week" => { i += 1; week = parts.get(i).map(|s| s.to_string()); }
                            "--sit" => { i += 1; sit_name = parts.get(i).map(|s| s.to_string()); }
                            "--priority" => { i += 1; priority = parts.get(i).map(|s| s.to_string()); }
                            "--risk" => { i += 1; risk = parts.get(i).map(|s| s.to_string()); }
                            "--level" => { i += 1; level = parts.get(i).map(|s| s.to_string()); }
                            "--agent" => { i += 1; agent = parts.get(i).map(|s| s.to_string()); }
                            _ => {}
                        }
                        i += 1;
                    }
                    match reporter.filter_intentions(
                        week.as_deref(),
                        sit_name.as_deref(),
                        priority.as_deref(),
                        risk.as_deref(),
                        level.as_deref(),
                        agent.as_deref(),
                    ) {
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
