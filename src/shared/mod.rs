// Declara os módulos (arquivos)
pub mod parse_functions;
pub mod message_functions;
pub mod consts;
pub mod input_functions;
pub mod date_functions;
pub mod logger; // módulo de logging para registrar mensagens em um arquivo
pub mod output_functions; // módulo para funções de saída (exibição) no terminal
pub mod path_functions;
pub mod crypto_functions;
pub mod db_errors;
pub mod ftp_errors;

// ==================== REEXPORTS (o mais importante) ====================

// Exporta funções específicas

/* pub use message_functions::{
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
}; */

// exporta todas as funções de input de uma vez

/* pub use date_functions::{
    current_year,
    chrono_now_str,
};


pub use path_functions::*;
 */
