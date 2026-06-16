use crate::cli::Command;
use crate::config;
use crate::deployer::tester;
use crate::input;
use crate::logs;
use chrono::Local;
use chrono::Datelike;
use crate::consts::{APP_NAME,APP_VERSION};
use crate::{deployer, logger, projects};

fn current_year() -> i32 {
    Local::now().year()
}

pub fn write_error(msg: &str) {
    eprintln!("\x1b[31m  ✘ {}\x1b[0m", msg); //Mensagem em vermelho
}

pub fn write_success(msg: &str) {
    println!("\x1b[32m  ✔ {}\x1b[0m", msg); //Mensagem em verde
}

pub fn write_warning(msg: &str) {
    println!("\x1b[33m  ⚠ {}\x1b[0m", msg); //Mensagem em amarelo
}

pub fn write_info(msg: &str) {
    println!("  → {}", msg); //Não usa cor diferente do padrão do console
}

pub fn print_banner() {
    let separator = "=".repeat(60);
    println!();
    println!();
    println!("   /\\_/\\       {:<28}       ", APP_NAME);
    println!("  ( o,o )      v{:<27}       ", APP_VERSION);
    println!("  /)   (\\      {:<28}", format!("© {} - Rafael de Figueiredo Alves", current_year()));
    println!("  \\/\\_/\\/                                         ");
    println!("{}", separator);
    println!();
}

fn print_help() {
    println!("Uso: deploy2monster <comando>");
    println!();
    println!("Comandos disponíveis:");
    println!("  -new <nome_do_projeto>                      Cria um novo projeto (.d2mproj)");
    println!("  -edit <nome_do_projeto>                     Edita um projeto existente");
    println!("  -list                                       Lista os projetos cadastrados");
    println!("  -export <nome_do_projeto> <caminho>          Exporta um projeto para o destino informado");
    println!("  -import <caminho_arquivo.d2mproj>            Importa um projeto exportado");
    println!("  -delete <nome_do_projeto>                   Remove um projeto cadastrado");
    println!();
    println!("  -deploy <nome_do_projeto> [--skip-sql]      Executa o deploy completo");
    println!("  -dbUpdate <nome_do_projeto>                 Executa apenas o banco de dados");
    println!("  -test <nome_do_projeto>                     Testa conexões FTP e banco de dados");
    println!("  -logs <nome_do_projeto>                     Lista e abre logs de deploy");
    println!();
    println!("  -version                                    Exibe a versão atual do sistema");
    println!("  -help                                       Exibe esta mensagem");
    println!();
    println!("Dica: pressione ESC nos prompts interativos para cancelar a operação.");
}

fn print_version() {
    println!("");
    println!("Versão do Deploy2Monster: {:<27}", APP_VERSION)
}

pub fn print_command_result(command: &Command, config: &config::AppConfig) {
    match command {
        Command::New(name) => {            
            if let Err(e) = projects::create_project_interactive(name, &config.key()) {
                write_error(&format!("Erro: {}", e));
                std::process::exit(1);                
            }
        },
        Command::Deploy(name, skip_sql) => {
            match projects::load_project(name, config.key()) {
                Err(e) => {
                    write_error(&e);
                    std::process::exit(1);
                }
                Ok(proj) => {
                    println!();
                    println!("  Projeto  : {}", proj.name);
                    println!("  Destino  : {}", proj.ftp_settings.ftp_host);
                    println!("  Banco    : {}/{}", proj.database_settings.host, proj.database_settings.database);
                    println!("  Publish  : {}", proj.publish_folder);
                    println!();

                    if !input::ask_confirm("Confirma o deploy?") {
                        println!();
                        write_warning("Deploy cancelado.");
                        return;
                    }

                    println!();
                    logger::init(&proj.name);
                    deployer::run(&proj, *skip_sql);
                }
            }
        },
        Command::DbUpdate(name) => {
            match projects::load_project(name, &config.key()) {
                Err(e) => {
                    write_error(&format!("Erro: {}", e));
                    std::process::exit(1);
                }
                Ok(proj) => {
                    logger::init(&proj.name);
                    if let Err(e) = deployer::database::run(&proj){
                        write_error(&format!("Erro: {}", e));
                        std::process::exit(1);                        
                    }
                }
            }
        }, 
        Command::Export(name, dest_path) => {
            match projects::export_project(name, dest_path, &config.key()) {
                Ok(_)  => write_success(&format!("Projeto '{}' exportado com sucesso!", name)),
                Err(e) => {
                    write_error(&e);
                    std::process::exit(1);
                }
            }
        }, 
        Command::Import(name) => {
            match projects::import_project(name, &config.key()) {
                Ok(_) => write_success(&format!("Projeto '{}' importado com sucesso!", name)),
                Err(e) => {
                    write_error(&e);
                    std::process::exit(1);
                }
            }
        },       
        Command::List => {
            match projects::list_projects() {
                Err(e) => {
                    write_error(&format!("Erro: {}", e));
                    std::process::exit(1);
                }
                Ok(list) if list.is_empty() => {
                        println!("  Nenhum projeto cadastrado.");
                        println!("  Use -new <nome> para criar um projeto.");
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
        }, 
        Command::Delete(name) => {
            // tenta carregar para confirmar que existe e mostrar detalhes
            match projects::find_project_name(name) {
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

                    if !input::ask_confirm(&format!("Confirma exclusão do projeto '{}'?", found_name)) {
                        println!();
                        write_warning("Operação cancelada.");
                        return;
                    }

                    match projects::delete_project(name) {
                        Ok(_)  => {
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
        },
        Command::Test(name) => {
            match projects::load_project(name, &config.key()) {
                Err(e) => {
                    write_error(&e);
                    std::process::exit(1);
                }
                Ok(proj) => {
                    tester::run(&proj);
                }
            }
        },  
        Command::Logs(name) => {
            match logs::list_logs(name) {
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
                    println!("  Logs do projeto '{}' ({} encontrado(s)):", name, entries.len());
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
                        match input::ask("Número do log") {
                            None => break, // ESC
                            Some(input) => {
                                match input.trim().parse::<usize>() {
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
                                    _ => write_error(&format!(
                                        "Digite um número entre 1 e {}.",
                                        entries.len()
                                    )),
                                }
                            }
                        }
                    }
                }
            }
        },      
        Command::Version => print_version(),     
        Command::Help => print_help(),
        Command::Edit(name) => {
            if let Err(e) = projects::edit_project_interactive(name, config.key()) {
                write_error(&e);
                std::process::exit(1);
            }
        }
        Command::Unknown(cmd) => {
            write_error(&format!("Comando desconhecido: '{}'", cmd));
            write_info("Use -help para ver os comandos disponíveis.");
            std::process::exit(1);
        }
        Command::None => print_help(),
    }
}

