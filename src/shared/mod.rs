// Declara os módulos (arquivos)
pub mod parse_functions;
pub mod message_functions;
pub mod consts;

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

// Ou exporta tudo de uma vez (se preferir):
// pub use parse_functions::*;
// pub use print_functions::*;