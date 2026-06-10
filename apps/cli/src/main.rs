use project_11::repl::Repl;
use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        println!("Project 11: Situation Engine (Experimental)");
        println!();
        println!("Usage: project-11 [--gallery <path>]");
        println!("       project-11 --help");
        println!();
        println!("Environment:");
        println!("  GALLERY_PATH    path to gallery directory (default: ../../../docs/gallery)");
        println!();
        println!("Note: All analysis commands have been migrated to qtcloud-think-cli.");
        println!("This experimental CLI only provides a basic REPL interface.");
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

    let repl = Repl::new(gallery_path);
    repl.run()
}
