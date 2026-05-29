use crate::cli::Command;
use crate::consts::{APP_NAME,APP_VERSION};

pub fn print_banner() {
    println!("╔══════════════════════════════════════╗");
    println!("║           {}             ║", APP_NAME);
    println!("║           versão {}               ║", APP_VERSION);
    println!("╚══════════════════════════════════════╝");
    println!();
}

fn print_help() {
    println!("Uso: deploy2monster <comando>");
    println!();
    println!("Comandos disponíveis:");
    println!("  -new      Cria um novo arquivo de projeto (.json)");
    println!("  -deploy   Executa o deploy da aplicação");
    println!("  -help     Exibe esta mensagem");
}

pub fn print_command_result(command: &Command) {
    match command {
        Command::New => println!("→ Criando novo arquivo de projeto..."),
        Command::Deploy => println!("→ Iniciando deploy..."),
        Command::Help => print_help(),
        Command::Unknown(cmd) => {
            eprintln!("Comando desconhecido: '{}'", cmd);
            eprintln!("Use -help para ver os comandos disponíveis.");
            std::process::exit(1);
        }
        Command::None => print_help(),
    }
}

