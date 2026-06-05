use std::fs;
use std::path::Path;
use suppaftp::FtpStream;
use crate::logger;
use crate::projects::Project;

pub fn run(project: &Project) -> Result<(), String> {
    let output_dir = Path::new(&project.publish_folder).join(&project.name);

    if !output_dir.exists() {
        return Err(format!(
            "Pasta publicada não encontrada: {}",
            output_dir.display()
        ));
    }

    let ftp = &project.ftp_settings;
    let host_port = format!("{}:{}", ftp.ftp_host, ftp.ftp_port);

    logger::info(&format!("Conectando ao FTP: {}", host_port));

    let mut stream = FtpStream::connect(&host_port)
        .map_err(|e| format!("Erro ao conectar FTP: {}", e))?;

    stream
        .login(&ftp.ftp_user, &ftp.ftp_password)
        .map_err(|e| format!("Erro ao autenticar FTP: {}", e))?;

    logger::info("Conectado. Limpando /wwwroot...");
    clean_remote_wwwroot(&mut stream)?;

    logger::info("Enviando arquivos...");
    upload_dir(&mut stream, &output_dir, "/wwwroot")?;

    stream.quit().ok();
    logger::info("Upload FTP concluído.");

    Ok(())
}

fn clean_remote_wwwroot(stream: &mut FtpStream) -> Result<(), String> {
    let entries = stream
        .nlst(Some("/wwwroot"))
        .unwrap_or_default();

    for entry in entries {
        // tenta remover como arquivo; se falhar, ignora (pode ser pasta)
        stream.rm(&entry).ok();
    }

    Ok(())
}

fn upload_dir(
    stream: &mut FtpStream,
    local_dir: &Path,
    remote_dir: &str,
) -> Result<(), String> {
    // garante que a pasta remota existe
    stream.mkdir(remote_dir).ok();
    stream
        .cwd(remote_dir)
        .map_err(|e| format!("Erro ao navegar para {}: {}", remote_dir, e))?;

    let entries = fs::read_dir(local_dir)
        .map_err(|e| format!("Erro ao ler pasta local {}: {}", local_dir.display(), e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            upload_dir(stream, &path, &name)?;
            stream.cdup().map_err(|e| format!("Erro ao voltar pasta FTP: {}", e))?;
        } else {
            let mut file = fs::File::open(&path)
                .map_err(|e| format!("Erro ao abrir arquivo {}: {}", name, e))?;

            stream
                .put_file(&name, &mut file)
                .map_err(|e| {
                    logger::warn(&format!("Falha ao enviar {}: {}", name, e));
                    e.to_string()
                })
                .ok(); // falhas individuais são logadas mas não abortam

            logger::info(&format!("  ✔ {}/{}", remote_dir, name));
        }
    }

    Ok(())
}