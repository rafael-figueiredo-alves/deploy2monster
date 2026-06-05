use mysql::{Pool, prelude::Queryable};
use crate::logger;
use crate::projects::Project;
use std::fs;

pub fn run(project: &Project) -> Result<(), String> {
    let script_path = &project.sql_script;

    if script_path.is_empty() {
        logger::warn("Nenhum script SQL configurado. Etapa ignorada.");
        return Ok(());
    }

    let content = fs::read_to_string(script_path)
        .map_err(|e| format!("Erro ao ler script SQL '{}': {}", script_path, e))?;

    let url = to_mysql_url(&project.sql_connection);
    logger::info(&format!(
        "Conectando ao banco: mysql://{}:***@{}",
        user_from(&url),
        host_from(&url)
    ));

    let pool = Pool::new(url.as_str())
        .map_err(|e| format!("Erro ao conectar ao banco: {}", e))?;

    let mut conn = pool.get_conn()
        .map_err(|e| format!("Erro ao obter conexão: {}", e))?;

    logger::info("Executando script SQL...");

    let statements = parse_statements(&content);
    let total = statements.len();
    let mut errors = 0;

    for (i, stmt) in statements.iter().enumerate() {
        let preview = preview_stmt(stmt);
        match conn.query_drop(stmt.as_str()) {
            Ok(_) => {
                logger::info(&format!("[{}/{}] OK — {}", i + 1, total, preview));
            }
            Err(e) => {
                errors += 1;
                logger::error(&format!("[{}/{}] ERRO — {}", i + 1, total, preview));
                logger::error(&format!("  Detalhe: {}", e));
            }
        }
    }

    if errors > 0 {
        logger::warn(&format!(
            "Script concluído com {} erro(s) de {} statement(s).",
            errors, total
        ));
    } else {
        logger::info(&format!("Script concluído. {} statement(s) executados.", total));
    }

    Ok(())
}

fn parse_statements(content: &str) -> Vec<String> {
    content
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| {
            let clean = remove_comments(s).trim().to_string();
            !clean.is_empty()
        })
        .collect()
}

fn remove_comments(input: &str) -> String {
    // remove comentários de bloco /* ... */
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(c2) = chars.next() {
                if c2 == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn preview_stmt(stmt: &str) -> String {
    let clean = remove_comments(stmt);
    let first_line = clean.trim().lines().next().unwrap_or("").trim();
    if first_line.len() > 60 {
        format!("{}...", &first_line[..60])
    } else {
        first_line.to_string()
    }
}

fn to_mysql_url(connection_string: &str) -> String {
    if connection_string.starts_with("mysql://") {
        return connection_string.to_string();
    }

    let mut server   = "localhost".to_string();
    let mut database = String::new();
    let mut user     = String::new();
    let mut password = String::new();
    let mut port     = "3306".to_string();

    for part in connection_string.split(';') {
        let part = part.trim();
        if part.is_empty() { continue; }

        // splitn(2) garante que valores com '=' (como senhas) não sejam cortados
        let mut kv = part.splitn(2, '=');
        let key = kv.next().unwrap_or("").trim().to_lowercase();
        let val = kv.next().unwrap_or("").trim().to_string();

        match key.as_str() {
            "server" | "host"     => server   = val,
            "database" | "db"     => database = val,
            "uid" | "user"        => user     = val,
            "pwd" | "password"    => password = val,
            "port"                => port     = val,
            _                     => {} // ignora SslMode e outros
        }
    }

    // encode de caracteres especiais na senha
    let password_encoded = encode_password(&password);

    format!(
        "mysql://{}:{}@{}:{}/{}",
        user, password_encoded, server, port, database
    )
}

fn user_from(url: &str) -> &str {
    url.splitn(3, '/').nth(2)
        .and_then(|s| s.split(':').next())
        .unwrap_or("?")
}

fn host_from(url: &str) -> &str {
    url.splitn(2, '@').nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap_or("?")
}

fn encode_password(pwd: &str) -> String {
    // caracteres que quebram a URL precisam de percent-encoding
    pwd.chars().map(|c| match c {
        '@'  => "%40".to_string(),
        '/'  => "%2F".to_string(),
        '?'  => "%3F".to_string(),
        '#'  => "%23".to_string(),
        '!'  => "%21".to_string(),
        '='  => "%3D".to_string(),
        '+'  => "%2B".to_string(),
        ' '  => "%20".to_string(),
        c    => c.to_string(),
    }).collect()
}