// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing::{debug, instrument::WithSubscriber, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Registry,
};

fn main() {
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();

    // 文件 appender 指定日志文件输出目录和文件名前缀
    // daily 指定生成文件名日期到年月日
    // 如： test-log.2023-08-30
    let file_appender = tracing_appender::rolling::daily(
        "/Users/xiaocy/Documents/fancy2110/code/rs-plugin-test/logs",
        "test-log",
    );
    // 生成非阻塞写入器
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        // .event_format(format().compact())
        .with_writer(non_blocking)
        .init();

    //
    desktop_lib::run()
}
