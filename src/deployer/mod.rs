pub mod database;
pub mod ftp;
pub mod publisher;
pub mod tester;
pub mod validator;

use crate::logger;
use crate::projects::Project;
use crate::ui as ui;
use std::time::Instant;

pub fn run(project: &Project, skip_sql: bool) {
    // validação antes de começar
    let validation = validator::validate(project);

    if !validation.warnings.is_empty() {
        for w in &validation.warnings {
            ui::write_warning(w);
        }
        println!();
    }

    if validation.has_errors() {
        println!();
        ui::write_error("Deploy abortado — corrija os problemas abaixo:");
        println!();
        for e in &validation.errors {
            ui::write_error(&format!("  • {}", e));
        }
        println!();
        ui::write_info("Use -edit <nome> para corrigir as configurações do projeto.");
        std::process::exit(1);
    }

    let started_at = Instant::now(); // ← início

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
    if skip_sql {
        println!("  [3/3] Etapa SQL ignorada (--skip-sql).");
    } else {
        println!("  [3/3] Executando script SQL...");
        if let Err(e) = database::run(project) {
            logger::error(&format!("Falha no banco de dados: {}", e));
        }
    }
    println!();

    let elapsed = started_at.elapsed();
    let mins    = elapsed.as_secs() / 60;
    let secs    = elapsed.as_secs() % 60;

    if mins > 0 {
        ui::write_success(&format!(
            "Deploy do projeto '{}' concluído com sucesso em {}m {}s!",
            project.name, mins, secs
        ));
    } else {
        ui::write_success(&format!(
            "Deploy do projeto '{}' concluído com sucesso em {}s!",
            project.name, secs
        ));
    }

    logger::info("Deploy concluído com sucesso!");

    // retenção automática — apaga logs com mais de 30 dias
    match crate::logs::delete_old_logs(&project.name, 30) {
        Ok(0) => {}
        Ok(n) => logger::info(&format!("  {} log(s) antigo(s) removido(s).", n)),
        Err(e) => logger::warn(&format!("  Erro na retenção de logs: {}", e)),
    }

    if let Some(path) = logger::log_path() {
        println!();
        println!("  📄 Log salvo em: {}", path.display());
    }
}

fn abort() {
    if let Some(path) = logger::log_path() {
        eprintln!();
        ui::write_warning(&format!("Deploy abortado. Consulte o log: {}", path.display()));
    }
    std::process::exit(1);
}