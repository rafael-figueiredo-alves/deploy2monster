use crate::cli::Command;
use crate::consts::{APP_NAME,APP_VERSION};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::{deployer, logger, projects};

fn current_year() -> u64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    1970 + (secs / 31_557_600) // segundos em um ano solar médio
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
    println!("  -new <nome_do_projeto>        Cria um novo arquivo de projeto (.json)");
    println!("  -deploy <nome_do_projeto>     Executa o deploy da aplicação");
    println!("  -dbUpdate <nome_do_projeto>   Executa a atualização do banco de dados");
    println!("  -version                      Obtenha a versão atual do sistema");
    println!("  -help                         Exibe esta mensagem");
}

fn print_version() {
    println!("");
    println!("Versão do Deploy2Monster: {:<27}", APP_VERSION)
}

pub fn print_command_result(command: &Command) {
    match command {
        Command::New(name) => {            
            if let Err(e) = projects::create_project_interactive(name) {
                write_error(&format!("Erro: {}", e));
                std::process::exit(1);                
            }
        },
        Command::Deploy(name) => {
            match projects::load_project(name) {
                Err(e) => {
                    write_error(&format!("Erro: {}", e));
                    std::process::exit(1);
                }
                Ok(proj) => {
                    logger::init(&proj.name);
                    deployer::run(&proj);
                }
            }
        },
        Command::DbUpdate(name) => {
            match projects::load_project(name) {
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
        Command::Version => print_version(),     
        Command::Help => print_help(),
        Command::Unknown(cmd) => {
            write_error(&format!("Comando desconhecido: '{}'", cmd));
            write_info("Use -help para ver os comandos disponíveis.");
            std::process::exit(1);
        }
        Command::None => print_help(),
    }
}

