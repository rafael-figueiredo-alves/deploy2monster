use crate::entities::{FtpSettings, Project};
use crate::shared::ftp_errors::{friendly_ftp_error, FileTransferFailure, FtpError};
use crate::shared::logger;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use suppaftp::FtpStream;

const PARALLEL_CONNECTIONS: usize = 3;

pub fn run(project: &Project) -> Result<(), FtpError> {
    let output_dir = Path::new(&project.publish_folder).join(&project.name);

    if !output_dir.exists() {
        return Err(FtpError::RemoteOutputDirMissing {
            path: output_dir.display().to_string(),
        });
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
    let chunks: Vec<&[(PathBuf, String)]> = files.chunks(chunk_size.max(1)).collect();

    let host = ftp.ftp_host.clone();
    let port = ftp.ftp_port;
    let user = ftp.ftp_user.clone();
    let password = ftp.ftp_password.clone();

    let results: Vec<Result<usize, FtpError>> = chunks
        .par_iter()
        .enumerate()
        .map(|(_thread_idx, chunk)| -> Result<usize, FtpError> {
            let host_port = format!("{}:{}", host, port);
            let target = host_port.clone();

            let mut stream = FtpStream::connect(&host_port).map_err(|e| FtpError::Connect {
                target: target.clone(),
                detail: friendly_ftp_error(&e.to_string(), &target),
            })?;

            stream.login(&user, &password).map_err(|e| FtpError::Login {
                target: target.clone(),
                detail: friendly_ftp_error(&e.to_string(), &target),
            })?;

            stream.transfer_type(suppaftp::types::FileType::Binary).ok();

            let mut uploaded = 0usize;
            let mut failures = Vec::new();

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

                if let Err(e) = stream.cwd(remote_dir) {
                    let detail = friendly_ftp_error(&e.to_string(), &target);
                    logger::warn(&format!("  ✘ {} — {}", remote_path, detail));
                    failures.push(FileTransferFailure {
                        path: remote_path.clone(),
                        detail: format!("Erro ao acessar diretório remoto '{}': {}", remote_dir, detail),
                    });
                    continue;
                }

                match fs::File::open(local_path) {
                    Ok(mut file) => match stream.put_file(&file_name, &mut file) {
                        Ok(_) => {
                            uploaded += 1;
                            logger::info(&format!("  ✔ {}", remote_path));
                        }
                        Err(e) => {
                            let detail = friendly_ftp_error(&e.to_string(), &target);
                            logger::warn(&format!("  ✘ {} — {}", remote_path, detail));
                            failures.push(FileTransferFailure {
                                path: remote_path.clone(),
                                detail,
                            });
                        }
                    },
                    Err(e) => {
                        let detail = format!("Falha ao abrir arquivo local '{}': {}", local_path.display(), e);
                        logger::warn(&format!("  ✘ {} — {}", remote_path, detail));
                        failures.push(FileTransferFailure {
                            path: remote_path.clone(),
                            detail,
                        });
                    }
                }
            }

            stream.quit().ok();

            if failures.is_empty() {
                Ok(uploaded)
            } else {
                Err(FtpError::UploadFailures {
                    total: chunk.len(),
                    failures,
                })
            }
        })
        .collect();

    let mut total_uploaded = 0usize;
    let mut total_errors = 0usize;
    for result in results {
        match result {
            Ok(n) => total_uploaded += n,
            Err(e) => {
                total_errors += 1;
                logger::error(&format!("{}", e));
            }
        }
    }

    if total_errors > 0 {
        logger::warn(&format!(
            "Upload concluído com problemas: {}/{} arquivos enviados.",
            total_uploaded, total
        ));
        return Err(FtpError::UploadFailures {
            total,
            failures: vec![FileTransferFailure {
                path: "/wwwroot".to_string(),
                detail: format!(
                    "Houve falhas em {} lote(s) de upload. Verifique os logs para mais detalhes.",
                    total_errors
                ),
            }],
        });
    }

    logger::info(&format!(
        "Upload concluído: {}/{} arquivos enviados.",
        total_uploaded, total
    ));

    Ok(())
}

fn connect(ftp: &FtpSettings) -> Result<FtpStream, FtpError> {
    let host_port = format!("{}:{}", ftp.ftp_host, ftp.ftp_port);

    let mut stream = FtpStream::connect(&host_port).map_err(|e| FtpError::Connect {
        target: host_port.clone(),
        detail: friendly_ftp_error(&e.to_string(), &host_port),
    })?;

    stream.login(&ftp.ftp_user, &ftp.ftp_password).map_err(|e| FtpError::Login {
        target: host_port.clone(),
        detail: friendly_ftp_error(&e.to_string(), &host_port),
    })?;

    stream.transfer_type(suppaftp::types::FileType::Binary).ok();

    Ok(stream)
}

fn clean_remote_wwwroot(stream: &mut FtpStream) -> Result<(), FtpError> {
    let entries = stream.nlst(Some("/wwwroot")).map_err(|e| FtpError::ListRemote {
        path: "/wwwroot".to_string(),
        detail: friendly_ftp_error(&e.to_string(), "/wwwroot"),
    })?;

    for entry in entries {
        stream.rm(&entry).map_err(|e| FtpError::RemoveRemote {
            path: entry.clone(),
            detail: friendly_ftp_error(&e.to_string(), &entry),
        })?;
    }

    Ok(())
}

fn create_remote_dirs(
    stream: &mut FtpStream,
    local_dir: &Path,
    remote_dir: &str,
) -> Result<(), FtpError> {
    stream.mkdir(remote_dir).map_err(|e| FtpError::MakeDir {
        path: remote_dir.to_string(),
        detail: friendly_ftp_error(&e.to_string(), remote_dir),
    })?;

    if let Ok(entries) = fs::read_dir(local_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
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
