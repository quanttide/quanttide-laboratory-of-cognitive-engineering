use qtcloud_think_intent::repl::Repl;
use qtcloud_think_intent::{ScaffoldEngine, SessionManager};

fn main() -> Result<(), String> {
    let engine = ScaffoldEngine::new("data/formal/intent-graph.json")?;
    let sessions = SessionManager::new("apps/intent/data");
    let repl = Repl::new(engine, sessions);
    repl.run()
}
