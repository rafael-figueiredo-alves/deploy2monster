use crate::entities::Command;
use crate::entities::AppConfig;

pub fn Run(command: &Command, config: &AppConfig) {
    match command {
        Command::New(name) => NewProject(name, config),
        Command::Edit(name) => EditProject(name, config),
        Command::List => ListProjects(),
    }
}

// region: Funções privadas para execução de comandos

fn NewProject(name: &str, config: &AppConfig) {
    if let Err(e) = projects::create_project_interactive(name, &config.key()) {
        write_error(&format!("Erro: {}", e));
        std::process::exit(1);                
    }
}

fn EditProject(name: &str, config: &AppConfig) {
    if let Err(e) = projects::edit_project_interactive(name, config.key()) {
        write_error(&e);
        std::process::exit(1);
    }
}

fn ListProjects() {
    match projects::list_projects() {
        Err(e) => {
                    write_error(&format!("Erro: {}", e));
                    std::process::exit(1);
                  }
                Ok(list) if list.is_empty() => {
                    println!("  Nenhum projeto cadastrado.");
                    println!("  Use --new <nome> para criar um projeto.");
                }
                Ok(list) => {
                    println!("  Projetos cadastrados ({}):", list.len());
                    println!();
                for (i, name) in list.iter().enumerate() {
                    println!("  {:>3}. {}", i + 1, name);
            }
        }
    }
    println!();
}

// endregion