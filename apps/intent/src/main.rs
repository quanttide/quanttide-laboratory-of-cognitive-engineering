use std::io::{self, BufRead};

use qtcloud_think_intent::{DiscoveryState, ScaffoldEngine, SessionManager, Turn};

const MAX_TURNS: usize = 16;

fn main() -> Result<(), String> {
    let engine = ScaffoldEngine::new("data/formal/intent-graph.json")?;
    let sessions = SessionManager::new("data/formal/sessions");

    println!("=== qtcloud-think-intent ===");
    println!("Type your thoughts ('exit' to quit)\n");

    let stdin = io::stdin();
    let mut state = DiscoveryState::new();
    let mut session = sessions.load_or_create();
    let mut n = 0usize;
    let mut stale = 0usize;

    for line in stdin.lock().lines() {
        if n >= MAX_TURNS {
            println!("  已到达最大轮次 {}, 结束。", MAX_TURNS);
            break;
        }
        let input = line.map_err(|e| e.to_string())?;
        let input = input.trim();
        if input.is_empty() { continue; }
        if input == "exit" { break; }

        n += 1;
        let now = ts();

        let (parsed, raw) = engine.process_with_state(input, &state)?;

        let matched = engine.match_with_history(input, &state);
        let state_before = state.clone();
        let has_new = !parsed.discovery_update.new_clusters.is_empty()
            || !parsed.discovery_update.new_node_ids.is_empty()
            || !parsed.discovery_update.new_edge_ids.is_empty();
        if has_new { stale = 0; } else { stale += 1; }

        state.merge(&parsed.discovery_update);

        let turn = Turn {
            id: format!("{}_{}", now, n),
            timestamp: now,
            input: input.to_string(),
            matched_clusters: matched,
            state_before,
            state_after: state.clone(),
            llm_raw: raw,
            parsed: parsed.clone(),
        };
        session.turns.push(turn);
        sessions.save(&session);

        println!("\n---");
        if !parsed.positioning.is_empty() { println!("📍 {}", parsed.positioning); }
        if !parsed.connections.is_empty() { println!("🔗 {}", parsed.connections); }
        if !parsed.exploration.is_empty() { println!("💡 {}", parsed.exploration); }
        if has_new {
            println!("🆕 簇: {:?}  边: {:?}  洞察: {}",
                parsed.discovery_update.new_clusters,
                parsed.discovery_update.new_edge_ids,
                parsed.discovery_update.new_insights.len());
        }
        if !state.open_questions.is_empty() {
            println!("❓ {}", state.open_questions.join(" | "));
        }
        println!("---\n");

        if stale >= 2 {
            println!("  连续 2 轮无新发现，探索收敛。\n");
            break;
        }
    }

    println!("\n=== 总结 ===");
    println!("总轮次: {}", n);
    let clusters: Vec<u32> = session.turns.iter().flat_map(|t| t.parsed.discovery_update.new_clusters.clone()).collect();
    let edges: Vec<u32> = session.turns.iter().flat_map(|t| t.parsed.discovery_update.new_edge_ids.clone()).collect();
    let insights: usize = session.turns.iter().flat_map(|t| t.parsed.discovery_update.new_insights.iter()).count();
    println!("探索: {} 簇, {} 边, {} 洞察", clusters.len(), edges.len(), insights);

    Ok(())
}

fn ts() -> String {
    format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
}
