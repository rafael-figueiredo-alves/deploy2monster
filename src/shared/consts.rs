use std::env;

pub const APP_NAME: &str = "Deploy2Monster";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CONFIG_FILENAME: &str = "deploy2monster.cfg";
pub const KEY_SIZE: usize = 32; // AES-256 = 32 bytes

pub const NONCE_SIZE: usize = 12; // AES-GCM padrão