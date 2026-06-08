use std::io::{self, BufRead};

use qtcloud_think_intent::{ScaffoldEngine, SessionManager};

fn main() -> Result<(), String> {
    let engine = ScaffoldEngine::new("data/formal/intent-graph.json")?;
    let sessions = SessionManager::new("data/formal/sessions");

    println!("=== qtcloud-think-intent ===");
    println!("Type your thoughts ('exit' to quit)\n");

    let stdin = io::stdin();
    let mut n = 0usize;

    for line in stdin.lock().lines() {
        let input = line.map_err(|e| e.to_string())?;
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" {
            break;
        }
        n += 1;
        let now = format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());

        let (parsed, raw) = engine.process(input)?;
        let turn = engine.build_turn(input, &parsed, &raw, &format!("{}_{}", now, n), &now);
        sessions.save_turn(&turn);

        println!("\n---");
        if !parsed.positioning.is_empty() { println!("📍 {}", parsed.positioning); }
        if !parsed.connections.is_empty() { println!("🔗 {}", parsed.connections); }
        if !parsed.exploration.is_empty() { println!("💡 {}", parsed.exploration); }
        println!("---\n");
    }

    Ok(())
}
