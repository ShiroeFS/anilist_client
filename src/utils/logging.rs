use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

/// Initialize the logger with custom formatting
pub fn setup_logger() -> Result<(), log::SetLoggerError> {
    let mut builder = Builder::new();

    // Set default log level based on debug/release mode
    #[cfg(debug_assertions)]
    let default_level = LevelFilter::Debug;

    #[cfg(not(debug_assertions))]
    let default_level = LevelFilter::Info;

    // Use RUST_LOG env var if set, otherwise use our default
    builder.filter_level(default_level);

    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        builder.parse_filters(&rust_log);
    }

    // Add timestamps and module path to log output
    builder.format(|buf, record| {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        writeln!(
            buf,
            "[{} {} {}:{}] {}",
            timestamp,
            record.level(),
            record.module_path().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        )
    });

    builder.init();

    log::info!("Logger initialized at level: {:?}", default_level);

    Ok(())
}
