use std::io::{self, BufRead};

use qtcloud_think_intent::{DiscoveryState, ScaffoldEngine, SessionManager, Turn};

fn ts() -> String {
    format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
}

fn main() -> Result<(), String> {
    let engine = ScaffoldEngine::new("data/formal/intent-graph.json")?;
    let sessions = SessionManager::new("examples/project-10/data");

    println!("=== project-10: 意图即母题 — 跨簇共鸣 ===");
    println!("输入你的想法，脚手架会识别它在变奏哪个已有的关切。\n");

    let stdin = io::stdin();
    let mut session = sessions.load_or_create();
    let mut state = DiscoveryState::new();
    let mut n = session.turns.len();

    for line in stdin.lock().lines() {
        let input = line.map_err(|e| e.to_string())?;
        let input = input.trim();
        if input.is_empty() { continue; }
        if input == "exit" { break; }

        n += 1;
        println!("  → 正在扫描全图，追踪潜在关切...");

        let matches = engine.match_with_history(input, &state);
        let state_before = state.clone();
        let (parsed, raw) = engine.process_with_state(input, &state)?;
        state.merge(&parsed.discovery_update);
        let turn = Turn {
            id: format!("{}_{}", ts(), n),
            timestamp: ts(),
            input: input.to_string(),
            matched_clusters: matches,
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

        if let Some(ref m) = parsed.motif {
            if !m.motif_statement.is_empty() {
                if m.is_new_motif {
                    println!("🆕 新母题：{}", m.motif_statement);
                } else {
                    println!("🎵 发现母题：{}", m.motif_statement);
                    println!("   变奏：");
                    for v in &m.variations {
                        println!("   簇{} ({}): {}", v.cluster_id, v.week, v.form);
                    }
                }
            }
            if !m.motif_arc.is_empty() {
                println!("   演化弧：{}", m.motif_arc);
            }
        }
        println!("---\n");
    }

    Ok(())
}
