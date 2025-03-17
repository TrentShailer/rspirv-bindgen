//! # rspirv-bindgen
//! CLI to generate Rust bindings for SPIR-V shaders.
//!

use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use tracing::{Level, subscriber::set_global_default};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;

mod cli;

fn main() -> Result<()> {
    color_eyre::install()?;
    let _loggers = setup_logger(false);

    let cli = Cli::parse();
    let modules = cli.read_source()?;
    cli.write_output(modules)?;

    Ok(())
}

/// Initializes the tracing logger.
fn setup_logger(should_debug: bool) -> [WorkerGuard; 1] {
    let level = if should_debug {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let filter = tracing_subscriber::filter::Targets::new().with_default(level);

    // stdout logger
    let (std_writer, _std_guard) = tracing_appender::non_blocking(std::io::stderr());
    let std_logger = tracing_subscriber::fmt::layer()
        .with_writer(std_writer)
        .with_ansi(true)
        .with_target(false)
        .without_time();

    // Register loggers
    let collector = tracing_subscriber::registry().with(std_logger).with(filter);

    set_global_default(collector).expect("Failed to set global logger");

    [_std_guard]
}
