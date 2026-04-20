use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub const PROGRESS_BAR_CHARS: &str = "█▓▒░ ";
pub const PROGRESS_BAR_TEMPLATE: &str =
    "{msg} {bar:40.cyan/blue} {bytes}/{total_bytes} {bytes_per_sec}  eta {eta}";
pub const SPINNER_TICK_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars(SPINNER_TICK_CHARS),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message(msg.to_string());
    pb
}

pub fn arrow() -> console::StyledObject<&'static str> {
    style("→").cyan()
}

pub fn ok() -> console::StyledObject<&'static str> {
    style("✓").green()
}

pub fn warn_glyph() -> console::StyledObject<&'static str> {
    style("!").yellow()
}

pub fn fail() -> console::StyledObject<&'static str> {
    style("✗").red()
}

pub fn error_prefix() -> console::StyledObject<&'static str> {
    style("error:").red().bold()
}

pub fn verdict_style(v: &str) -> console::StyledObject<String> {
    match v.to_ascii_uppercase().as_str() {
        "BULLISH" | "LONG" => style(v.to_string()).green().bold(),
        "BEARISH" | "SHORT" => style(v.to_string()).red().bold(),
        "CAUTION" => style(v.to_string()).yellow().bold(),
        _ => style(v.to_string()).white().bold(),
    }
}

pub fn section(title: &str) {
    println!("\n{} {}", arrow(), style(title).bold());
}
