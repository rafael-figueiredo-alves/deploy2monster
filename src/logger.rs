use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

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
        "ERROR" => eprintln!("  ✘ {}", msg),
        "WARN"  => println!("  ⚠ {}", msg),
        _       => println!("  → {}", msg),
    }

    // arquivo
    if let Some(path) = LOG_PATH.get() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            writeln!(file, "{}", line).ok();
        }
    }
}

fn chrono_now_str() -> String {
    // sem chrono por ora — usa std::time para timestamp simples
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // ano aproximado para nome do arquivo
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;

    format!("{:04}{:02}{:02}_{:02}{:02}{:02}", year, month, day, h, m, s)
}