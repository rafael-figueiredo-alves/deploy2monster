use crate::cli::Command;
use crate::consts::{APP_NAME,APP_VERSION};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::projects;

fn current_year() -> u64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    1970 + (secs / 31_557_600) // segundos em um ano solar médio
}

pub fn print_banner() {
    let separator = "=".repeat(52);
    println!();
    println!("  __________________________________________________");
    println!(" |                                                  |");
    println!(" |   /\\_/\\       {:<28}       |", APP_NAME);
    println!(" |  ( o,o )      v{:<27}       |", APP_VERSION);
    println!(" |  /)   (\\      {:<28}|", format!("© {} - Rafael de Figueiredo Alves", current_year()));
    println!(" |  \\/\\_/\\/                                         |");
    println!(" |__________________________________________________|");
    println!("  {}", separator);
    println!();
}

fn print_help() {
    println!("Uso: deploy2monster <comando>");
    println!();
    println!("Comandos disponíveis:");
    println!("  -new <nome_do_projeto>      Cria um novo arquivo de projeto (.json)");
    println!("  -deploy <nome_do_projeto>   Executa o deploy da aplicação");
    println!("  -help                       Exibe esta mensagem");
}

pub fn print_command_result(command: &Command) {
    match command {
        Command::New(name) => {
            print!("→ Criando novo projeto: {} ... ", name);
            if let Err(e) = projects::create_project_interactive(name) {
                eprintln!("✘ Erro: {}", e);
                std::process::exit(1);                
            }
        },
        Command::Deploy(name) => println!("→ Iniciando deploy do projeto: {}", name ),
        Command::Help => print_help(),
        Command::Unknown(cmd) => {
            eprintln!("Comando desconhecido: '{}'", cmd);
            eprintln!("Use -help para ver os comandos disponíveis.");
            std::process::exit(1);
        }
        Command::None => print_help(),
    }
}

