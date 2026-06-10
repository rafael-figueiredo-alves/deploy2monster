pub enum Command {
    New(String),
    Deploy(String),
    DbUpdate(String),
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
        Some("-new")      => {
            match args.get(2) {
                Some(name) => Command::New(name.to_string()),
                None => Command::Unknown("-new requer um nome para o novo projeto".to_string()),                
            }
        }
        Some("-deploy")   => {
             match args.get(2) {
                Some(name) => Command::Deploy(name.to_string()),
                None => Command::Unknown("-deploy requer o nome do projeto a ser implantado".to_string()),                
            }
        }
        Some("-dbUpdate") => {
            match args.get(2) {
                Some(name) => Command::DbUpdate(name.to_string()),
                None => Command::Unknown("-dbUpdate requer o nome do projeto a ser atualizado".to_string()),
            }
        }        
        Some("-help")     => Command::Help,
        Some(other) => Command::Unknown(other.to_string()),
        None              => Command::None,
    }
}