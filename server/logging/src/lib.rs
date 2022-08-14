use rand::{rngs, Rng};
use slog_term::{FullFormat, PlainSyncDecorator};
use std::{cell::RefCell, sync::Arc};

use std::io::Stderr;

use lazy_static::lazy_static;
use slog::*;
use slog_atomic::*;

pub use logging_macro::*;
pub use slog_scope::{scope, logger, error, warn, info, trace, debug, GlobalLoggerGuard};
pub use slog::{slog_o, FnValue, Level, Value, Record, Key, Serializer, Result};
pub use slog_scope_futures::FutureExt;

pub fn new_trace_id() -> String {
    let rng = CURRENT_RNG.with(|rng| rng.borrow_mut().gen::<[u8; 4]>());
    return format!("{:04x}", u32::from_be_bytes(rng));
}

pub fn set_level(level: Level) -> GlobalLoggerGuard {
    slog_stdlog::init_with_level(log::Level::Trace).err().or(None);
    let drain = Arc::new(logger_base(level).fuse());
    DRAIN_SWITCH.ctrl().set(drain.clone());
    slog_scope::set_global_logger(Logger::root(drain, o!()))
}

fn logger_base(level: Level) -> LevelFilter<Fuse<FullFormat<PlainSyncDecorator<Stderr>>>> {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let drain = slog_term::FullFormat::new(plain).build().fuse();
    drain.filter_level(level)
}

thread_local! {
    static CURRENT_RNG: RefCell<rngs::ThreadRng> = RefCell::new(rngs::ThreadRng::default());
}

lazy_static! {
    static ref DRAIN_SWITCH: AtomicSwitch<()> = {
        let logger = logger_base(Level::Info).fuse();
        AtomicSwitch::new(logger)
    };
}
