use mysql::{Pool, prelude::Queryable, OptsBuilder};
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

    let db = &project.database_settings;

    logger::info(&format!(
        "Conectando ao banco: {}@{}:{}/{}",
        db.user, db.host, db.port, db.database
    ));

    let opts = OptsBuilder::new()
        .ip_or_hostname(Some(&db.host))
        .tcp_port(db.port)
        .user(Some(&db.user))
        .pass(Some(&db.password))
        .db_name(Some(&db.database));

    let pool = Pool::new(opts)
        .map_err(|e| {
            if e.to_string().contains("Access denied") {
                format!(
                    "Acesso negado ao banco '{}'. Verifique usuário, senha e se o acesso remoto está habilitado no painel.",
                    db.host
                )
            } else {
                format!("Erro ao conectar ao banco: {}", e)
            }
        })?;

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
                logger::error(&format!("         Detalhe: {}", e));
            }
        }
    }

    if errors > 0 {
        logger::warn(&format!(
            "Script concluído com {} erro(s) de {} statement(s).",
            errors, total
        ));
    } else {
        logger::info(&format!(
            "Script concluído. {} statement(s) executados com sucesso.",
            total
        ));
    }

    Ok(())
}

fn parse_statements(content: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut delimiter = ";".to_string();

    for line in content.lines() {
        let trimmed = line.trim();

        // detecta mudança de DELIMITER
        if trimmed.to_uppercase().starts_with("DELIMITER") {
            let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
            if let Some(new_delim) = parts.get(1) {
                // flush do que tinha antes
                let stmt = current.trim().to_string();
                if !stmt.is_empty() {
                    let clean = remove_comments(&stmt).trim().to_string();
                    if !clean.is_empty() {
                        statements.push(stmt);
                    }
                }
                current = String::new();
                delimiter = new_delim.trim().to_string();
            }
            continue;
        }

        current.push_str(line);
        current.push('\n');

        // verifica se a linha termina com o delimiter atual
        if trimmed.ends_with(&delimiter) {
            let stmt = current
                .trim()
                .trim_end_matches(&delimiter as &str)
                .trim()
                .to_string();

            if !stmt.is_empty() {
                let clean = remove_comments(&stmt).trim().to_string();
                if !clean.is_empty() {
                    statements.push(stmt);
                }
            }
            current = String::new();
        }
    }

    // flush final
    let stmt = current.trim().to_string();
    if !stmt.is_empty() {
        let clean = remove_comments(&stmt).trim().to_string();
        if !clean.is_empty() {
            statements.push(stmt);
        }
    }

    statements
}

fn remove_comments(input: &str) -> String {
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