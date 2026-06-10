use std::io::{self, BufRead};

pub struct Repl {
    gallery_path: String,
}

impl Repl {
    pub fn new(gallery_path: String) -> Self {
        Self { gallery_path }
    }

    pub fn run(&self) -> Result<(), String> {
        println!("=== Project 11 ===");
        println!("Commands: help / exit");
        println!("Note: All analysis commands have been migrated to qtcloud-think-cli.");
        println!("Type 'help' for more information.\n");

        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.map_err(|e| e.to_string())?;
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts[0] {
                "exit" | "quit" => break,
                "help" => {
                    println!("Available commands:");
                    println!("  help     - show this help message");
                    println!("  exit     - quit the program");
                    println!();
                    println!("Note: All analysis commands have been migrated to qtcloud-think-cli.");
                    println!("Please use qtcloud-think-cli for situation analysis and intention management.");
                }
                _ => println!("Unknown command. Type 'help' for available commands."),
            }
        }
        Ok(())
    }
}
