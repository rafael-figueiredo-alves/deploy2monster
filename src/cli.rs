pub enum Command {
    New(String),
    Edit(String),
    List,
    Export(String, String),  // ← nome do projeto, caminho de destino
    Import(String),        // ← caminho do arquivo a ser importado
    Deploy(String, bool),
    DbUpdate(String),
    Delete(String),
    Test(String),
    Logs(String),
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
                Some(name) => {
                    let skip_sql = args.get(3).map(|s| s.as_str()) == Some("--skip-sql");
                    Command::Deploy(name.to_string(), skip_sql)
                },
                None => Command::Unknown("-deploy requer o nome do projeto".to_string()),
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
        Some("-import") => {
            match args.get(2) {
                Some(path) => Command::Import(path.to_string()),
                None => Command::Unknown("-import requer o caminho do arquivo .d2mproj a ser importado".to_string()),
            }
        },     
        Some("-list")     => Command::List,
        Some("-delete") => {
            match args.get(2) {
                Some(name) => Command::Delete(name.to_string()),
                None => Command::Unknown("-delete requer o nome do projeto".to_string()),
            }
        }, 
        Some("-test") => {
            match args.get(2) {
                Some(name) => Command::Test(name.to_string()),
                None => Command::Unknown("-test requer o nome do projeto".to_string()),
            }
        },                 
        Some("-version")  => Command::Version,    
        Some("-help")     => Command::Help,
        Some("-edit")     => {
            match args.get(2) {
                Some(name) => Command::Edit(name.to_string()),
                None => Command::Unknown("-edit requer o nome do projeto a ser editado".to_string()),
            }
        },
        Some("-logs") => {
            match args.get(2) {
                Some(name) => Command::Logs(name.to_string()),
                None => Command::Unknown("-logs requer o nome do projeto".to_string()),
            }
        },    
        Some(other) => Command::Unknown(other.to_string()),
        None              => Command::None,
    }
}