use crate::business_layer::database;
use crate::business_layer::logs;
use crate::business_layer::project_bll::*;
use crate::business_layer::run_deploy;
use crate::business_layer::tester;
use crate::entities::{AppConfig, Command};
use crate::shared::input_functions::*;
use crate::shared::logger;
use crate::shared::message_functions::*;
use crate::shared::output_functions::*;

pub fn Run(command: Command, config: &AppConfig) {
    match command {
        Command::New(name) => new_project(&name, config),
        Command::Edit(name) => edit_project(&name, config),
        Command::List => list_projects_available(),
        Command::Delete(name) => delete_selected_project(&name),
        Command::Export(name, path) => export_available_project(&name, &path, config),
        Command::Import(name) => import_available_project(&name, config),
        Command::Deploy(name, skip_sql) => deploy_project(name, skip_sql, config),
        Command::DbUpdate(name) => update_database(&name, config),
        Command::Test(name) => do_a_test(name, config),
        Command::Logs(name, limit) => open_logs(name, limit),
        Command::Version => show_version(),
        Command::Help => show_help(),
        Command::Unknown(cmd) => on_unknown_command(cmd),
        Command::None => on_none_command(),
    }
}

// region: Funções privadas para execução de comandos

fn new_project(name: &str, config: &AppConfig) {
    if let Err(e) = create_project_interactive(name, &config.key()) {
        write_error(&format!("Erro: {}", e));
        std::process::exit(1);
    }
}

fn edit_project(name: &str, config: &AppConfig) {
    if let Err(e) = edit_project_interactive(name, config.key()) {
        write_error(&format!("Erro: {}", e));
        std::process::exit(1);
    }
}

fn list_projects_available() {
    match list_projects() {
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

fn delete_selected_project(name: &str) {
    match find_project_name(name) {
        Err(e) => {
            write_error(&e);
            std::process::exit(1);
        }
        Ok(found_name) => {
            println!();
            write_warning(&format!(
                "Você está prestes a apagar o projeto '{}'.",
                found_name
            ));
            write_warning("Esta ação não pode ser desfeita.");
            println!();

            if !ask_confirm(&format!("Confirma exclusão do projeto '{}'?", found_name)) {
                println!();
                write_warning("Operação cancelada.");
                return;
            }

            match delete_project(name) {
                Ok(_) => {
                    println!();
                    write_success(&format!("Projeto '{}' removido com sucesso!", found_name));
                }
                Err(e) => {
                    write_error(&e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn export_available_project(name: &str, dest_path: &str, config: &AppConfig) {
    match export_project(name, dest_path, &config.key()) {
        Ok(_) => write_success(&format!("Projeto '{}' exportado com sucesso!", name)),
        Err(e) => {
            write_error(&e);
            std::process::exit(1);
        }
    }
}

fn import_available_project(name: &str, config: &AppConfig) {
    match import_project(name, &config.key()) {
        Ok(_) => write_success(&format!("Projeto '{}' importado com sucesso!", name)),
        Err(e) => {
            write_error(&e);
            std::process::exit(1);
        }
    }
}

fn deploy_project(name: String, skip_sql: bool, config: &AppConfig) {
    match load_project(&name, config.key()) {
        Err(e) => {
            write_error(&e);
            std::process::exit(1);
        }
        Ok(proj) => {
            println!();
            println!("  Projeto  : {}", proj.name);
            println!("  Destino  : {}", proj.ftp_settings.ftp_host);
            println!(
                "  Banco    : {}/{}",
                proj.database_settings.host, proj.database_settings.database
            );
            println!("  Publish  : {}", proj.publish_folder);
            println!();

            if !ask_confirm("Confirma o deploy?") {
                println!();
                write_warning("Deploy cancelado.");
                return;
            }

            println!();
            logger::init(&proj.name);
            run_deploy(&proj, skip_sql);
        }
    }
}

fn show_version() {
    print_version();
}

fn show_help() {
    print_help();
}

fn on_unknown_command(cmd: String) {
    write_error(&format!("Comando desconhecido: '{}'", cmd));
    std::process::exit(1);
}

fn on_none_command() {
    write_error("Nenhum comando fornecido. Use --help para ver os comandos disponíveis.");
    std::process::exit(1);
}

fn update_database(name: &str, config: &AppConfig) {
    match load_project(name, &config.key()) {
        Err(e) => {
            write_error(&format!("Erro: {}", e));
            std::process::exit(1);
        }
        Ok(proj) => {
            logger::init(&proj.name);
            if let Err(e) = database::run(&proj) {
                write_error(&format!("Erro: {}", e));
                std::process::exit(1);
            }
        }
    }
}

fn do_a_test(name: String, config: &AppConfig) {
    match load_project(&name, &config.key()) {
        Err(e) => {
            write_error(&e);
            std::process::exit(1);
        }
        Ok(proj) => {
            tester::run(&proj);
        }
    }
}

fn open_logs(name: String, limit: Option<u64>) {
    // modo limpeza
    if let Some(days) = limit {
        println!();
        write_warning(&format!(
            "Removendo logs do projeto '{}' com mais de {} dia(s)...",
            name, days
        ));

        match logs::delete_old_logs(&name, days) {
            Ok(0) => write_info("Nenhum log removido."),
            Ok(n) => write_success(&format!("{} log(s) removido(s).", n)),
            Err(e) => write_error(&e),
        }
        println!();
        return;
    }

    // modo listagem — igual ao anterior
    match logs::list_logs(&name) {
        Err(e) => {
            write_error(&e);
            std::process::exit(1);
        }
        Ok(entries) if entries.is_empty() => {
            println!();
            println!("  Nenhum log encontrado para o projeto '{}'.", name);
            println!("  Execute -deploy para gerar logs.");
            println!();
        }
        Ok(entries) => {
            println!();
            println!(
                "  Logs do projeto '{}' ({} encontrado(s)):",
                name,
                entries.len()
            );
            println!();

            for (i, entry) in entries.iter().enumerate() {
                println!(
                    "  {:>3}. {}   {:>4} KB",
                    i + 1,
                    entry.created_at,
                    entry.size_kb.max(1)
                );
            }

            println!();
            println!("  Digite o número para visualizar, ou pressione ESC para sair.");
            println!();

            loop {
                match ask("Número do log") {
                    None => break,
                    Some(input) => match input.trim().parse::<usize>() {
                        Ok(n) if n >= 1 && n <= entries.len() => {
                            let entry = &entries[n - 1];
                            match logs::open_log(&entry.path) {
                                Ok(content) => {
                                    println!();
                                    println!("  {}", "─".repeat(60));
                                    println!("  Log: {}", entry.filename);
                                    println!("  {}", "─".repeat(60));
                                    println!();
                                    for line in content.lines() {
                                        println!("  {}", line);
                                    }
                                    println!();
                                    println!("  {}", "─".repeat(60));
                                    println!();
                                }
                                Err(e) => write_error(&e),
                            }
                        }
                        _ => write_error(&format!("Digite um número entre 1 e {}.", entries.len())),
                    },
                }
            }
        }
    }
}

// endregion
