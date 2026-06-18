pub fn write_error(msg: &str) {
    eprintln!("\x1b[31m  ✘ {}\x1b[0m", msg); //Mensagem em vermelho
}

pub fn write_success(msg: &str) {
    println!("\x1b[32m  ✔ {}\x1b[0m", msg); //Mensagem em verde
}

pub fn write_warning(msg: &str) {
    println!("\x1b[33m  ⚠ {}\x1b[0m", msg); //Mensagem em amarelo
}

pub fn write_info(msg: &str) {
    println!("  → {}", msg); //Não usa cor diferente do padrão do console
}