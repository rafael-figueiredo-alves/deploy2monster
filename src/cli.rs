pub enum Command {
    New,
    Deploy,
    Help,
    Unknown(String),
    None,
}

pub fn parse_args() -> Command {
    let args: Vec<String> = std::env::args().collect();
    parse_command(&args)
}

fn parse_command(args: &[String]) -> Command {
    match args.get(1).map(|s| s.as_str()) {
        Some("-new")      => Command::New,
        Some("-deploy")   => Command::Deploy,
        Some("-help")     => Command::Help,
        Some(other) => Command::Unknown(other.to_string()),
        None              => Command::None,
    }
}