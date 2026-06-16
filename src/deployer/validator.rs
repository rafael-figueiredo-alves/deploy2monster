use crate::projects::Project;
use std::path::Path;

pub struct ValidationResult {
    pub errors:   Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors:   Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

pub fn validate(project: &Project) -> ValidationResult {
    let mut result = ValidationResult::new();

    // — Geral —
    if project.project_file.is_empty() {
        result.errors.push("Caminho do arquivo .csproj não configurado.".to_string());
    } else if !Path::new(&project.project_file).exists() {
        result.errors.push(format!(
            "Arquivo .csproj não encontrado: {}",
            project.project_file
        ));
    }

    if project.publish_folder.is_empty() {
        result.errors.push("Pasta de publicação não configurada.".to_string());
    } else if !Path::new(&project.publish_folder).exists() {
        result.errors.push(format!(
            "Pasta de publicação não encontrada: {}",
            project.publish_folder
        ));
    }

    // — FTP —
    if project.ftp_settings.ftp_host.is_empty() {
        result.errors.push("Host FTP não configurado.".to_string());
    }

    if project.ftp_settings.ftp_user.is_empty() {
        result.errors.push("Usuário FTP não configurado.".to_string());
    }

    if project.ftp_settings.ftp_password.is_empty() {
        result.errors.push("Senha FTP não configurada.".to_string());
    }

    // — Banco —
    if project.database_settings.host.is_empty() {
        result.errors.push("Host do banco de dados não configurado.".to_string());
    }

    if project.database_settings.user.is_empty() {
        result.errors.push("Usuário do banco de dados não configurado.".to_string());
    }

    if project.database_settings.password.is_empty() {
        result.errors.push("Senha do banco de dados não configurada.".to_string());
    }

    if project.database_settings.database.is_empty() {
        result.errors.push("Nome do banco de dados não configurado.".to_string());
    }

    // — Warnings —
    if project.sql_script.is_empty() {
        result.warnings.push("Script SQL não configurado — etapa de banco será ignorada.".to_string());
    } else if !Path::new(&project.sql_script).exists() {
        result.warnings.push(format!(
            "Script SQL não encontrado: {} — etapa de banco será ignorada.",
            project.sql_script
        ));
    }

    result
}