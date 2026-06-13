use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use chrono::Local;

use crate::ui::{write_error, write_info, write_warning};

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init(project_name: &str) {
    let exe_path = std::env::current_exe().unwrap();
    let logs_dir = exe_path.parent().unwrap().join("logs");
    fs::create_dir_all(&logs_dir).unwrap();

    let timestamp = chrono_now_str();
    let filename = format!("deploy_{}_{}.log", project_name, timestamp);
    let path = logs_dir.join(filename);

    LOG_PATH.set(path).ok();
}

pub fn info(msg: &str) {
    log("INFO", msg);
}

pub fn warn(msg: &str) {
    log("WARN", msg);
}

pub fn error(msg: &str) {
    log("ERROR", msg);
}

pub fn log_path() -> Option<&'static PathBuf> {
    LOG_PATH.get()
}

fn log(level: &str, msg: &str) {
    let timestamp = chrono_now_str();
    let line = format!("[{}] [{}] {}", timestamp, level, msg);

    // tela
    match level {
        "ERROR" => write_error(&format!(" Erro: {}", msg)),
        "WARN"  => write_warning(&format!(" Atenção: {}", msg)),
        _       => write_info(&format!(" Info: {}", msg)),
    }

    // arquivo
    if let Some(path) = LOG_PATH.get() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            writeln!(file, "{}", line).ok();
        }
    }
}

fn chrono_now_str() -> String {
    Local::now().format("%Y%m%d_%H%M%S").to_string()
}