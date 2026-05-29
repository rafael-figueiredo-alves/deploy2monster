
mod ui;
mod cli;
mod consts;

fn main() {
    ui::print_banner();

    let command = cli::parse_args();

    if let cli::Command::Unknown(_) = &command {
        ui::print_command_result(&command);
        std::process::exit(1);
    }

    ui::print_command_result(&command);
}