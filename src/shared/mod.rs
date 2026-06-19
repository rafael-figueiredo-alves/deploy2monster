// Declara os módulos (arquivos)
pub mod parse_functions;
pub mod message_functions;
pub mod consts;
pub mod input_functions;
pub mod date_functions;
pub mod logger; // módulo de logging para registrar mensagens em um arquivo
pub mod output_functions; // módulo para funções de saída (exibição) no terminal

// ==================== REEXPORTS (o mais importante) ====================

// Exporta funções específicas
pub use parse_functions::{
    parse_args,
    // outras funções de parse que você queira exportar...
};

pub use message_functions::{
    write_error,
    write_success,
    write_warning,
    write_info,
    // etc...
};

pub use consts::{
    APP_NAME,
    APP_VERSION,
    CONFIG_FILENAME,
    KEY_SIZE,
    NONCE_SIZE,
    // etc...
};

// exporta todas as funções de input de uma vez
pub use input_functions::*;

pub use date_functions::{
    current_year,
    chrono_now_str,
};

pub use logger::*;

pub use output_functions::{
    print_banner,    
    print_help,
    print_version,
    // etc...
};