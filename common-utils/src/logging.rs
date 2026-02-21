use io::stdout;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread::panicking;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum LoggingLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}
impl LoggingLevel {
    fn to_str(&self) -> &'static str {
        match self {
            LoggingLevel::ERROR => "ERROR",
            LoggingLevel::WARN => "WARN",
            LoggingLevel::INFO => "INFO",
            LoggingLevel::DEBUG => "DEBUG",
            LoggingLevel::TRACE => "TRACE",
        }
    }
}
thread_local! {
    static MDC: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static LOGGING_LEVEL: RefCell<LoggingLevel> = RefCell::new(LoggingLevel::DEBUG);
    static THREAD_NAME: RefCell<String> = RefCell::new(String::new());
    static JOB_NAME: RefCell<String> = RefCell::new(String::new());
}

pub struct Logger;
impl Logger {
    pub fn new() -> Logger {
        Logger
    }
    fn format(level: LoggingLevel, message: &str) -> String {
        let mdc = MDC.with(|map| {
            let map = map.borrow();
            if map.is_empty() {
                String::new()
            } else {
                let kv_list: Vec<String> = map.iter().map(|(k ,v)| format!("{}: {}", k, v)).collect();
                format!(" [{}]", kv_list.join(", "))
            }
        });
        let mut thread_name = THREAD_NAME.with(|thread_name| thread_name.borrow().clone());
        if thread_name == "" {
            thread_name = std::thread::current().name().unwrap().to_string();
        }
        format!("[{}] ({}){} {}", level.to_str(), thread_name, mdc, message)
    }
    fn log(level: LoggingLevel, message: &str) {
        let thread_level = LOGGING_LEVEL.with(|level| {
            let level = level.borrow();
            level.clone()
        });
        if level <= thread_level {
            let _ = writeln!(stdout(), "{}", Self::format(level, message));
        }
    }

    fn set_logging_level(level: LoggingLevel) {
        LOGGING_LEVEL.set(level);
    }
    pub fn initialize() {
        Self::initialize_with(LoggingLevel::INFO);
    }
    pub fn initialize_with<T: Into<LoggingLevel>>(level: T) {
        Self::set_logging_level(level.into());
    }
    pub fn set_job_name(job_name: &str) {
        JOB_NAME.set(job_name.to_string());
    }
    pub fn set_level<T: Into<LoggingLevel>>(level: T) {
        Self::set_logging_level(level.into());
    }
    pub fn insert_mdc<K: Into<String>, V: Into<String>>(key: K, value: V) {
        MDC.with(|map| {
            map.borrow_mut().insert(key.into(), value.into());
        });
    }
    pub fn remove_mdc<K: AsRef<str>>(key: K) {
        MDC.with(|map| {
            map.borrow_mut().remove(key.as_ref());
        });
    }
    fn clear_mdc() {
        MDC.with(|map| {
            map.borrow_mut().clear();
        });
    }

    pub fn error(msg: &str) {
        Self::log(LoggingLevel::ERROR, msg);
    }
    pub fn warn(msg: &str) {
        Self::log(LoggingLevel::WARN, msg);
    }
    pub fn info(msg: &str) {
        Self::log(LoggingLevel::INFO, msg);
    }
    pub fn debug(msg: &str) {
        Self::log(LoggingLevel::DEBUG, msg);
    }
    pub fn trace(msg: &str) {
        Self::log(LoggingLevel::TRACE, msg);
    }
}
impl Drop for Logger {
    fn drop(&mut self) {
        if !panicking() {
            Self::clear_mdc();
            Self::log(LoggingLevel::TRACE, "Dropping logger");
        }
    }
}