use std::env;

const APP_NAME: &str = "Deploy2Monster";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();

    match parse_command(&args) {
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

enum Command {
    New,
    Deploy,
    Help,
    Unknown(String),
    None,
}

fn parse_command(args: &[String]) -> Command {
    match args.get(1).map(|s| s.as_str()) {
        Some("-new")      => Command::New,
        Some("-deploy")   => Command::Deploy,
        Some("-help")     => Command::Help,
        Some(other) => Command::Unknown(other.to_string()),
        None              => Command::None,
    }
}

fn print_banner() {
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