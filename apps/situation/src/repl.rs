use std::io::{self, BufRead};

use crate::{DiscoveryState, ParsedResponse, ScaffoldEngine, SessionManager, Turn, ts};
use crate::summary::SessionSummary;

const MAX_TURNS: usize = 16;

pub struct Repl {
    engine: ScaffoldEngine,
    sessions: SessionManager,
}

impl Repl {
    pub fn new(engine: ScaffoldEngine, sessions: SessionManager) -> Self {
        Self { engine, sessions }
    }

    pub fn run(&self) -> Result<(), String> {
        println!("=== qtcloud-think-intent ===");
        println!("Type your thoughts ('exit' to quit)\n");

        let stdin = io::stdin();
        let mut state = DiscoveryState::new();
        let mut session = self.sessions.load_or_create();
        let mut n = 0usize;
        let mut detector = ConvergenceDetector::new(2);

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

            let (parsed, raw) = self.engine.process_with_state(input, &state)?;
            let matched = self.engine.match_with_history(input, &state);
            let state_before = state.clone();
            let has_new = !parsed.discovery_update.new_situations.is_empty()
                || !parsed.discovery_update.new_node_ids.is_empty()
                || !parsed.discovery_update.new_edge_ids.is_empty();

            state.merge(&parsed.discovery_update);

            let turn = Turn {
                id: format!("{}_{}", now, n),
                timestamp: now,
                input: input.to_string(),
                matched_situations: matched,
                state_before,
                state_after: state.clone(),
                llm_raw: raw,
                parsed: parsed.clone(),
            };
            session.turns.push(turn);
            self.sessions.save(&session);

            print_turn_output(&parsed, &state, has_new);

            if detector.check(has_new) {
                println!("  连续 2 轮无新发现，探索收敛。\n");
                break;
            }
        }

        let summary = SessionSummary::from_session(&session);
        println!("\n=== 总结 ===");
        println!("{}", summary);

        Ok(())
    }
}

fn print_turn_output(parsed: &ParsedResponse, state: &DiscoveryState, has_new: bool) {
    println!("\n---");
    if !parsed.positioning.is_empty() { println!("📍 {}", parsed.positioning); }
    if !parsed.connections.is_empty() { println!("🔗 {}", parsed.connections); }
    if !parsed.exploration.is_empty() { println!("💡 {}", parsed.exploration); }

    if let Some(ref m) = parsed.motif {
        if m.is_new_motif {
            println!("🆕 新母题：{}", m.motif_statement);
        } else {
            println!("🎵 母题：{}", m.motif_statement);
            for v in &m.variations {
                println!("   情境{} ({}): {}", v.situation_id, v.week, v.form);
            }
        }
        if !m.motif_arc.is_empty() {
            println!("   演化弧：{}", m.motif_arc);
        }
    }

    if has_new {
        println!("🆕 情境: {:?}  边: {:?}  洞察: {}",
            parsed.discovery_update.new_situations,
            parsed.discovery_update.new_edge_ids,
            parsed.discovery_update.new_insights.len());
    }
    if !state.open_questions.is_empty() {
        println!("❓ {}", state.open_questions.join(" | "));
    }
    println!("---\n");
}

pub struct ConvergenceDetector {
    max_stale: usize,
    stale: usize,
}

impl ConvergenceDetector {
    pub fn new(max_stale: usize) -> Self {
        Self { max_stale, stale: 0 }
    }

    /// Returns true if convergence detected (should stop).
    pub fn check(&mut self, has_new: bool) -> bool {
        if has_new { self.stale = 0; } else { self.stale += 1; }
        self.stale >= self.max_stale
    }
}
