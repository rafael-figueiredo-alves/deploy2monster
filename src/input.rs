use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use std::io::{self, Write};

pub fn ask(question: &str) -> Option<String> {
    ask_with_default(question, "")
}

pub fn ask_with_default(question: &str, default: &str) -> Option<String> {
    loop {
        if default.is_empty() {
            print!("  {}: ", question);
        } else {
            print!("  {} [{}]: ", question, default);
        }
        io::stdout().flush().unwrap();

        match read_line_or_esc() {
            None => return None, // ESC pressionado
            Some(input) => {
                let trimmed = input.trim().to_string();

                if trimmed.is_empty() && !default.is_empty() {
                    return Some(default.to_string());
                }

                if !trimmed.is_empty() {
                    return Some(trimmed);
                }

                crate::ui::write_error("Este campo é obrigatório.");
            }
        }
    }
}

pub fn ask_validated<F>(question: &str, validator: F) -> Option<String> where F: Fn(&str) -> Result<(), String>,
{
    loop {
        match ask(question) {
            None => return None, // ESC propagado
            Some(value) => match validator(&value) {
                Ok(_)    => return Some(value),
                Err(msg) => crate::ui::write_error(&msg),
            },
        }
    }
}

pub fn ask_validated_with_default<F>(
    question: &str,
    default: &str,
    validator: F,
) -> Option<String>
where
    F: Fn(&str) -> Result<(), String>,
{
    loop {
        match ask_with_default(question, default) {
            None => return None,
            Some(value) => match validator(&value) {
                Ok(_)    => return Some(value),
                Err(msg) => crate::ui::write_error(&msg),
            },
        }
    }
}

pub fn ask_u16_with_default(question: &str, default: u16) -> Option<u16> {
    loop {
        match ask_with_default(question, &default.to_string()) {
            None => return None,
            Some(input) => match input.parse::<u16>() {
                Ok(v)  => return Some(v),
                Err(_) => crate::ui::write_error("Digite um número válido entre 1 e 65535."),
            },
        }
    }
}

pub fn ask_optional_with_default(question: &str, default: &str) -> Option<String> {
    print!(
        "  {} (opcional) [{}]: ",
        question,
        if default.is_empty() { "vazio" } else { default }
    );
    io::stdout().flush().unwrap();

    match read_line_or_esc() {
        None        => None,
        Some(input) => {
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                Some(default.to_string()) // mantém o valor atual
            } else {
                Some(trimmed)
            }
        }
    }
}

pub fn ask_password_optional(question: &str, current: &str) -> Option<String> {
    match rpassword::prompt_password(format!("  {}: ", question)) {
        Ok(pwd) => {
            let trimmed = pwd.trim().to_string();
            if trimmed.is_empty() {
                Some(current.to_string()) // mantém senha atual
            } else {
                Some(trimmed)
            }
        }
        Err(_) => {
            crate::ui::write_error("Erro ao ler senha.");
            None
        }
    }
}

pub fn ask_optional(question: &str) -> Option<String> {
    print!("  {} (opcional): ", question);
    io::stdout().flush().unwrap();

    match read_line_or_esc() {
        None        => None,
        Some(input) => Some(input.trim().to_string()),
    }
}

pub fn ask_u16(question: &str, default: u16) -> Option<u16> {
    loop {
        match ask_with_default(question, &default.to_string()) {
            None => return None,
            Some(input) => match input.parse::<u16>() {
                Ok(v)  => return Some(v),
                Err(_) => crate::ui::write_error("Digite um número válido entre 1 e 65535."),
            },
        }
    }
}

pub fn ask_password(question: &str) -> Option<String> {
    loop {
        match rpassword::prompt_password(format!("  {}: ", question)) {
            Ok(pwd) if !pwd.trim().is_empty() => return Some(pwd.trim().to_string()),
            Ok(_)  => crate::ui::write_error("Senha é obrigatória."),
            Err(_) => crate::ui::write_error("Erro ao ler senha. Tente novamente."),
        }
    }
}

pub fn ask_confirm(question: &str) -> bool {
    loop {
        print!("  {} (s/n): ", question);
        io::stdout().flush().unwrap();

        match read_line_or_esc() {
            None => return false, // ESC = não
            Some(input) => match input.trim().to_lowercase().as_str() {
                "s" | "sim" => return true,
                "n" | "não" | "nao" => return false,
                _ => crate::ui::write_error("Digite 's' para sim ou 'n' para não."),
            },
        }
    }
}

// lê caractere a caractere — retorna None se ESC for pressionado
fn read_line_or_esc() -> Option<String> {
    let mut buffer = String::new();
    let mut stdout = io::stdout();

    terminal::enable_raw_mode().unwrap();

    // desabilita echo explicitamente no Windows
    #[cfg(target_os = "windows")]
    {
        use crossterm::event::DisableMouseCapture;
        use crossterm::execute;
        let _ = execute!(stdout, DisableMouseCapture);
    }

    let result = loop {
        match event::read() {
            Ok(Event::Key(key_event)) => {
                // no Windows, ignora eventos KeyRelease — só processa Press
                #[cfg(target_os = "windows")]
                if key_event.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }

                match key_event.code {
                    KeyCode::Esc => {
                        break None;
                    }
                    KeyCode::Enter => {
                        break Some(buffer);
                    }
                    KeyCode::Backspace => {
                        if !buffer.is_empty() {
                            buffer.pop();
                            print!("\x08 \x08");
                            stdout.flush().unwrap();
                        }
                    }
                    KeyCode::Char(c) => {
                        if !c.is_control() {
                            buffer.push(c);
                            print!("{}", c);
                            stdout.flush().unwrap();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    };

    terminal::disable_raw_mode().unwrap();
    println!();
    result
}