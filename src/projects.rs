use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::crypto;

use crate::{input};

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

pub fn create_project_interactive(name: &str, key: &[u8; 32]) -> Result<(), String> {
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

    save_project(&project, &project_path, key)?;
    crate::ui::write_success(&format!("Projeto '{}' criado com sucesso!", name));
    Ok(())
}

fn save_project(project: &Project, path: &PathBuf, key: &[u8; 32]) -> Result<(), String> {
    // cria uma versão com senhas criptografadas para gravar
    let to_save = Project {
        name:           project.name.clone(),
        publish_folder: project.publish_folder.clone(),
        project_file:   project.project_file.clone(),
        sql_script:     project.sql_script.clone(),
        ftp_settings: FtpSettings {
            ftp_host:     project.ftp_settings.ftp_host.clone(),
            ftp_port:     project.ftp_settings.ftp_port,
            ftp_user:     project.ftp_settings.ftp_user.clone(),
            ftp_password: crypto::encrypt(&project.ftp_settings.ftp_password, key)?,
        },
        database_settings: DatabaseSettings {
            host:     project.database_settings.host.clone(),
            port:     project.database_settings.port,
            user:     project.database_settings.user.clone(),
            password: crypto::encrypt(&project.database_settings.password, key)?,
            database: project.database_settings.database.clone(),
        },
    };    
    
    let json = serde_json::to_string_pretty(&to_save)
        .map_err(|e| format!("Erro ao serializar projeto: {}", e))?;

    fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| format!("Erro ao criar pasta projects: {}", e))?;

    fs::write(path, json)
        .map_err(|e| format!("Erro ao gravar arquivo: {}", e))?;

    println!();
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

pub fn load_project(name: &str, key: &[u8; 32]) -> Result<Project, String> {
    let mut project = find_project_case_insensitive(name)?;

    project.ftp_settings.ftp_password =
        crypto::decrypt(&project.ftp_settings.ftp_password, key)?;

    project.database_settings.password =
        crypto::decrypt(&project.database_settings.password, key)?;

    Ok(project)
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

pub fn list_projects() -> Result<Vec<String>, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Erro ao obter caminho do executável: {}", e))?;

    let projects_dir = exe_path.parent()
        .ok_or_else(|| "Não foi possível determinar o diretório do executável".to_string())?
        .join("projects");

    if !projects_dir.exists() {
        return Ok(vec![]); // sem pasta, sem projetos
    }

    let mut projects = vec![];

    for entry in std::fs::read_dir(&projects_dir)
        .map_err(|e| format!("Erro ao ler pasta 'projects': {}", e))? 
    {
        let entry = entry.map_err(|e| format!("Erro ao acessar entrada de projeto: {}", e))?;
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.ends_with(".d2mproj") {
            let name = fname.trim_end_matches(".d2mproj").to_string();
            projects.push(name);
        }
    }

    Ok(projects)
}

