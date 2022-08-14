use std::str::FromStr;

use logging::{Level, error};

pub fn handle_log_level_change<F: FnOnce(Level)>(log_level: String, callback: F) {
    match Level::from_str(log_level.as_str()) {
        Ok(level) => callback(level),
        Err(_) => error!("got unexpected log level from config"; "level" => log_level),
    };
}