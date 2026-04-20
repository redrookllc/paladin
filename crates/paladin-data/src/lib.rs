use chrono::{DateTime, TimeZone, Utc};
use paladin_core::candle::{Candle, Interval};
use paladin_core::error::{Error, Result};
use paladin_core::paths;
use std::path::PathBuf;
use tracing::debug;

/// Default range string matching Yahoo Finance semantics for a given interval.
pub fn default_range_for(interval: Interval) -> &'static str {
    match interval {
        Interval::OneMinute | Interval::FiveMinute | Interval::FifteenMinute => "5d",
        Interval::OneHour => "60d",
        Interval::OneDay => "2y",
        Interval::OneWeek => "10y",
    }
}

fn cache_file(symbol: &str, interval: Interval) -> Result<PathBuf> {
    let dir = paths::cache_dir()?;
    let safe: String = symbol
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    Ok(dir.join(format!("{}_{}.json", safe, interval.as_yahoo())))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    ts: i64,
    candles: Vec<Candle>,
}

pub fn read_cache(symbol: &str, interval: Interval, max_age_secs: i64) -> Result<Option<Vec<Candle>>> {
    let path = cache_file(symbol, interval)?;
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)?;
    let entry: CacheEntry = serde_json::from_str(&text)?;
    let now = Utc::now().timestamp();
    if now - entry.ts > max_age_secs {
        return Ok(None);
    }
    Ok(Some(entry.candles))
}

pub fn write_cache(symbol: &str, interval: Interval, candles: &[Candle]) -> Result<()> {
    let path = cache_file(symbol, interval)?;
    let entry = CacheEntry {
        ts: Utc::now().timestamp(),
        candles: candles.to_vec(),
    };
    std::fs::write(&path, serde_json::to_vec_pretty(&entry)?)?;
    Ok(())
}

pub fn clear_cache() -> Result<usize> {
    let dir = paths::cache_dir()?;
    let mut count = 0;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.path().is_file() {
            std::fs::remove_file(entry.path())?;
            count += 1;
        }
    }
    Ok(count)
}

pub async fn fetch_history(
    symbol: &str,
    interval: Interval,
    range: Option<&str>,
) -> Result<Vec<Candle>> {
    let provider = yahoo_finance_api::YahooConnector::new()
        .map_err(|e| Error::Other(format!("yahoo connector: {e}")))?;
    let range_s = range.unwrap_or_else(|| default_range_for(interval)).to_string();
    debug!("yahoo fetch {} {} {}", symbol, interval.as_yahoo(), range_s);
    let response = provider
        .get_quote_range(symbol, interval.as_yahoo(), &range_s)
        .await
        .map_err(|e| Error::Other(format!("yahoo fetch {symbol}: {e}")))?;
    let quotes = response
        .quotes()
        .map_err(|e| Error::Other(format!("yahoo quotes {symbol}: {e}")))?;
    if quotes.is_empty() {
        return Err(Error::NoData(symbol.to_string()));
    }
    let candles = quotes
        .into_iter()
        .map(|q| {
            let ts: DateTime<Utc> = Utc
                .timestamp_opt(q.timestamp as i64, 0)
                .single()
                .unwrap_or_else(Utc::now);
            Candle {
                ts,
                open: q.open,
                high: q.high,
                low: q.low,
                close: q.close,
                volume: q.volume as f64,
            }
        })
        .collect();
    Ok(candles)
}

/// Fetch with cache (max 5 minutes stale).
pub async fn fetch_cached(
    symbol: &str,
    interval: Interval,
    range: Option<&str>,
) -> Result<Vec<Candle>> {
    if let Some(cached) = read_cache(symbol, interval, 300)? {
        debug!("cache hit {symbol}");
        return Ok(cached);
    }
    let candles = fetch_history(symbol, interval, range).await?;
    write_cache(symbol, interval, &candles)?;
    Ok(candles)
}
