pub enum Command {
    New(String),
    Edit(String),
    List,
    Export(String, String),  // ← nome do projeto, caminho de destino
    Import(String),        // ← caminho do arquivo a ser importado
    Deploy(String),
    DbUpdate(String),
    Version,
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
        Some("-export") => {
            match (args.get(2), args.get(3)) {
                (Some(name), Some(path)) => Command::Export(name.to_string(), path.to_string()),
                (Some(_), None) => Command::Unknown("-export requer nome do projeto e caminho de destino. Ex: -export MeuProjeto C:\\Backup".to_string()),
                _ => Command::Unknown("-export requer nome do projeto e caminho de destino.".to_string()),
            }  
        },      
        Some("-list")     => Command::List,  
        Some("-version")  => Command::Version,    
        Some("-help")     => Command::Help,
        Some("-edit")     => {
            match args.get(2) {
                Some(name) => Command::Edit(name.to_string()),
                None => Command::Unknown("-edit requer o nome do projeto a ser editado".to_string()),
            }
        }
        Some(other) => Command::Unknown(other.to_string()),
        None              => Command::None,
    }
}