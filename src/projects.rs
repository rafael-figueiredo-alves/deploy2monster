use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::{input, ui::write_success};

#[derive(Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

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
    pub database_settings: DatabaseSettings,
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

    // macro local para propagar cancelamento via ESC
    macro_rules! ask_or_cancel {
        ($expr:expr) => {
            match $expr {
                Some(v) => v,
                None => {
                    crate::ui::write_warning("Criação de projeto cancelada.");
                    return Ok(());
                }
            }
        };
    }

    println!("→ Criando novo projeto: {}...", name);
    println!();
    println!("  (Pressione Enter para aceitar o valor padrão quando disponível)");
    println!("  (Pressione ESC a qualquer momento para cancelar)");
    println!();

    println!("  — Geral —");
    let project_file = ask_or_cancel!(input::ask_validated(
        "Caminho do arquivo .csproj",
        |v| {
            if !std::path::Path::new(v).exists() {
                Err(format!("Arquivo não encontrado: {}", v))
            } else if !v.ends_with(".csproj") {
                Err("O arquivo deve ter extensão .csproj".to_string())
            } else {
                Ok(())
            }
        },
    ));

    let publish_folder = ask_or_cancel!(input::ask_validated(
        "Pasta de publicação",
        |v| {
            if !std::path::Path::new(v).exists() {
                Err(format!("Pasta não encontrada: {}", v))
            } else {
                Ok(())
            }
        },
    ));

    println!();
    println!("  — FTP —");
    let ftp_host = ask_or_cancel!(input::ask_validated(
        "Host FTP",
        |v| {
            if v.is_empty() {
                Err("Host FTP é obrigatório.".to_string())
            } else {
                Ok(())
            }
        },
    ));

    let ftp_port     = ask_or_cancel!(input::ask_u16("Porta FTP", 21));
    let ftp_user     = ask_or_cancel!(input::ask_validated("Usuário FTP", |v| {
        if v.is_empty() { Err("Usuário FTP é obrigatório.".to_string()) } else { Ok(()) }
    }));
    let ftp_password = ask_or_cancel!(input::ask_password("Senha FTP"));

    println!();
    println!("  — Banco de Dados —");
    let db_host = ask_or_cancel!(input::ask_validated("Endereço do banco de dados", |v| {
        if v.is_empty() { Err("Endereço do banco é obrigatório.".to_string()) } else { Ok(()) }
    }));

    let db_port     = ask_or_cancel!(input::ask_u16("Porta do banco de dados", 3306));
    let db_user     = ask_or_cancel!(input::ask_validated("Usuário do banco de dados", |v| {
        if v.is_empty() { Err("Usuário do banco é obrigatório.".to_string()) } else { Ok(()) }
    }));
    let db_password = ask_or_cancel!(input::ask_password("Senha do banco de dados"));
    let db_database = ask_or_cancel!(input::ask_validated("Nome do banco de dados", |v| {
        if v.is_empty() { Err("Nome do banco é obrigatório.".to_string()) } else { Ok(()) }
    }));

    let sql_script = loop {
        match input::ask_optional("Caminho do script SQL (.sql ou .txt)") {
            None => {
                crate::ui::write_warning("Criação de projeto cancelada.");
                return Ok(());
            }
            Some(path) if path.is_empty() => break path,
            Some(path) => match validate_sql_script(&path) {
                Ok(_) => {
                    crate::ui::write_success("Script SQL validado.");
                    break path;
                }
                Err(msg) => crate::ui::write_error(&msg),
            },
        }
    };

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
        database_settings: DatabaseSettings {
            host:     db_host,
            port:     db_port,
            user:     db_user,
            password: db_password,
            database: db_database,
        },
        sql_script,
    };

    save_project(&project, &project_path)?;
    crate::ui::write_success(&format!("Projeto '{}' criado com sucesso!", name));
    Ok(())
}

fn save_project(project: &Project, path: &PathBuf) -> Result<(), String> {
    let json = serde_json::to_string_pretty(project)
        .map_err(|e| format!("Erro ao serializar projeto: {}", e))?;

    fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| format!("Erro ao criar pasta projects: {}", e))?;

    fs::write(path, json)
        .map_err(|e| format!("Erro ao gravar arquivo: {}", e))?;

    println!();
    write_success(&format!(" Projeto '{}' criado em: {}", project.name, path.display()));
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

fn validate_sql_script(path: &str) -> Result<(), String> {
    use std::path::Path;

    let p = Path::new(path);

    // verifica extensão
    match p.extension().and_then(|e| e.to_str()) {
        Some("sql") | Some("txt") => {}
        _ => return Err("O script deve ser um arquivo .sql ou .txt".to_string()),
    }

    // verifica se existe
    if !p.exists() {
        return Err(format!("Arquivo não encontrado: {}", path));
    }

    // verifica se não está vazio
    let content = std::fs::read_to_string(p)
        .map_err(|e| format!("Erro ao ler o arquivo: {}", e))?;

    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("O arquivo de script está vazio.".to_string());
    }

    // verifica se contém ao menos um statement SQL válido
    let has_valid_statement = trimmed
        .split(';')
        .map(|s| s.trim().to_lowercase())
        .any(|s| {
            s.starts_with("create")
                || s.starts_with("alter")
                || s.starts_with("drop")
                || s.starts_with("insert")
                || s.starts_with("update")
                || s.starts_with("delete")
                || s.starts_with("select")
        });

    if !has_valid_statement {
        return Err("O arquivo não parece conter statements SQL válidos.".to_string());
    }

    Ok(())
}