use tracing_appender::{non_blocking::WorkerGuard, rolling::{daily}};
use tracing_subscriber::{filter, fmt, prelude::*, util::SubscriberInitExt};

pub fn initialize_logger(file_name: &str) -> WorkerGuard {

    let file_appender = daily("./log", file_name);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer().with_writer(non_blocking).json().pretty();

    let stdout_log = tracing_subscriber::fmt::layer().compact();
    tracing_subscriber::registry().with(filter::LevelFilter::INFO).with(stdout_log).with(file_layer).init();
    _guard
}