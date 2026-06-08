use std::io::{self, BufRead};

use project_09::{DiscoveryState, MultiTurnEngine, SessionManager, Turn};

const MAX_TURNS: usize = 8;

fn main() -> Result<(), String> {
    let engine = MultiTurnEngine::new("data/formal/intent-graph.json")?;
    let sessions = SessionManager::new("data/formal/sessions");

    println!("=== project-09: Multi-turn GraphRAG Scaffold ===");
    println!("MAX_TURNS={}", MAX_TURNS);
    println!("Type your thoughts ('exit' to quit)\n");

    let stdin = io::stdin();
    let mut state = DiscoveryState::new();
    let mut session = sessions.load_or_create();
    let mut n = 0usize;
    let mut stale_rounds = 0usize;

    for line in stdin.lock().lines() {
        if n >= MAX_TURNS {
            println!("  已到达最大轮次 {}，结束。", MAX_TURNS);
            break;
        }

        let input = line.map_err(|e| e.to_string())?;
        let input = input.trim();
        if input.is_empty() { continue; }
        if input == "exit" { break; }

        n += 1;
        let now = ts();

        println!("  → Processing turn {}...", n);
        let (parsed, raw) = engine.process(input, &state)?;

        let matched = engine.match_with_history(input, &state);
        let state_before = state.clone();
        let has_new = !parsed.discovery_update.new_clusters.is_empty()
            || !parsed.discovery_update.new_node_ids.is_empty()
            || !parsed.discovery_update.new_edge_ids.is_empty();
        if has_new { stale_rounds = 0; } else { stale_rounds += 1; }

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
            println!("🆕 新发现: {:?} 簇, {:?} 边, {} 洞察",
                parsed.discovery_update.new_clusters,
                parsed.discovery_update.new_edge_ids,
                parsed.discovery_update.new_insights.len());
        }
        if !state.open_questions.is_empty() {
            println!("❓ 遗留问题: {:?}", state.open_questions);
        }
        println!("---\n");

        // Staleness warning
        if stale_rounds >= 2 {
            println!("  ⏹️  连续 2 轮无新发现，探索收敛。结束。\n");
            break;
        }

        // Cluster focus warning
        if n >= 3 {
            let last_clusters: Vec<u32> = session.turns.iter().rev().take(3).flat_map(|t| {
                t.matched_clusters.iter().map(|c| c.id)
            }).collect();
            let set: std::collections::HashSet<&u32> = last_clusters.iter().collect();
            if set.len() <= 2 {
                println!("  ⚠️  已连续 3 轮聚焦于有限簇，建议尝试切换视角或做跨簇连接。");
            }
        }

        // Prompt for next input if not stale
        if stale_rounds < 2 && n < MAX_TURNS {
            println!("  (输入新想法继续，或 exit 退出)");
        }
    }

    // Summary report
    let clusters: Vec<u32> = session.turns.iter().flat_map(|t| t.parsed.discovery_update.new_clusters.clone()).collect();
    let edges: Vec<u32> = session.turns.iter().flat_map(|t| t.parsed.discovery_update.new_edge_ids.clone()).collect();
    let insights: Vec<&str> = session.turns.iter().flat_map(|t| t.parsed.discovery_update.new_insights.iter().map(|s| s.as_str())).collect();
    println!("\n=== 实验总结 ===");
    println!("总轮次: {}", n);
    println!("探索簇数: {}", clusters.len());
    println!("发现边数: {}", edges.len());
    println!("生成洞察: {}", insights.len());
    if state.open_questions.is_empty() {
        println!("遗留问题: 无");
    } else {
        println!("遗留问题 ({}):", state.open_questions.len());
        for q in &state.open_questions { println!("  - {}", q); }
    }

    Ok(())
}

fn ts() -> String {
    format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
}
