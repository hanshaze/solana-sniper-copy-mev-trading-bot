use chrono::Local;
use serde_json::json;
use std::{
    fs::OpenOptions,
    io::Write,
    sync::{Arc, Mutex},
};

const LOG_LEVEL: &str = "LOG";

#[derive(Clone)]
pub struct Logger {
    prefix: String,
    date_format: String,
    file: Arc<Mutex<std::fs::File>>,
}

impl Logger {
    // Constructor to create a new logger that writes to "logs.json"
    pub fn new(prefix: String) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("detectionlogs.json")
            .expect("Failed to open logs.json for writing");

        Logger {
            prefix,
            date_format: String::from("%Y-%m-%d %H:%M:%S"),
            file: Arc::new(Mutex::new(file)),
        }
    }

    pub fn log(&self, message: String) -> String {
        let full_log = format!("{} {}", self.prefix_with_date(), message);
        println!("{}", full_log);
        self.write_json("INFO", &message);
        full_log
    }

    pub fn debug(&self, message: String) -> String {
        let full_log = format!("{} [{}] {}", self.prefix_with_date(), "DEBUG", message);
        if LogLevel::new().is_debug() {
            println!("{}", full_log);
            self.write_json("DEBUG", &message);
        }
        full_log
    }

    pub fn error(&self, message: String) -> String {
        let full_log = format!("{} [{}] {}", self.prefix_with_date(), "ERROR", message);
        println!("{}", full_log);
        self.write_json("ERROR", &message);
        full_log
    }

    fn prefix_with_date(&self) -> String {
        let date = Local::now();
        format!("[{}] {}", date.format(&self.date_format), self.prefix)
    }

    fn write_json(&self, level: &str, message: &str) {
        let now = Local::now();
        let entry = json!({
            "timestamp": now.to_rfc3339(),
            "level": level,
            "prefix": self.prefix,
            "message": message,
        });

        if let Ok(mut file) = self.file.lock() {
            if let Err(e) = writeln!(file, "{}", entry) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }
    }
}

struct LogLevel<'a> {
    level: &'a str,
}
impl LogLevel<'_> {
    fn new() -> Self {
        LogLevel { level: LOG_LEVEL }
    }

    fn is_debug(&self) -> bool {
        self.level.to_lowercase() == "debug"
    }
}
