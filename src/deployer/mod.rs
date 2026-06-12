pub mod database;
pub mod ftp;
pub mod publisher;

use crate::logger;
use crate::projects::Project;
use crate::ui::{write_info, write_warning};

pub fn run(project: &Project) {
    logger::info(&format!("Iniciando deploy do projeto '{}'", project.name));
    println!();

    // Etapa 1 — dotnet publish
    println!("  [1/3] Publicando aplicação...");
    if let Err(e) = publisher::run(project) {
        logger::error(&format!("Falha na publicação: {}", e));
        abort();
    }
    println!();

    // Etapa 2 — FTP
    println!("  [2/3] Enviando arquivos via FTP...");
    if let Err(e) = ftp::run(project) {
        logger::error(&format!("Falha no FTP: {}", e));
        abort();
    }
    println!();

    // Etapa 3 — SQL
    println!("  [3/3] Executando script SQL...");
    if let Err(e) = database::run(project) {
        logger::error(&format!("Falha no banco de dados: {}", e));
        // não aborta — erros SQL são logados mas não interrompem
    }
    println!();

    logger::info("Deploy concluído com sucesso!");

    if let Some(path) = logger::log_path() {
        println!();
        write_info(&format!("  📄 Log salvo em: {}", path.display()));
    }
}

fn abort() {
    if let Some(path) = logger::log_path() {
        eprintln!();
        write_warning(&format!("Deploy abortado. Consulte o log: {}", path.display()));
    }
    std::process::exit(1);
}