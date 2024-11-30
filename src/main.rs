#![feature(backtrace_frames)]

use logger::*;
use core::time;
use std::{path::Path, thread::sleep};
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

    for i in 0..=6 {
        logger.critical(&format!("message in second {i}"));
        sleep(time::Duration::from_secs(1));
    }

}
