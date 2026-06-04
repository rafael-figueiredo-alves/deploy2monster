use std::io::{self, Write};

pub fn ask(question: &str) -> String {
    ask_with_default(question, "")
}

pub fn ask_with_default(question: &str, default: &str) -> String {
    let prompt = if default.is_empty() {
        format!("{}: ", question)
    } else {
        format!("{} [{}]: ", question, default)
    };

    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_string();

    if input.is_empty() {
        default.to_string()
    } else {
        input.to_string()
    }
}

pub fn ask_u16(question: &str, default: u16) -> u16 {
    loop {
        let input = ask_with_default(question, &default.to_string());
        match input.parse::<u16>() {
            Ok(v)  => return v,
            Err(_) => eprintln!("  ✘ Digite um número válido."),
        }
    }
}