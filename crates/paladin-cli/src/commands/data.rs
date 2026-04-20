use crate::ui;
use console::style;
use paladin_core::candle::Interval;
use paladin_core::error::{Error, Result};

pub async fn fetch(symbol: &str, interval: &str) -> Result<()> {
    let iv = Interval::parse(interval)
        .ok_or_else(|| Error::InvalidInput(format!("unknown interval: {interval}")))?;
    let sp = ui::spinner(&format!("fetching {}", style(symbol).magenta()));
    let candles = paladin_data::fetch_history(symbol, iv, None).await?;
    paladin_data::write_cache(symbol, iv, &candles)?;
    sp.finish_and_clear();
    println!("  {} cached {} candles", ui::ok(), candles.len());
    Ok(())
}

pub fn clear() -> Result<()> {
    let n = paladin_data::clear_cache()?;
    println!("  {} cleared {} cache file(s)", ui::ok(), n);
    Ok(())
}
