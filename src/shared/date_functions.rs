use chrono::Local;
use chrono::Datelike;

pub fn current_year() -> i32 {
    Local::now().year()
}

pub fn chrono_now_str() -> String {
    Local::now().format("%Y%m%d_%H%M%S").to_string()
}