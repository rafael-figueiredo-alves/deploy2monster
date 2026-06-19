use crate::shared::consts::{APP_NAME, APP_VERSION};
use crate::shared::date_functions::current_year;

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

pub fn print_help() {
    println!("Uso: deploy2monster <comando>");
    println!();
    println!("Comandos disponíveis:");
    println!("==============================================================================================");
    println!("  --new <nome_do_projeto>                      Cria um novo projeto (.d2mproj)");
    println!("  --edit <nome_do_projeto>                     Edita um projeto existente");
    println!("  --list                                       Lista os projetos cadastrados");
    println!("  --export <nome_do_projeto> <caminho>         Exporta um projeto para o destino informado");
    println!("  --import <caminho_arquivo.d2mproj>           Importa um projeto exportado");
    println!("  --delete <nome_do_projeto>                   Remove um projeto cadastrado");
    println!();
    println!("  --deploy <nome_do_projeto> [--skip-sql]      Executa o deploy completo");
    println!("  --dbUpdate <nome_do_projeto>                 Executa apenas o banco de dados");
    println!("  --test <nome_do_projeto>                     Testa conexões FTP e banco de dados");
    println!("  --logs <nome> [--clean <dias>]               Lista logs ou limpa logs antigos");
    println!();
    println!("  --version                                    Exibe a versão atual do sistema");
    println!("  --help                                       Exibe esta mensagem");
    println!("==============================================================================================");
    println!();
    println!("Dica: pressione [ESC] nos prompts interativos para cancelar a operação.");
    println!();
}

pub fn print_version() {
    println!("");
    println!("Versão do Deploy2Monster: {:<27}", APP_VERSION);
    println!();
}