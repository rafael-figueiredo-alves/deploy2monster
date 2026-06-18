// Função principal que faz o parse dos argumentos da linha de comando e retorna um enum Command correspondente
pub fn parse_args() -> Command {
    let args: Vec<String> = std::env::args().collect();
    parse_command(&args)
}

// region: Função auxiliar que faz o parse detalhado dos argumentos

fn parse_command(args: &[String]) -> Command {
    match args.get(1).map(|s| s.as_str()) {
        Some("--new")      => parse_new_command(args),
        Some("--deploy")   => parse_deploy_command(args),
        Some("--dbUpdate") => parse_dbupdate_command(args),
        Some("--export")   => parse_export_command(args),
        Some("--import")   => parse_import_command(args),
        Some("--list")     => Command::List,
        Some("--delete")   => parse_delete_command(args),
        Some("--test")     => parse_test_command(args),
        Some("--edit")     => parse_edit_command(args),
        Some("--logs")     => parse_logs_command(args),
        Some("--version")  => Command::Version,
        Some("--help")     => Command::Help,
        Some(other)        => Command::Unknown(format!("Comando desconhecido: {}", other)),
        None               => Command::None,
    }
}

// region: ==================== FUNÇÕES DE PARSE INDIVIDUAIS ====================

fn parse_new_command(args: &[String]) -> Command {
    match args.get(2) {
        Some(name) => Command::New(name.to_string()),
        None => Command::Unknown("--new requer um nome para o novo projeto".to_string()),
    }
}

fn parse_deploy_command(args: &[String]) -> Command {
    match args.get(2) {
        Some(name) => {
            let skip_sql = args.get(3).map(|s| s.as_str()) == Some("--skip-sql");
            Command::Deploy(name.to_string(), skip_sql)
        }
        None => Command::Unknown("--deploy requer o nome do projeto".to_string()),
    }
}

fn parse_dbupdate_command(args: &[String]) -> Command {
    match args.get(2) {
        Some(name) => Command::DbUpdate(name.to_string()),
        None => Command::Unknown("--dbUpdate requer o nome do projeto a ser atualizado".to_string()),
    }
}

fn parse_export_command(args: &[String]) -> Command {
    match (args.get(2), args.get(3)) {
        (Some(name), Some(path)) => Command::Export(name.to_string(), path.to_string()),
        (Some(_), None) => Command::Unknown(
            "--export requer nome do projeto e caminho de destino. Ex: --export MeuProjeto C:\\Backup".to_string()
        ),
        _ => Command::Unknown("--export requer nome do projeto e caminho de destino.".to_string()),
    }
}

fn parse_import_command(args: &[String]) -> Command {
    match args.get(2) {
        Some(path) => Command::Import(path.to_string()),
        None => Command::Unknown("--import requer o caminho do arquivo .d2mproj a ser importado".to_string()),
    }
}

fn parse_delete_command(args: &[String]) -> Command {
    match args.get(2) {
        Some(name) => Command::Delete(name.to_string()),
        None => Command::Unknown("--delete requer o nome do projeto".to_string()),
    }
}

fn parse_test_command(args: &[String]) -> Command {
    match args.get(2) {
        Some(name) => Command::Test(name.to_string()),
        None => Command::Unknown("--test requer o nome do projeto".to_string()),
    }
}

fn parse_edit_command(args: &[String]) -> Command {
    match args.get(2) {
        Some(name) => Command::Edit(name.to_string()),
        None => Command::Unknown("--edit requer o nome do projeto a ser editado".to_string()),
    }
}

fn parse_logs_command(args: &[String]) -> Command {
    match args.get(2) {
        Some(name) => {
            let clean_days = if args.get(3).map(|s| s.as_str()) == Some("--clean") {
                args.get(4)
                    .and_then(|s| s.parse::<u64>().ok())
                    .or(Some(30))
            } else {
                None
            };
            Command::Logs(name.to_string(), clean_days)
        }
        None => Command::Unknown("--logs requer o nome do projeto".to_string()),
    }
}
// endregion

// endregion