use project_11::repl::Repl;
use project_11::query::QueryEngine;
use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        println!("Project 11: Situation Engine");
        println!();
        println!("Usage: project-11 [--gallery <path>]");
        println!("       project-11 --help");
        println!();
        println!("Environment:");
        println!("  GALLERY_PATH    path to gallery directory (default: ../../docs/gallery)");
        println!();
        println!("Commands (REPL):");
        println!("  weeks                    - list available weeks");
        println!("  show <week>              - show week summary");
        println!("  landscape <week>         - show week landscape (compact)");
        println!("  explore <name>           - track situation evolution across weeks");
        println!("  registry                 - show situation registry");
        println!("  report <week>            - generate structured weekly report");
        println!("  diff <weekA> <weekB>      - compare two weeks");
        println!("  relate <week>            - LLM infer situation relations");
        println!("  exit                     - quit");
        return Ok(());
    }

    let gallery_path = if let Some(pos) = args.iter().position(|a| a == "--gallery") {
        args.get(pos + 1).cloned().unwrap_or_default()
    } else {
        env::var("GALLERY_PATH").unwrap_or_else(|_| "../../../docs/gallery".to_string())
    };

    if gallery_path.is_empty() {
        return Err("Gallery path not set. Use --gallery <path> or GALLERY_PATH env var.".to_string());
    }

    let engine = QueryEngine::new(&gallery_path);
    let repl = Repl::new(engine, gallery_path);
    repl.run()
}
