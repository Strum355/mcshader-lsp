use rand::{rngs, Rng};
use slog::slog_o;
use slog_scope::GlobalLoggerGuard;
use slog_term::{FullFormat, PlainSyncDecorator};
use std::{cell::RefCell, io::Stderr, sync::Arc};
use lazy_static::lazy_static;
use slog::*;
use slog_atomic::*;

fn new_trace_id() -> String {
    let rng = CURRENT_RNG.with(|rng| rng.borrow_mut().gen::<[u8; 4]>());
    return format!("{:04x}", u32::from_be_bytes(rng));
}

pub fn slog_with_trace_id<F: FnOnce()>(f: F) {
    slog_scope::scope(&slog_scope::logger().new(slog_o!("trace" => new_trace_id())), || {
        f()
    })
}

pub fn set_logger_with_level(level: Level) -> GlobalLoggerGuard {
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