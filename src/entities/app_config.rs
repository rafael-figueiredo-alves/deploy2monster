use std::{fs, path::PathBuf};

use crate::shared::consts::{CONFIG_FILENAME, KEY_SIZE};

pub struct AppConfig {
    pub crypto_key: [u8; KEY_SIZE],
}

impl AppConfig {
    pub fn load_or_create() -> Result<Self, String> {
        let path = config_path()?;

        if path.exists() {
            load_config(&path)
        } else {
            create_config(&path)
        }
    }

    pub fn key(&self) -> &[u8; KEY_SIZE] {
        &self.crypto_key
    }
}

fn config_path() -> Result<PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Erro ao obter caminho do executável: {}", e))?;

    Ok(exe.parent()
        .ok_or("Erro ao obter pasta do executável")?
        .join(CONFIG_FILENAME))
}

fn create_config(path: &PathBuf) -> Result<AppConfig, String> {
    use rand::RngCore;

    let mut key = [0u8; KEY_SIZE];
    rand::thread_rng().fill_bytes(&mut key);

    // formato binário: [4 bytes magic] [32 bytes key]
    let mut data = Vec::new();
    data.extend_from_slice(b"D2MC");  // magic header
    data.extend_from_slice(&key);

    fs::write(path, &data)
        .map_err(|e| format!("Erro ao criar arquivo de configuração: {}", e))?;

    crate::shared::message_functions::write_info(&format!(
        "Arquivo de configuração criado em: {}",
        path.display()
    ));

    Ok(AppConfig { crypto_key: key })
}

fn load_config(path: &PathBuf) -> Result<AppConfig, String> {
    let data = fs::read(path)
        .map_err(|e| format!("Erro ao ler arquivo de configuração: {}", e))?;

    // valida magic header
    if data.len() < 4 + KEY_SIZE || &data[..4] != b"D2MC" {
        return Err("Arquivo de configuração inválido ou corrompido.".to_string());
    }

    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(&data[4..4 + KEY_SIZE]);

    Ok(AppConfig { crypto_key: key })
}