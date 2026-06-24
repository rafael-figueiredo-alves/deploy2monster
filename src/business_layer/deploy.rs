use crate::shared::logger;
use crate::entities::project::Project;
use std::time::Instant;
use crate::business_layer::validator;
use crate::business_layer::publisher;
use crate::business_layer::ftp;
use crate::business_layer::database;
use crate::business_layer::logs;
use crate::shared::message_functions::*;

pub fn run_deploy(project: &Project, skip_sql: bool) {
    // validação antes de começar
    let validation = validator::validate(project);

    if !validation.warnings.is_empty() {
        for w in &validation.warnings {
            write_warning(w);
        }
        println!();
    }

    if validation.has_errors() {
        println!();
        write_error("Deploy abortado — corrija os problemas abaixo:");
        println!();
        for e in &validation.errors {
            write_error(&format!("  • {}", e));
        }
        println!();
        write_info("Use --edit <nome> para corrigir as configurações do projeto.");
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
        write_success(&format!(
            "Deploy do projeto '{}' concluído com sucesso em {}m {}s!",
            project.name, mins, secs
        ));
    } else {
        write_success(&format!(
            "Deploy do projeto '{}' concluído com sucesso em {}s!",
            project.name, secs
        ));
    }

    logger::info("Deploy concluído com sucesso!");

    // retenção automática — apaga logs com mais de 30 dias
    match logs::delete_old_logs(&project.name, 30) {
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
        write_warning(&format!("Deploy abortado. Consulte o log: {}", path.display()));
    }
    std::process::exit(1);
}