use paladin_core::error::Result;
use paladin_core::paths;
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub fn init(verbose: bool) -> Result<()> {
    let log_dir = paths::logs_dir()?;
    std::fs::create_dir_all(&log_dir)?;
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("paladin.log"))?;
    let level = if verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(log_file.with_max_level(Level::TRACE))
        .with_ansi(false)
        .init();
    Ok(())
}