pub fn edit_project_interactive(name: &str, key: &[u8; 32]) -> Result<(), String> {
    let project_path = resolve_project_path(name)?;
    let project = find_project_case_insensitive(name)?;

    macro_rules! ask_or_cancel {
        ($expr:expr) => {
            match $expr {
                Some(v) => v,
                None => {
                    crate::ui::write_warning("Edição cancelada. Nenhuma alteração foi salva.");
                    return Ok(());
                }
            }
        };
    }

    println!("→ Editando projeto: {}...", project.name);
    println!();
    println!("  (Pressione Enter para manter o valor atual)");
    println!("  (Pressione ESC a qualquer momento para cancelar)");
    println!();

    println!("  — Geral —");
    let project_file = ask_or_cancel!(input::ask_validated_with_default(
        "Caminho do arquivo .csproj",
        &project.project_file,
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

    let publish_folder = ask_or_cancel!(input::ask_validated_with_default(
        "Pasta de publicação",
        &project.publish_folder,
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
    let ftp_host = ask_or_cancel!(input::ask_validated_with_default(
        "Host FTP",
        &project.ftp_settings.ftp_host,
        |v| {
            if v.is_empty() { Err("Host FTP é obrigatório.".to_string()) } else { Ok(()) }
        },
    ));

    let ftp_port     = ask_or_cancel!(input::ask_u16_with_default(
        "Porta FTP",
        project.ftp_settings.ftp_port
    ));

    let ftp_user     = ask_or_cancel!(input::ask_validated_with_default(
        "Usuário FTP",
        &project.ftp_settings.ftp_user,
        |v| {
            if v.is_empty() { Err("Usuário FTP é obrigatório.".to_string()) } else { Ok(()) }
        },
    ));

    println!("  Senha FTP atual: (oculta) — pressione Enter para manter ou digite nova senha");
    let ftp_password = ask_or_cancel!(input::ask_password_optional(
        "Nova senha FTP",
        &project.ftp_settings.ftp_password
    ));

    println!();
    println!("  — Banco de Dados —");
    let db_host = ask_or_cancel!(input::ask_validated_with_default(
        "Endereço do banco de dados",
        &project.database_settings.host,
        |v| {
            if v.is_empty() { Err("Endereço do banco é obrigatório.".to_string()) } else { Ok(()) }
        },
    ));

    let db_port = ask_or_cancel!(input::ask_u16_with_default(
        "Porta do banco de dados",
        project.database_settings.port
    ));

    let db_user = ask_or_cancel!(input::ask_validated_with_default(
        "Usuário do banco de dados",
        &project.database_settings.user,
        |v| {
            if v.is_empty() { Err("Usuário do banco é obrigatório.".to_string()) } else { Ok(()) }
        },
    ));

    println!("  Senha do banco atual: (oculta) — pressione Enter para manter ou digite nova senha");
    let db_password = ask_or_cancel!(input::ask_password_optional(
        "Nova senha do banco",
        &project.database_settings.password
    ));

    let db_database = ask_or_cancel!(input::ask_validated_with_default(
        "Nome do banco de dados",
        &project.database_settings.database,
        |v| {
            if v.is_empty() { Err("Nome do banco é obrigatório.".to_string()) } else { Ok(()) }
        },
    ));

    let sql_script = loop {
        match input::ask_optional_with_default(
            "Caminho do script SQL (.sql ou .txt)",
            &project.sql_script,
        ) {
            None => {
                crate::ui::write_warning("Edição cancelada. Nenhuma alteração foi salva.");
                return Ok(());
            }
            Some(path) if path.is_empty() => break path.to_string(),
            Some(path) => match validate_sql_script(&path) {
                Ok(_) => {
                    crate::ui::write_success("Script SQL validado.");
                    break path;
                }
                Err(msg) => crate::ui::write_error(&msg),
            },
        }
    };

    let updated = Project {
        name: project.name.clone(),
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

    save_project(&updated, &project_path, key)?;
    crate::ui::write_success(&format!("Projeto '{}' atualizado com sucesso!", updated.name));
    Ok(())
}

pub fn export_project(name: &str, dest_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let project = find_project_case_insensitive(name)?;

    let dest = std::path::Path::new(dest_path);

    // se for pasta, usa nome do arquivo original
    let dest_file = if dest.is_dir() {
        dest.join(format!("{}.d2mproj", project.name))
    } else if dest.extension().is_some() {
        dest.to_path_buf()
    } else {
        // assume que é pasta mas ainda não existe — cria
        std::fs::create_dir_all(dest)
            .map_err(|e| format!("Erro ao criar pasta de destino: {}", e))?;
        dest.join(format!("{}.d2mproj", project.name))
    };

    let ftp_password = crypto::decrypt(&project.ftp_settings.ftp_password, key)?;
    let db_password = crypto::decrypt(&project.database_settings.password, key)?;

    // exporta com senha ofuscada
    let export = ProjectExport {
        name:              project.name.clone(),
        publish_folder:    project.publish_folder.clone(),
        project_file:      project.project_file.clone(),
        ftp_settings: FtpSettingsExport {
            ftp_host:     project.ftp_settings.ftp_host.clone(),
            ftp_port:     project.ftp_settings.ftp_port,
            ftp_user:     project.ftp_settings.ftp_user.clone(),            
            ftp_password: obfuscate(&ftp_password),
        },
        database_settings: DatabaseSettingsExport {
            host:     project.database_settings.host.clone(),
            port:     project.database_settings.port,
            user:     project.database_settings.user.clone(),
            password: obfuscate(&db_password),
            database: project.database_settings.database.clone(),
        },
        sql_script: project.sql_script.clone(),
    };

    let json = serde_json::to_string_pretty(&export)
        .map_err(|e| format!("Erro ao serializar projeto: {}", e))?;

    std::fs::write(&dest_file, json)
        .map_err(|e| format!("Erro ao gravar arquivo exportado: {}", e))?;

    println!("  Exportado para: {}", dest_file.display());
    Ok(())
}

// ofusca senha — não é criptografia, apenas evita exposição acidental
fn obfuscate(value: &str) -> String {
    let encoded: String = value
        .bytes()
        .map(|b| format!("{:02X}", b ^ 0xAB))
        .collect();
    format!("obf:{}", encoded)
}

// structs de export com senhas ofuscadas — separadas para não afetar o Project original
#[derive(Serialize)]
struct FtpSettingsExport {
    pub ftp_host:     String,
    pub ftp_port:     u16,
    pub ftp_user:     String,
    pub ftp_password: String,
}

#[derive(Serialize)]
struct DatabaseSettingsExport {
    pub host:     String,
    pub port:     u16,
    pub user:     String,
    pub password: String,
    pub database: String,
}

#[derive(Serialize)]
struct ProjectExport {
    pub name:              String,
    pub publish_folder:    String,
    pub project_file:      String,
    pub ftp_settings:      FtpSettingsExport,
    pub database_settings: DatabaseSettingsExport,
    pub sql_script:        String,
}