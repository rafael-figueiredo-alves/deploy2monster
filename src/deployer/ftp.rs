use std::fs;
use std::path::{Path, PathBuf};
use suppaftp::FtpStream;
use rayon::prelude::*;
use crate::logger;
use crate::projects::{Project, FtpSettings};

const PARALLEL_CONNECTIONS: usize = 3;

pub fn run(project: &Project) -> Result<(), String> {
    let output_dir = Path::new(&project.publish_folder).join(&project.name);

    if !output_dir.exists() {
        return Err(format!(
            "Pasta publicada não encontrada: {}",
            output_dir.display()
        ));
    }

    let ftp = &project.ftp_settings;

    logger::info("Conectando ao FTP para preparar estrutura de pastas...");
    {
        let mut stream = connect(ftp)?;
        logger::info("Limpando /wwwroot...");
        clean_remote_wwwroot(&mut stream)?;
        logger::info("Criando estrutura de pastas...");
        create_remote_dirs(&mut stream, &output_dir, "/wwwroot")?;
        stream.quit().ok();
    }

    let mut files: Vec<(PathBuf, String)> = Vec::new();
    collect_files(&output_dir, "/wwwroot", &mut files);

    let total = files.len();
    logger::info(&format!(
        "Enviando {} arquivo(s) com {} conexões paralelas...",
        total, PARALLEL_CONNECTIONS
    ));

    let chunk_size = (total + PARALLEL_CONNECTIONS - 1) / PARALLEL_CONNECTIONS;
    let chunks: Vec<&[(PathBuf, String)]> = files.chunks(chunk_size).collect();

    let host     = ftp.ftp_host.clone();
    let port     = ftp.ftp_port;
    let user     = ftp.ftp_user.clone();
    let password = ftp.ftp_password.clone();

    let results: Vec<Result<usize, String>> = chunks
        .par_iter()
        .enumerate()
        .map(|(thread_idx, chunk)| -> Result<usize, String> {
            let host_port = format!("{}:{}", host, port);

            let mut stream = FtpStream::connect(&host_port)
                .map_err(|e| format!("Thread {}: erro ao conectar FTP: {}", thread_idx, e))?;

            stream
                .login(&user, &password)
                .map_err(|e| format!("Thread {}: erro ao autenticar: {}", thread_idx, e))?;

            stream.transfer_type(suppaftp::types::FileType::Binary).ok();

            let mut uploaded = 0usize;
            for (local_path, remote_path) in chunk.iter() {
                let file_name = local_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let remote_dir = remote_path
                    .rsplit_once('/')
                    .map(|(dir, _)| dir)
                    .unwrap_or("/wwwroot");

                if stream.cwd(remote_dir).is_err() {
                    logger::warn(&format!("Erro ao navegar para {}", remote_dir));
                    continue;
                }

                match fs::File::open(local_path) {
                    Ok(mut file) => {
                        match stream.put_file(&file_name, &mut file) {
                            Ok(_) => {
                                uploaded += 1;
                                logger::info(&format!("  ✔ {}", remote_path));
                            }
                            Err(e) => {
                                logger::warn(&format!("  ✘ {} — {}", remote_path, e));
                            }
                        }
                    }
                    Err(e) => {
                        logger::warn(&format!("  ✘ Erro ao abrir {}: {}", file_name, e));
                    }
                }
            }

            stream.quit().ok();
            Ok(uploaded)
        })
        .collect();

    let mut total_uploaded = 0usize;
    let mut total_errors   = 0usize;
    for result in results {
        match result {
            Ok(n)  => total_uploaded += n,
            Err(e) => {
                total_errors += 1;
                logger::error(&e);
            }
        }
    }

    if total_errors > 0 {
        logger::warn(&format!(
            "Upload concluído com problemas: {}/{} arquivos enviados.",
            total_uploaded, total
        ));
    } else {
        logger::info(&format!(
            "Upload concluído: {}/{} arquivos enviados.",
            total_uploaded, total
        ));
    }

    Ok(())
}

fn connect(ftp: &FtpSettings) -> Result<FtpStream, String> {
    let host_port = format!("{}:{}", ftp.ftp_host, ftp.ftp_port);

    let mut stream = FtpStream::connect(&host_port)
        .map_err(|e| format!("Erro ao conectar FTP: {}", e))?;

    stream
        .login(&ftp.ftp_user, &ftp.ftp_password)
        .map_err(|e| format!("Erro ao autenticar FTP: {}", e))?;

    stream.transfer_type(suppaftp::types::FileType::Binary).ok();

    Ok(stream)
}

fn clean_remote_wwwroot(stream: &mut FtpStream) -> Result<(), String> {
    let entries = stream.nlst(Some("/wwwroot")).unwrap_or_default();
    for entry in entries {
        stream.rm(&entry).ok();
    }
    Ok(())
}

fn create_remote_dirs(
    stream: &mut FtpStream,
    local_dir: &Path,
    remote_dir: &str,
) -> Result<(), String> {
    stream.mkdir(remote_dir).ok();

    if let Ok(entries) = fs::read_dir(local_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name       = entry.file_name().to_string_lossy().to_string();
                let sub_remote = format!("{}/{}", remote_dir, name);
                create_remote_dirs(stream, &path, &sub_remote)?;
            }
        }
    }

    Ok(())
}

fn collect_files(local_dir: &Path, remote_dir: &str, files: &mut Vec<(PathBuf, String)>) {
    if let Ok(entries) = fs::read_dir(local_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                let sub_remote = format!("{}/{}", remote_dir, name);
                collect_files(&path, &sub_remote, files);
            } else {
                let remote_path = format!("{}/{}", remote_dir, name);
                files.push((path, remote_path));
            }
        }
    }
}