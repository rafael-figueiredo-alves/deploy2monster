// Importa os módulos necessários para o aplicativo
mod app;
mod business_layer;
mod entities;
mod shared;

// Ponto de entrada do aplicativo
fn main() {
    // Imprime o banner com informações do aplicativo e icone
    shared::output_functions::print_banner();

    // Cria ou carrega as configurações do aplicativo com a chave de criptografia
    let app_config = match entities::AppConfig::load_or_create() {
        Ok(cfg) => cfg,
        Err(e)  => {
            shared::message_functions::write_error(&format!("Erro ao carregar configurações: {}", e));
            std::process::exit(1);
        }
    };    

    // Analisa os argumentos da linha de comando e executa o comando correspondente
    let command = shared::parse_functions::parse_args();

    // Executa o comando e captura o resultado
    app::run(command, &app_config);
}