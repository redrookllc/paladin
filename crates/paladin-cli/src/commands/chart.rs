use crate::ui;
use console::style;
use paladin_core::candle::Interval;
use paladin_core::error::{Error, Result};
use paladin_core::indicators;

pub async fn run(symbol: &str, interval: &str) -> Result<()> {
    let iv = Interval::parse(interval)
        .ok_or_else(|| Error::InvalidInput(format!("unknown interval: {interval}")))?;
    let sp = ui::spinner(&format!("fetching {}", style(symbol).magenta()));
    let candles = paladin_data::fetch_cached(symbol, iv, None).await?;
    sp.finish_and_clear();

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let min = closes.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = closes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let spark = sparkline(&closes, 60);

    ui::section(&format!("{} — {} candles", symbol, candles.len()));
    println!("  low {:.4}   high {:.4}", min, max);
    println!("  {}", style(spark).cyan());

    let rsi = indicators::last_finite(&indicators::rsi(&closes, 14)).unwrap_or(f64::NAN);
    let macd = indicators::macd(&closes, 12, 26, 9);
    println!(
        "  close {:.4}   rsi14 {:.1}   macd {:+.4}   hist {:+.4}",
        closes.last().copied().unwrap_or(0.0),
        rsi,
        indicators::last_finite(&macd.macd).unwrap_or(f64::NAN),
        indicators::last_finite(&macd.hist).unwrap_or(f64::NAN),
    );
    Ok(())
}

fn sparkline(values: &[f64], width: usize) -> String {
    if values.is_empty() {
        return String::new();
    }
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let step = (values.len() as f64 / width as f64).max(1.0);
    let mut sampled = Vec::with_capacity(width);
    let mut i = 0.0;
    while (i as usize) < values.len() && sampled.len() < width {
        sampled.push(values[i as usize]);
        i += step;
    }
    let min = sampled.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = sampled.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(1e-9);
    sampled
        .into_iter()
        .map(|v| {
            let n = ((v - min) / range * (chars.len() - 1) as f64).round() as usize;
            chars[n.min(chars.len() - 1)]
        })
        .collect()
}
