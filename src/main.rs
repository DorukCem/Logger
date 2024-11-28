#![feature(backtrace_frames)]

use logger::*;
use std::path::Path;
mod logger;

fn nest(mut logger : Logger){
    logger.critical("From nested");
    deep_nest(logger);
}

fn deep_nest(mut logger : Logger){
    logger.critical("From deep nest");
}


fn main() {
    let config = LogConfig::from_json_file(Path::new("./config.demo.json"));
    let mut logger = Logger::new(Some(config));
    
    logger.error("Hello world!");
    nest(logger);
}
