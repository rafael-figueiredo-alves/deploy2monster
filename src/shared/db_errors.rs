use mysql::Error as MySqlError;
use std::fmt;

#[derive(Debug)]
pub struct StatementFailure {
    pub index: usize,
    pub preview: String,
    pub detail: String,
}

#[derive(Debug)]
pub enum DatabaseError {
    ScriptRead { path: String, detail: String },
    Connection { target: String, detail: String },
    QueryFailures { total: usize, failures: Vec<StatementFailure> },
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ScriptRead { path, detail } => {
                write!(f, "Erro ao ler o script SQL '{}': {}", path, detail)
            }
            DatabaseError::Connection { target, detail } => {
                write!(f, "Falha de conexão com o banco '{}': {}", target, detail)
            }
            DatabaseError::QueryFailures { total, failures } => {
                if let Some(first) = failures.first() {
                    write!(
                        f,
                        "A etapa SQL terminou com {} erro(s) em {} statement(s). Primeiro erro no statement {} ('{}'): {}",
                        failures.len(),
                        total,
                        first.index,
                        first.preview,
                        first.detail
                    )
                } else {
                    write!(f, "A etapa SQL terminou com erros em {} statement(s).", total)
                }
            }
        }
    }
}

impl std::error::Error for DatabaseError {}

pub fn friendly_mysql_error(
    error: &MySqlError,
    host: &str,
    port: u16,
    database: &str,
    user: &str,
) -> String {
    let raw = error.to_string();
    let lower = raw.to_lowercase();
    let target = format!("{}:{}/{}", host, port, database);

    if lower.contains("access denied") {
        format!(
            "Acesso negado ao banco '{}', usuário '{}'. Verifique usuário, senha e permissões. Detalhe técnico: {}",
            target, user, raw
        )
    } else if lower.contains("unknown database") {
        format!(
            "O banco '{}' não existe ou não está acessível. Detalhe técnico: {}",
            database, raw
        )
    } else if lower.contains("unknown mysql server host")
        || lower.contains("could not connect to address")
        || lower.contains("can't connect to mysql server")
        || lower.contains("connection refused")
        || lower.contains("timed out")
        || lower.contains("os error 11001")
    {
        format!(
            "Não foi possível conectar ao servidor '{}'. Verifique host, porta, DNS e disponibilidade da rede. Detalhe técnico: {}",
            target, raw
        )
    } else if lower.contains("lost connection") || lower.contains("server has gone away") {
        format!(
            "A conexão com o banco '{}' foi perdida durante a operação. Detalhe técnico: {}",
            target, raw
        )
    } else {
        format!("Falha no banco '{}': {}", target, raw)
    }
}

