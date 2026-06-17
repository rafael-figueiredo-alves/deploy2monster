// Importa os módulos necessários para o aplicativo
mod ui;
mod cli;
mod config;
mod consts;
mod projects;
mod input;
mod logger;
mod deployer;
mod crypto;
mod logs;

// Ponto de entrada do aplicativo
fn main() {
    // Imprime o banner com informações do aplicativo e icone
    ui::print_banner();

    // Cria ou carrega as configurações do aplicativo com a chave de criptografia
    let app_config = match config::AppConfig::load_or_create() {
        Ok(cfg) => cfg,
        Err(e)  => {
            ui::write_error(&format!("Erro ao carregar configurações: {}", e));
            std::process::exit(1);
        }
    };    

    // Analisa os argumentos da linha de comando e executa o comando correspondente
    let command = cli::parse_args();

    // Executa o comando e captura o resultado
    ui::print_command_result(&command, &app_config);
}