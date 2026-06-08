use qtcloud_think_situation::repl::Repl;
use qtcloud_think_situation::{ScaffoldEngine, SessionManager};

fn main() -> Result<(), String> {
    let engine = ScaffoldEngine::new("apps/situation/assets/situation-graph.json")?;
    let sessions = SessionManager::new("apps/situation/data");
    let repl = Repl::new(engine, sessions);
    repl.run()
}
