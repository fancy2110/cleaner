// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::LazyCell;
use std::fmt::Debug;

use tracing::{debug, level_filters::LevelFilter};
use tracing_subscriber::fmt::{format, time, Layer};
use tracing_subscriber::{prelude::*, Registry};

fn main() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();

    // let fmt = format().with_timer(time::Uptime::default());

    // let stdout_layer = tracing_subscriber::fmt::layer::<Registry>()
    //     .event_format(fmt)
    //     .with_target(false)
    //     .pretty();
    // let _ = stdout_layer.with_subscriber(tracing_subscriber::registry::Registry::default());

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .event_format(format().compact())
        .with_writer(std::io::stdout)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    //
    desktop_lib::run()
}
