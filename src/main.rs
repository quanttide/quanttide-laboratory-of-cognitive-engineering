mod detect;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: lab check --input <file.md> [--mode summary|normal|verbose]");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "check" => {
            let mut detect_args = vec!["detect".to_string(), "check".to_string()];
            detect_args.extend(args[2..].iter().cloned());
            detect::dispatch(clap::Parser::parse_from(detect_args));
        }
        _ => eprintln!("Unknown command: {}", args[1]),
    }
}
