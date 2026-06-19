use std::path::PathBuf;
use std::fs;

pub struct LogEntry {
    pub filename:   String,
    pub path:       PathBuf,
    pub created_at: String,
    pub size_kb:    u64,
}

pub fn list_logs(project_name: &str) -> Result<Vec<LogEntry>, String> {
    let logs_dir = logs_dir()?;

    if !logs_dir.exists() {
        return Ok(Vec::new());
    }

    let prefix = format!("deploy_{}_", project_name.to_lowercase());

    let mut entries: Vec<LogEntry> = fs::read_dir(&logs_dir)
        .map_err(|e| format!("Erro ao ler pasta de logs: {}", e))?
        .flatten()
        .filter_map(|entry| {
            let fname = entry.file_name().to_string_lossy().to_lowercase();
            if !fname.starts_with(&prefix) || !fname.ends_with(".log") {
                return None;
            }

            let metadata = entry.metadata().ok()?;
            let size_kb  = metadata.len() / 1024;

            // extrai timestamp do nome: deploy_<projeto>_<YYYYMMDD_HHMMSS>.log
            let created_at = fname
                .trim_start_matches(&prefix)
                .trim_end_matches(".log")
                .replace('_', " ")
                .to_string();

            Some(LogEntry {
                filename:   entry.file_name().to_string_lossy().to_string(),
                path:       entry.path(),
                created_at: format_timestamp(&created_at),
                size_kb,
            })
        })
        .collect();

    // mais recente primeiro
    entries.sort_by(|a, b| b.filename.cmp(&a.filename));

    Ok(entries)
}

pub fn open_log(path: &PathBuf) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("Erro ao ler arquivo de log: {}", e))
}

pub fn delete_old_logs(project_name: &str, retention_days: u64) -> Result<usize, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let logs_dir = logs_dir()?;
    if !logs_dir.exists() {
        return Ok(0);
    }

    let prefix   = format!("deploy_{}_", project_name.to_lowercase());
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let cutoff = now_secs - (retention_days * 86400);
    let mut deleted = 0;

    for entry in fs::read_dir(&logs_dir).map_err(|e| e.to_string())?.flatten() {
        let fname = entry.file_name().to_string_lossy().to_lowercase();
        if !fname.starts_with(&prefix) || !fname.ends_with(".log") {
            continue;
        }

        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                let modified_secs = modified
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if modified_secs < cutoff {
                    fs::remove_file(entry.path()).ok();
                    deleted += 1;
                }
            }
        }
    }

    Ok(deleted)
}

fn logs_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Erro ao obter executável: {}", e))?;

    Ok(exe.parent()
        .ok_or("Erro ao obter pasta do executável")?
        .join("logs"))
}

fn format_timestamp(raw: &str) -> String {
    // raw: "20250603 143022"
    if raw.len() < 15 {
        return raw.to_string();
    }
    let date = &raw[..8];
    let time = &raw[9..];

    let year  = &date[..4];
    let month = &date[4..6];
    let day   = &date[6..8];
    let hour  = &time[..2];
    let min   = &time[2..4];
    let sec   = &time[4..6];

    format!("{}/{}/{} {}:{}:{}", day, month, year, hour, min, sec)
}