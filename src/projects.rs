use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::input;

#[derive(Serialize, Deserialize)]
pub struct FtpSettings {
    pub ftp_host: String,
    pub ftp_port: u16,
    pub ftp_user: String,
    pub ftp_password: String,    
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub publish_folder: String,
    pub project_file:   String,
    pub ftp_settings: FtpSettings,
    pub sql_connection: String,
    pub sql_script: String,
}

pub fn create_project_interactive(name: &str) -> Result<(), String> {
    let project_path = resolve_project_path(name)?;

    if project_path.exists() {
        return Err(format!(
            "Projeto '{}' já existe em: {}",
            name,
            project_path.display()
        ));
    }

    println!("→ Criando novo projeto: {} ... ", name);
    println!();
    println!("  Configurando projeto '{}'...", name);
    println!("  (Pressione Enter para aceitar o valor padrão quando disponível)");
    println!();

    println!("  — Geral —");
    let publish_folder = input::ask("Pasta de publicação");
    let project_file   = input::ask("Caminho do arquivo .csproj");

    println!();
    println!("  — FTP —");
    let ftp_host     = input::ask("Host FTP");
    let ftp_port     = input::ask_u16("Porta FTP", 21);
    let ftp_user     = input::ask("Usuário FTP");
    let ftp_password = input::ask("Senha FTP");

    println!();
    println!("  — Banco de Dados —");
    let sql_connection = input::ask("Connection string");
    let sql_script     = input::ask("Caminho do script SQL");

    let project = Project {
        name: name.to_string(),
        publish_folder,
        project_file,
        ftp_settings: FtpSettings {
            ftp_host,
            ftp_port,
            ftp_user,
            ftp_password,
        },
        sql_connection,
        sql_script,
    };

    save_project(&project, &project_path)
}

fn save_project(project: &Project, path: &PathBuf) -> Result<(), String> {
    let json = serde_json::to_string_pretty(project)
        .map_err(|e| format!("Erro ao serializar projeto: {}", e))?;

    fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| format!("Erro ao criar pasta projects: {}", e))?;

    fs::write(path, json)
        .map_err(|e| format!("Erro ao gravar arquivo: {}", e))?;

    println!();
    println!("  ✔ Projeto '{}' criado em: {}", project.name, path.display());
    Ok(())
}

fn resolve_project_path(name: &str) -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Erro ao obter caminho do executável: {}", e))?;

    let projects_dir = exe_path.parent()
        .ok_or_else(|| "Não foi possível determinar o diretório do executável".to_string())?
        .join("projects");

    Ok(projects_dir.join(format!("{}.d2mproj", name)))
}

pub fn load_project(name: &str) -> Result<Project, String> {
    let path = resolve_project_path(name)?;

    if !path.exists() {
        // busca case-insensitive
        return find_project_case_insensitive(name);
    }

    read_project_file(&path)
}

fn find_project_case_insensitive(name: &str) -> Result<Project, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Erro ao obter executável: {}", e))?;

    let projects_dir = exe_path
        .parent()
        .ok_or("Erro ao obter pasta do executável")?
        .join("projects");

    let name_lower = name.to_lowercase();

    let entries = std::fs::read_dir(&projects_dir)
        .map_err(|_| format!("Pasta 'projects' não encontrada."))?;

    for entry in entries.flatten() {
        let fname = entry.file_name().to_string_lossy().to_lowercase();
        let expected = format!("{}.d2mproj", name_lower);
        if fname == expected {
            return read_project_file(&entry.path());
        }
    }

    Err(format!("Projeto '{}' não encontrado.", name))
}

fn read_project_file(path: &std::path::Path) -> Result<Project, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Erro ao ler arquivo de projeto: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Erro ao interpretar projeto JSON: {}", e))
}