
mod ui;
mod cli;
mod config;
mod consts;
mod projects;
mod input;
mod logger;
mod deployer;

fn main() {
    ui::print_banner();

    let app_config = match config::AppConfig::load_or_create() {
        Ok(cfg) => cfg,
        Err(e)  => {
            ui::write_error(&format!("Erro ao carregar configurações: {}", e));
            std::process::exit(1);
        }
    };    

    let command = cli::parse_args();

    if let cli::Command::Unknown(_) = &command {
        ui::print_command_result(&command, &app_config);
        std::process::exit(1);
    }

    ui::print_command_result(&command, &app_config);
}