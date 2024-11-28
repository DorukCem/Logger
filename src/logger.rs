use std::{
    backtrace::{self, Backtrace},
    env, fmt,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use chrono::{self, Utc};
use serde_json::Value;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Critical = 4,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl LogLevel {
    fn from_u64(num: u64) -> Result<Self, ()> {
        match num {
            0 => Ok(Self::Debug),
            1 => Ok(Self::Info),
            2 => Ok(Self::Warn),
            3 => Ok(Self::Error),
            4 => Ok(Self::Critical),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Logger {
    config: LogConfig,
    log_file: File,
}

impl Logger {
    pub fn new(config: Option<LogConfig>) -> Self {
        let config = config.unwrap_or_else(LogConfig::new);
        let mut path = PathBuf::from(env::current_dir().unwrap());
        path.push("logs");
        path.push(format!(
            "{}{}",
            &config.file_prefix,
            &Utc::now()
                .to_string()
                .replace(['.', ':'], "-")
                .replace(' ', "T")
        ));
        path.set_extension("log");

        fs::create_dir_all(env::current_dir().unwrap().join("logs")).unwrap();
        let log_file = File::create(path).unwrap();
        Self { config, log_file }
    }

    fn log(&mut self, message: &str, log_level: LogLevel) {
        let bt = Backtrace::force_capture();
        let caller_name = bt
            .frames()
            .iter()
            .rev()
            .nth_back(4)
            .expect("Could not get caller name");

        if log_level >= self.config.level {
            self.log_file
                .write(format!("{}:{:?} {}\n", log_level, caller_name, message).as_bytes())
                .unwrap();
        }
    }

    // Public methods for different log levels
    pub fn debug(&mut self, message: &str) {
        self.log(message, LogLevel::Debug);
    }

    pub fn info(&mut self, message: &str) {
        self.log(message, LogLevel::Info);
    }

    pub fn warn(&mut self, message: &str) {
        self.log(message, LogLevel::Warn);
    }

    pub fn error(&mut self, message: &str) {
        self.log(message, LogLevel::Error);
    }

    pub fn critical(&mut self, message: &str) {
        self.log(message, LogLevel::Critical);
    }
}

#[derive(Debug)]
pub struct LogConfig {
    level: LogLevel,
    rolling_config: RollingConfig,
    file_prefix: String,
}

impl LogConfig {
    pub fn new() -> Self {
        Self {
            level: LogLevel::Info,
            rolling_config: RollingConfig::new(),
            file_prefix: "Logtar_".to_string(),
        }
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }
    pub fn with_rolling_config(mut self, config: RollingConfig) -> Self {
        self.rolling_config = config;
        self
    }
    pub fn with_file_prefix(mut self, prefix: String) -> Self {
        self.file_prefix = prefix;
        self
    }

    pub fn from_json_file(file_path: &Path) -> Self {
        let mut config_file = "".to_string();
        File::open(file_path)
            .expect("LogLevel reading config")
            .read_to_string(&mut config_file)
            .unwrap();

        let js: Value = serde_json::from_str(&config_file).unwrap();
        let keys = js.as_object().expect("Cannot convert JSON to Dictionary");
        let mut config = Self::new();
        for (k, v) in keys {
            match k.as_str() {
                "level" => {
                    config = config.with_level(
                        LogLevel::from_u64(v.as_u64().expect("expexted config level to be number"))
                            .expect("Expected log level to be in range 0..=4"),
                    )
                }

                "rolling_config" => {
                    config = config.with_rolling_config(RollingConfig::from_json(v))
                }
                "file_prefix" => {
                    config = config.with_file_prefix(
                        v.as_str()
                            .expect("Expected file prefix to be a string")
                            .to_string(),
                    )
                }
                _ => continue,
            }
        }

        config
    }
}

#[derive(Debug)]
pub struct RollingConfig {
    time_threshold: RollingTimeOptions,
    size_threshold: RollingSizeOptions,
}

impl RollingConfig {
    fn new() -> Self {
        Self {
            time_threshold: RollingTimeOptions::Hourly,
            size_threshold: RollingSizeOptions::FiveMB,
        }
    }

    pub fn with_time_threshold(mut self, time: RollingTimeOptions) -> Self {
        self.time_threshold = time;
        self
    }

    pub fn with_size_threshold(mut self, size: RollingSizeOptions) -> Self {
        self.size_threshold = size;
        self
    }

    pub fn from_json(json_value: &Value) -> Self {
        let keys = json_value
            .as_object()
            .expect("Cannot convert JSON to Dictionary");
        let mut rolling_config = Self::new();
        for (k, v) in keys {
            match k.as_str() {
                "size_threshold" => {
                    rolling_config = rolling_config.with_size_threshold(
                        RollingSizeOptions::from_u64(
                            v.as_u64().expect("expexted size threshold to be number"),
                        )
                        .unwrap(),
                    )
                }
                "time_threshold" => {
                    rolling_config = rolling_config.with_time_threshold(
                        RollingTimeOptions::from_u64(
                            v.as_u64().expect("expexted size threshold to be number"),
                        )
                        .unwrap(),
                    )
                }
                _ => continue,
            }
        }

        rolling_config
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RollingSizeOptions {
    OneKB = 1024,
    FiveKB = 5 * 1024,
    TenKB = 10 * 1024,
    TwentyKB = 20 * 1024,
    FiftyKB = 50 * 1024,
    HundredKB = 100 * 1024,
    HalfMB = 512 * 1024,
    OneMB = 1024 * 1024,
    FiveMB = 5 * 1024 * 1024,
    TenMB = 10 * 1024 * 1024,
    TwentyMB = 20 * 1024 * 1024,
    FiftyMB = 50 * 1024 * 1024,
    HundredMB = 100 * 1024 * 1024,
}

impl RollingSizeOptions {
    pub fn from_u64(value: u64) -> Result<Self, &'static str> {
        match value {
            x if x == RollingSizeOptions::OneKB as u64 => Ok(RollingSizeOptions::OneKB),
            x if x == RollingSizeOptions::FiveKB as u64 => Ok(RollingSizeOptions::FiveKB),
            x if x == RollingSizeOptions::TenKB as u64 => Ok(RollingSizeOptions::TenKB),
            x if x == RollingSizeOptions::TwentyKB as u64 => Ok(RollingSizeOptions::TwentyKB),
            x if x == RollingSizeOptions::FiftyKB as u64 => Ok(RollingSizeOptions::FiftyKB),
            x if x == RollingSizeOptions::HundredKB as u64 => Ok(RollingSizeOptions::HundredKB),
            x if x == RollingSizeOptions::HalfMB as u64 => Ok(RollingSizeOptions::HalfMB),
            x if x == RollingSizeOptions::OneMB as u64 => Ok(RollingSizeOptions::OneMB),
            x if x == RollingSizeOptions::FiveMB as u64 => Ok(RollingSizeOptions::FiveMB),
            x if x == RollingSizeOptions::TenMB as u64 => Ok(RollingSizeOptions::TenMB),
            x if x == RollingSizeOptions::TwentyMB as u64 => Ok(RollingSizeOptions::TwentyMB),
            x if x == RollingSizeOptions::FiftyMB as u64 => Ok(RollingSizeOptions::FiftyMB),
            x if x == RollingSizeOptions::HundredMB as u64 => Ok(RollingSizeOptions::HundredMB),
            _ => Err("Value does not match any known size option"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RollingTimeOptions {
    Minutely = 60,
    Hourly = 60 * 60,
    Daily = 24 * 60 * 60,
    Weekly = 7 * 24 * 60 * 60,
    Monthly = 30 * 24 * 60 * 60,
    Yearly = 12 * 30 * 24 * 60 * 60,
}

impl RollingTimeOptions {
    fn from_u64(value: u64) -> Result<Self, &'static str> {
        match value {
            x if x == RollingTimeOptions::Minutely as u64 => Ok(RollingTimeOptions::Minutely),
            x if x == RollingTimeOptions::Hourly as u64 => Ok(RollingTimeOptions::Hourly),
            x if x == RollingTimeOptions::Daily as u64 => Ok(RollingTimeOptions::Daily),
            x if x == RollingTimeOptions::Weekly as u64 => Ok(RollingTimeOptions::Weekly),
            x if x == RollingTimeOptions::Monthly as u64 => Ok(RollingTimeOptions::Monthly),
            x if x == RollingTimeOptions::Yearly as u64 => Ok(RollingTimeOptions::Yearly),
            _ => Err("Value does not match any known time option"),
        }
    }
}
