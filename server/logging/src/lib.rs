use rand::{rngs, Rng};
use slog_term::{FullFormat, PlainSyncDecorator};
use std::cell::RefCell;

use std::io::Stderr;

use lazy_static::lazy_static;
use slog::*;
use slog_atomic::*;

pub use logging_macro::*;
pub use slog::{slog_o, FnValue, Key, Level, Record, Result, Serializer, Value};
pub use slog_scope::{debug, error, info, logger, scope, trace, warn, GlobalLoggerGuard};
pub use slog_scope_futures::FutureExt;

type LoggerBase = Fuse<LevelFilter<Fuse<FullFormat<PlainSyncDecorator<Stderr>>>>>;

pub fn new_trace_id() -> String {
    let rng = CURRENT_RNG.with(|rng| rng.borrow_mut().gen::<[u8; 4]>());
    format!("{:04x}", u32::from_be_bytes(rng))
}

pub fn init_logger() -> GlobalLoggerGuard {
    slog_stdlog::init_with_level(log::Level::Debug).unwrap();
    slog_scope::set_global_logger(Logger::root(&*DRAIN_SWITCH, o!()))
}

pub fn set_level(level: Level) {
    let drain = match level {
        Level::Critical => &*ERROR_DRAIN,
        Level::Error => &*ERROR_DRAIN,
        Level::Warning => &*WARN_DRAIN,
        Level::Info => &*INFO_DRAIN,
        Level::Debug => &*DEBUG_DRAIN,
        Level::Trace => &*TRACE_DRAIN,
    };
    
    DRAIN_SWITCH.ctrl().set(drain);
    eprintln!("new level {}", level);
    
}

fn logger_base(level: Level) -> LoggerBase {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let drain = slog_term::FullFormat::new(plain).build().fuse();
    drain.filter_level(level).fuse()
}

thread_local! {
    static CURRENT_RNG: RefCell<rngs::ThreadRng> = RefCell::new(rngs::ThreadRng::default());
}

lazy_static! {
    static ref DRAIN_SWITCH: AtomicSwitch<()> = {
        AtomicSwitch::new(&*DEBUG_DRAIN)
    };
    static ref TRACE_DRAIN: LoggerBase = logger_base(Level::Trace);
    static ref DEBUG_DRAIN: LoggerBase = logger_base(Level::Debug);
    static ref INFO_DRAIN:  LoggerBase = logger_base(Level::Info);
    static ref WARN_DRAIN:  LoggerBase = logger_base(Level::Warning);
    static ref ERROR_DRAIN: LoggerBase = logger_base(Level::Error);
}
