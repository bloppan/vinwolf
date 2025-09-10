#![forbid(unsafe_code)]

use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::io::{self, Write};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Level { Error=1, Warn=2, Info=3, Debug=4, Trace=5 }

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LevelFilter { Off=0, Error=1, Warn=2, Info=3, Debug=4, Trace=5 }

pub struct Record<'a> {
    pub level: Level,
    pub target: &'a str,
    pub args: fmt::Arguments<'a>,
}

pub trait Log: Sync + Send + 'static {
    fn enabled(&self, level: Level) -> bool;
    fn log(&self, record: &Record);
}

static LOGGER: OnceLock<&'static dyn Log> = OnceLock::new();
static MAX_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);

pub fn set_logger<L: Log>(logger: &'static L) -> Result<(), ()> {
    LOGGER.set(logger).map_err(|_| ())
}

pub fn set_max_level(f: LevelFilter) {
    MAX_LEVEL.store(f as usize, Ordering::SeqCst);
}

pub fn max_level() -> LevelFilter {
    match MAX_LEVEL.load(Ordering::SeqCst) {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

pub fn __private_log(level: Level, target: &str, args: fmt::Arguments) {
    let filt = max_level() as usize;
    if level as usize > filt || filt == 0 { return; }
    if let Some(l) = LOGGER.get() {
        if l.enabled(level) {
            let rec = Record { level, target, args };
            l.log(&rec);
        }
    }
}

#[macro_export]
macro_rules! log {
    ($lvl:expr, $($arg:tt)+) => {{
        $crate::log::__private_log($lvl, module_path!(), format_args!($($arg)+));
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => { $crate::log::log!($crate::log::Level::Error, $($arg)+) }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => { $crate::log::log!($crate::log::Level::Warn, $($arg)+) }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => { $crate::log::log!($crate::log::Level::Info, $($arg)+) }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => { $crate::log::log!($crate::log::Level::Debug, $($arg)+) }
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => { $crate::log::log!($crate::log::Level::Trace, $($arg)+) }
}

pub use crate::{log as log, error as error, warn as warn, info as info, debug as debug, trace as trace};

pub mod prelude {
    pub use super::{Level, LevelFilter};
}

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _level: Level) -> bool { true }

    fn log(&self, record: &Record) {
        let ts = timestamp();
        let (lvl_str, color) = match record.level {
            Level::Error => ("ERROR", "\x1b[31m"),   // rojo
            Level::Warn  => ("WARN",  "\x1b[33m"),   // amarillo
            Level::Info  => ("INFO",  "\x1b[32m"),   // verde
            Level::Debug => ("DEBUG", "\x1b[34m"),   // azul
            Level::Trace => ("TRACE", "\x1b[35m"),   // magenta
        };
        let reset = "\x1b[0m";

        let mut w = std::io::stderr().lock();
        let _ = writeln!(
            w,
            "{} {}{}{} {}: {}",
            ts,
            color, lvl_str, reset,
            record.target,
            record.args
        );
    }
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;

    let days = secs / 86_400;
    let secs_of_day = (secs % 86_400) as i64;
    let mut year = 1970;
    let mut day_count = days;

    loop {
        let ydays = if is_leap(year) { 366 } else { 365 };
        if day_count >= ydays {
            day_count -= ydays;
            year += 1;
        } else {
            break;
        }
    }

    let month_days = if is_leap(year) {
        [31,29,31,30,31,30,31,31,30,31,30,31]
    } else {
        [31,28,31,30,31,30,31,31,30,31,30,31]
    };
    let mut month = 1;
    let mut day_in_month = day_count;
    for md in month_days.iter() {
        if day_in_month >= *md {
            day_in_month -= *md;
            month += 1;
        } else {
            break;
        }
    }
    let day = day_in_month + 1;

    let hour = secs_of_day / 3600;
    let min  = (secs_of_day % 3600) / 60;
    let sec  = secs_of_day % 60;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        year, month, day, hour, min, sec)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}


pub struct Env {
    var: String,
    default: Option<String>,
}

impl Env {
    pub fn default() -> Self {
        Env { var: "RUST_LOG".into(), default: None }
    }
    pub fn default_filter_or<S: Into<String>>(mut self, s: S) -> Self {
        self.default = Some(s.into());
        self
    }
}

pub struct Builder {
    lvl: LevelFilter,
    from_env: Option<Env>,
    load_dotenv: bool,
}

impl Builder {
    pub fn new() -> Self {
        Builder { lvl: LevelFilter::Info, from_env: None, load_dotenv: false }
    }
    pub fn from_env(envv: Env) -> Self {
        Builder { lvl: LevelFilter::Info, from_env: Some(envv), load_dotenv: false }
    }
    pub fn filter_level(mut self, lvl: LevelFilter) -> Self {
        self.lvl = lvl;
        self
    }
    pub fn with_dotenv(mut self, yes: bool) -> Self {
        self.load_dotenv = yes;
        self
    }
    pub fn init(self) {
        if self.load_dotenv { let _ = dotenv_load(".env"); }
        let lvl = resolve_level(self.from_env.as_ref(), self.lvl);
        set_max_level(lvl);
        static LOGGER: OnceLock<SimpleLogger> = OnceLock::new();
        let _ = set_logger(LOGGER.get_or_init(|| SimpleLogger {}));
    }
}

pub fn init() {
    let _ = dotenv_load(".env");
    let lvl = resolve_level(None, LevelFilter::Info);
    set_max_level(lvl);
    static LOGGER: OnceLock<SimpleLogger> = OnceLock::new();
    let _ = set_logger(LOGGER.get_or_init(|| SimpleLogger {}));
}

fn resolve_level(from_env: Option<&Env>, fallback: LevelFilter) -> LevelFilter {
    use std::env;
    if let Some(e) = from_env {
        if let Ok(v) = env::var(&e.var) {
            return parse_level(&v).unwrap_or(fallback);
        }
        if let Some(d) = &e.default {
            return parse_level(d).unwrap_or(fallback);
        }
    }
    if let Ok(v) = env::var("RUST_LOG") {
        return parse_level(&v).unwrap_or(fallback);
    }
    fallback
}

fn parse_level(s: &str) -> Option<LevelFilter> {
    let t = s.trim().to_ascii_lowercase();
    match t.as_str() {
        "off" => Some(LevelFilter::Off),
        "error" => Some(LevelFilter::Error),
        "warn" | "warning" => Some(LevelFilter::Warn),
        "info" => Some(LevelFilter::Info),
        "debug" => Some(LevelFilter::Debug),
        "trace" => Some(LevelFilter::Trace),
        _ => None,
    }
}

fn dotenv_load(path: &str) -> Result<(), ()> {
    use std::env;
    use std::fs;
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if line.starts_with('#') { continue; }
        let (k, v) = match line.split_once('=') {
            Some(x) => x,
            None => continue,
        };
        let key = k.trim();
        let val = dotenv_parse_value(v.trim());
        let _ = env::set_var(key, val);
    }
    Ok(())
}

fn dotenv_parse_value(s: &str) -> String {
    let mut out = String::new();
    let bytes = s.as_bytes();
    let quoted = (bytes.first() == Some(&b'"') && bytes.last() == Some(&b'"')) ||
                 (bytes.first() == Some(&b'\'') && bytes.last() == Some(&b'\''));
    if quoted && bytes.len() >= 2 {
        let quote = bytes[0];
        let mut i = 1;
        while i + 1 < bytes.len() {
            let c = bytes[i];
            if c == b'\\' && i + 2 < bytes.len() {
                let n = bytes[i + 1];
                match n {
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'"' => out.push('"'),
                    b'\'' => out.push('\''), 
                    b'\\' => out.push('\\'),
                    _ => out.push(n as char),
                }
                i += 2;
            } else if c == quote && i + 1 == bytes.len() - 1 {
                break;
            } else {
                out.push(c as char);
                i += 1;
            }
        }
        return out;
    }
    s.to_string()
}
