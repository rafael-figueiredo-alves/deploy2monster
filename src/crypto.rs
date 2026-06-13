use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;

const NONCE_SIZE: usize = 12; // AES-GCM padrão

pub fn encrypt(value: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new(key.into());

    // nonce aleatório a cada cifragem — garante que a mesma senha
    // gera textos cifrados diferentes a cada vez
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, value.as_bytes())
        .map_err(|e| format!("Erro ao criptografar: {}", e))?;

    // salva: [12 bytes nonce][N bytes ciphertext] → hex string
    let mut combined = Vec::new();
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(hex::encode(combined))
}

pub fn decrypt(encoded: &str, key: &[u8; 32]) -> Result<String, String> {
    let data = hex::decode(encoded)
        .map_err(|_| "Valor criptografado inválido.".to_string())?;

    if data.len() < NONCE_SIZE {
        return Err("Dados criptografados muito curtos.".to_string());
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(key.into());

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Falha ao descriptografar. Chave incorreta ou dados corrompidos.".to_string())?;

    String::from_utf8(plaintext)
        .map_err(|_| "Erro ao converter texto descriptografado.".to_string())
}