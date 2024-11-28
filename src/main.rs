use logger::*;
use std::path::Path;
mod logger;

fn main() {
    let config = LogConfig::from_json_file(Path::new("./config.demo.json"));
    let _logger = Logger::new(Some(config));
    
}
