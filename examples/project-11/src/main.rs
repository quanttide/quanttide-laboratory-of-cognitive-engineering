use project_11::repl::Repl;
use project_11::query::QueryEngine;

fn main() -> Result<(), String> {
    let engine = QueryEngine::new(
        "/home/iguo/repos/quanttide/domains/quanttide-think/docs/gallery",
    );
    let repl = Repl::new(engine);
    repl.run()
}
