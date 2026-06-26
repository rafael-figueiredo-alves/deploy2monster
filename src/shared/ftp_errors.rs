use std::fmt;

#[derive(Debug)]
pub struct FileTransferFailure {
    pub path: String,
    pub detail: String,
}

#[derive(Debug)]
pub enum FtpError {
    RemoteOutputDirMissing { path: String },
    Connect { target: String, detail: String },
    Login { target: String, detail: String },
    ListRemote { path: String, detail: String },
    MakeDir { path: String, detail: String },
    RemoveRemote { path: String, detail: String },
    UploadFailures { total: usize, failures: Vec<FileTransferFailure> },
}

impl fmt::Display for FtpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FtpError::RemoteOutputDirMissing { path } => {
                write!(f, "Pasta publicada não encontrada: {}", path)
            }
            FtpError::Connect { target, detail } => {
                write!(f, "Falha ao conectar ao FTP '{}': {}", target, detail)
            }
            FtpError::Login { target, detail } => {
                write!(f, "Falha na autenticação FTP em '{}': {}", target, detail)
            }
            FtpError::ListRemote { path, detail } => {
                write!(f, "Falha ao listar diretório remoto '{}': {}", path, detail)
            }
            FtpError::MakeDir { path, detail } => {
                write!(f, "Falha ao criar diretório remoto '{}': {}", path, detail)
            }
            FtpError::RemoveRemote { path, detail } => {
                write!(f, "Falha ao remover item remoto '{}': {}", path, detail)
            }
            FtpError::UploadFailures { total, failures } => {
                if let Some(first) = failures.first() {
                    write!(
                        f,
                        "Upload concluído com {} erro(s) em {} arquivo(s). Primeiro erro em '{}': {}",
                        failures.len(),
                        total,
                        first.path,
                        first.detail
                    )
                } else {
                    write!(f, "Upload concluído com erros em {} arquivo(s).", total)
                }
            }
        }
    }
}

impl std::error::Error for FtpError {}

pub fn friendly_ftp_error(detail: &str, target: &str) -> String {
    let lower = detail.to_lowercase();

    if lower.contains("name or service not known")
        || lower.contains("host not found")
        || lower.contains("could not resolve")
        || lower.contains("os error 11001")
        || lower.contains("unknown host")
    {
        format!(
            "Não foi possível resolver o host FTP '{}'. Verifique nome, DNS e conectividade. Detalhe técnico: {}",
            target, detail
        )
    } else if lower.contains("connection refused")
        || lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("network is unreachable")
        || lower.contains("broken pipe")
    {
        format!(
            "Falha ao conectar ao FTP '{}'. Verifique porta, firewall e disponibilidade do servidor. Detalhe técnico: {}",
            target, detail
        )
    } else if lower.contains("authentication")
        || lower.contains("login incorrect")
        || lower.contains("530")
        || lower.contains("not logged in")
    {
        format!(
            "Falha na autenticação FTP em '{}'. Verifique usuário, senha e permissões. Detalhe técnico: {}",
            target, detail
        )
    } else if lower.contains("permission denied") {
        format!(
            "Permissão negada no FTP '{}'. Detalhe técnico: {}",
            target, detail
        )
    } else {
        format!("Falha no FTP '{}': {}", target, detail)
    }
}
