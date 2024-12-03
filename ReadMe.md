# Logtar - A Flexible Logging Library for Rust

Logtar is a customizable logging library for Rust applications, featuring rolling log files based on size and time thresholds. It allows fine-grained control over logging levels, log file configurations, and dynamic configuration via JSON files.

---

## Features

- **Log Levels:** Supports `Debug`, `Info`, `Warn`, `Error`, and `Critical` log levels.
- **Rolling Logs:** Automatically rotates log files based on:
  - File size.
  - Time thresholds (e.g., hourly, daily).
- **Dynamic Configuration:** Configure logging behavior using JSON files.
- **Backtrace Integration:** Includes the caller's backtrace information in log entries for better debugging.
- **Customizable File Prefix:** Allows customization of log file names with prefixes.

---

## USAGE

```rs
let config = LogConfig::new()
        .with_level(LogLevel::Debug) // Set log level
        .with_file_prefix("CustomLog_".to_string()) // Set log file prefix
        .with_rolling_config(
            RollingConfig::new()
                .with_size_threshold(RollingSizeOptions::OneMB) // Rotate logs at 1 MB
                .with_time_threshold(RollingTimeOptions::Daily), // Rotate logs daily
        );

    let mut logger = Logger::new(Some(config));

    logger.info("Custom configuration applied!");
```

Example log_config.json
```json
{
  "level": 1,
  "rolling_config": {
    "size_threshold": 5242880,
    "time_threshold": 3600
  },
  "file_prefix": "AppLog_"
}

```