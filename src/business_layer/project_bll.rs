use crate::entities::project::Project;
use crate::shared::{resolve_project_path};
use crate::shared::message_functions::*;
use crate::shared::input_functions::*;
use crate::shared::crypto_functions::*;
use std::path::PathBuf;

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
                    write_warning("Criação de projeto cancelada.");
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
    let project_file = ask_or_cancel!(ask_validated(
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

    let publish_folder = ask_or_cancel!(ask_validated(
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
    let ftp_host = ask_or_cancel!(ask_validated(
        "Host FTP",
        |v| {
            if v.is_empty() {
                Err("Host FTP é obrigatório.".to_string())
            } else {
                Ok(())
            }
        },
    ));

    let ftp_port     = ask_or_cancel!(ask_u16("Porta FTP", 21));
    let ftp_user     = ask_or_cancel!(ask_validated("Usuário FTP", |v| {
        if v.is_empty() { Err("Usuário FTP é obrigatório.".to_string()) } else { Ok(()) }
    }));
    let ftp_password = ask_or_cancel!(ask_password("Senha FTP"));

    println!();
    println!("  — Banco de Dados —");
    let db_host = ask_or_cancel!(ask_validated("Endereço do banco de dados", |v| {
        if v.is_empty() { Err("Endereço do banco é obrigatório.".to_string()) } else { Ok(()) }
    }));

    let db_port     = ask_or_cancel!(ask_u16("Porta do banco de dados", 3306));
    let db_user     = ask_or_cancel!(ask_validated("Usuário do banco de dados", |v| {
        if v.is_empty() { Err("Usuário do banco é obrigatório.".to_string()) } else { Ok(()) }
    }));
    let db_password = ask_or_cancel!(ask_password("Senha do banco de dados"));
    let db_database = ask_or_cancel!(ask_validated("Nome do banco de dados", |v| {
        if v.is_empty() { Err("Nome do banco é obrigatório.".to_string()) } else { Ok(()) }
    }));

    let sql_script = loop {
        match ask_optional("Caminho do script SQL (.sql ou .txt)") {
            None => {
                write_warning("Criação de projeto cancelada.");
                return Ok(());
            }
            Some(path) if path.is_empty() => break path,
            Some(path) => match validate_sql_script(&path) {
                Ok(_) => {
                    write_success("Script SQL validado.");
                    break path;
                }
                Err(msg) => write_error(&msg),
            },
        }
    };

    let project = Project::builder()
        .name(name.to_string())
        .publish_folder(publish_folder)
        .project_file(project_file)
        .ftp_settings()
            .ftp_host(ftp_host)
            .ftp_port(ftp_port)
            .ftp_user(ftp_user)
            .ftp_password(ftp_password)
            .end()
        .database_settings()
            .host(db_host)
            .port(db_port)
            .user(db_user)
            .password(db_password)
            .database(db_database)
            .end()
        .sql_script(sql_script)
        .build()
        .expect("Falha ao construir projeto");

    save_project(&project, &project_path, key)?;
    write_success(&format!("Projeto '{}' criado com sucesso!", name));
    Ok(())
}

fn save_project(project: &Project, path: &PathBuf, key: &[u8; 32]) -> Result<(), String> {
    // cria uma versão com senhas criptografadas para gravar    
    let to_save = Project::builder()
        .name(project.name.to_string())
        .publish_folder(project.publish_folder.to_string())
        .project_file(project.project_file.to_string())
        .ftp_settings()
            .ftp_host(project.ftp_settings.ftp_host.to_string())
            .ftp_port(project.ftp_settings.ftp_port)
            .ftp_user(project.ftp_settings.ftp_user.to_string())
            .ftp_password(encrypt(&project.ftp_settings.ftp_password, key)?)
            .end()
        .database_settings()
            .host(project.database_settings.host.to_string())
            .port(project.database_settings.port)
            .user(project.database_settings.user.to_string())
            .password(encrypt(&project.database_settings.password, key)?)
            .database(project.database_settings.database.to_string())
            .end()
        .sql_script(project.sql_script.to_string())
        .build()
        .expect("Falha ao construir projeto");    

    let json = serde_json::to_string_pretty(&to_save)
        .map_err(|e| format!("Erro ao serializar projeto: {}", e))?;


        std::fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| format!("Erro ao criar pasta projects: {}", e))?;

    std::fs::write(path, json)
        .map_err(|e| format!("Erro ao gravar arquivo: {}", e))?;

    println!();
    Ok(())
}

pub fn load_project(name: &str, key: &[u8; 32]) -> Result<Project, String> {
    let mut project = find_project_case_insensitive(name)?;

    project.ftp_settings.ftp_password =
        decrypt(&project.ftp_settings.ftp_password, key)?;

    project.database_settings.password =
        decrypt(&project.database_settings.password, key)?;

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
                    write_warning("Edição cancelada. Nenhuma alteração foi salva.");
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
    let project_file = ask_or_cancel!(ask_validated_with_default(
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

    let publish_folder = ask_or_cancel!(ask_validated_with_default(
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
    let ftp_host = ask_or_cancel!(ask_validated_with_default(
        "Host FTP",
        &project.ftp_settings.ftp_host,
        |v| {
            if v.is_empty() { Err("Host FTP é obrigatório.".to_string()) } else { Ok(()) }
        },
    ));

    let ftp_port     = ask_or_cancel!(ask_u16_with_default(
        "Porta FTP",
        project.ftp_settings.ftp_port
    ));

    let ftp_user     = ask_or_cancel!(ask_validated_with_default(
        "Usuário FTP",
        &project.ftp_settings.ftp_user,
        |v| {
            if v.is_empty() { Err("Usuário FTP é obrigatório.".to_string()) } else { Ok(()) }
        },
    ));

    println!("  Senha FTP atual: (oculta) — pressione Enter para manter ou digite nova senha");
    let ftp_password = ask_or_cancel!(ask_password_optional(
        "Nova senha FTP",
        &project.ftp_settings.ftp_password
    ));

    println!();
    println!("  — Banco de Dados —");
    let db_host = ask_or_cancel!(ask_validated_with_default(
        "Endereço do banco de dados",
        &project.database_settings.host,
        |v| {
            if v.is_empty() { Err("Endereço do banco é obrigatório.".to_string()) } else { Ok(()) }
        },
    ));

    let db_port = ask_or_cancel!(ask_u16_with_default(
        "Porta do banco de dados",
        project.database_settings.port
    ));

    let db_user = ask_or_cancel!(ask_validated_with_default(
        "Usuário do banco de dados",
        &project.database_settings.user,
        |v| {
            if v.is_empty() { Err("Usuário do banco é obrigatório.".to_string()) } else { Ok(()) }
        },
    ));

    println!("  Senha do banco atual: (oculta) — pressione Enter para manter ou digite nova senha");
    let db_password = ask_or_cancel!(ask_password_optional(
        "Nova senha do banco",
        &project.database_settings.password
    ));

    let db_database = ask_or_cancel!(ask_validated_with_default(
        "Nome do banco de dados",
        &project.database_settings.database,
        |v| {
            if v.is_empty() { Err("Nome do banco é obrigatório.".to_string()) } else { Ok(()) }
        },
    ));

    let sql_script = loop {
        match ask_optional_with_default(
            "Caminho do script SQL (.sql ou .txt)",
            &project.sql_script,
        ) {
            None => {
                write_warning("Edição cancelada. Nenhuma alteração foi salva.");
                return Ok(());
            }
            Some(path) if path.is_empty() => break path.to_string(),
            Some(path) => match validate_sql_script(&path) {
                Ok(_) => {
                    write_success("Script SQL validado.");
                    break path;
                }
                Err(msg) => write_error(&msg),
            },
        }
    };

    let updated = Project::builder()
        .name(name.to_string())
        .publish_folder(publish_folder)
        .project_file(project_file)
        .ftp_settings()
            .ftp_host(ftp_host)
            .ftp_port(ftp_port)
            .ftp_user(ftp_user)
            .ftp_password(ftp_password)
            .end()
        .database_settings()
            .host(db_host)
            .port(db_port)
            .user(db_user)
            .password(db_password)
            .database(db_database)
            .end()
        .sql_script(sql_script)
        .build()
        .expect("Falha ao construir projeto");    

    save_project(&updated, &project_path, key)?;
    write_success(&format!("Projeto '{}' atualizado com sucesso!", updated.name));
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

    let ftp_password = decrypt(&project.ftp_settings.ftp_password, key)?;
    let db_password = decrypt(&project.database_settings.password, key)?;

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

pub fn import_project(file_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let path = std::path::Path::new(file_path);

    // valida existência e extensão
    if !path.exists() {
        return Err(format!("Arquivo não encontrado: {}", file_path));
    }

    match path.extension().and_then(|e| e.to_str()) {
        Some("d2mproj") => {}
        _ => return Err("O arquivo deve ter extensão .d2mproj".to_string()),
    }

    // lê e desserializa como ProjectExport (formato ofuscado)
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Erro ao ler arquivo: {}", e))?;

    let exported: ProjectExport = serde_json::from_str(&content)
        .map_err(|e| format!("Erro ao interpretar arquivo de projeto: {}", e))?;

    // verifica se já existe projeto com mesmo nome
    let dest_path = resolve_project_path(&exported.name)?;
    if dest_path.exists() {
        return Err(format!(
            "Já existe um projeto com o nome '{}'. Remova-o antes de importar.",
            exported.name
        ));
    }

    // desofusca as senhas
    let ftp_password_plain = deobfuscate(&exported.ftp_settings.ftp_password)?;
    let db_password_plain  = deobfuscate(&exported.database_settings.password)?;

    // monta projeto com senhas descriptografadas para re-criptografar
    let project = Project {
        name:           exported.name.clone(),
        publish_folder: exported.publish_folder.clone(),
        project_file:   exported.project_file.clone(),
        sql_script:     exported.sql_script.clone(),
        ftp_settings: FtpSettings {
            ftp_host:     exported.ftp_settings.ftp_host.clone(),
            ftp_port:     exported.ftp_settings.ftp_port,
            ftp_user:     exported.ftp_settings.ftp_user.clone(),
            ftp_password: ftp_password_plain,
        },
        database_settings: DatabaseSettings {
            host:     exported.database_settings.host.clone(),
            port:     exported.database_settings.port,
            user:     exported.database_settings.user.clone(),
            password: db_password_plain,
            database: exported.database_settings.database.clone(),
        },
    };

    // salva já com a criptografia local
    save_project(&project, &dest_path, key)?;

    Ok(())
}

pub fn delete_project(name: &str) -> Result<(), String> {
    let path = resolve_project_path(name)
        .or_else(|_| find_project_path_case_insensitive(name))?;

    if !path.exists() {
        return Err(format!("Projeto '{}' não encontrado.", name));
    }

    std::fs::remove_file(&path)
        .map_err(|e| format!("Erro ao remover arquivo: {}", e))?;

    Ok(())
}

fn find_project_path_case_insensitive(name: &str) -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Erro ao obter executável: {}", e))?;

    let projects_dir = exe_path
        .parent()
        .ok_or("Erro ao obter pasta do executável")?
        .join("projects");

    let name_lower = name.to_lowercase();

    let entries = std::fs::read_dir(&projects_dir)
        .map_err(|_| "Pasta 'projects' não encontrada.".to_string())?;

    for entry in entries.flatten() {
        let fname = entry.file_name().to_string_lossy().to_lowercase();
        let expected = format!("{}.d2mproj", name_lower);
        if fname == expected {
            return Ok(entry.path());
        }
    }

    Err(format!("Projeto '{}' não encontrado.", name))
}

pub fn find_project_name(name: &str) -> Result<String, String> {
    let path = resolve_project_path(name);

    if let Ok(p) = path {
        if p.exists() {
            return Ok(name.to_string());
        }
    }

    // busca case-insensitive e retorna o nome real do arquivo
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Erro ao obter executável: {}", e))?;

    let projects_dir = exe_path
        .parent()
        .ok_or("Erro ao obter pasta do executável")?
        .join("projects");

    let name_lower = name.to_lowercase();

    let entries = std::fs::read_dir(&projects_dir)
        .map_err(|_| "Pasta 'projects' não encontrada.".to_string())?;

    for entry in entries.flatten() {
        let fname = entry.file_name().to_string_lossy().to_lowercase();
        let expected = format!("{}.d2mproj", name_lower);
        if fname == expected {
            let real_name = entry
                .file_name()
                .to_string_lossy()
                .trim_end_matches(".d2mproj")
                .to_string();
            return Ok(real_name);
        }
    }

    Err(format!("Projeto '{}' não encontrado.", name))
}

// ofusca senha — não é criptografia, apenas evita exposição acidental
fn obfuscate(value: &str) -> String {
    let encoded: String = value
        .bytes()
        .map(|b| format!("{:02X}", b ^ 0xAB))
        .collect();
    format!("obf:{}", encoded)
}

fn deobfuscate(value: &str) -> Result<String, String> {
    let hex_part = value
        .strip_prefix("obf:")
        .ok_or("Valor não está no formato ofuscado esperado.")?;

    let bytes: Result<Vec<u8>, _> = (0..hex_part.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex_part[i..i + 2], 16)
                .map(|b| b ^ 0xAB)
        })
        .collect();

    let bytes = bytes.map_err(|_| "Erro ao decodificar valor ofuscado.".to_string())?;

    String::from_utf8(bytes)
        .map_err(|_| "Erro ao converter bytes deofuscados.".to_string())
}
