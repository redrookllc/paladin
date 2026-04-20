use crate::candle::Candle;
use crate::indicators;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureBundle {
    pub symbol: String,
    pub interval: String,
    pub last_close: f64,
    pub change_pct_1d: f64,
    pub ema20: f64,
    pub ema50: f64,
    pub ema200: f64,
    pub rsi14: f64,
    pub macd: f64,
    pub macd_signal: f64,
    pub macd_hist: f64,
    pub atr14: f64,
    pub bb_upper: f64,
    pub bb_lower: f64,
    pub adx14: f64,
    pub recent: Vec<CandleLite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleLite {
    pub t: String,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
}

impl FeatureBundle {
    pub fn from_candles(symbol: &str, interval: &str, candles: &[Candle]) -> Self {
        let close: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let high: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let low: Vec<f64> = candles.iter().map(|c| c.low).collect();
        let last = *close.last().unwrap_or(&0.0);
        let prev = close
            .get(close.len().saturating_sub(2))
            .copied()
            .unwrap_or(last);
        let change_pct_1d = if prev != 0.0 {
            (last - prev) / prev * 100.0
        } else {
            0.0
        };
        let e20 = indicators::ema(&close, 20);
        let e50 = indicators::ema(&close, 50);
        let e200 = indicators::ema(&close, 200);
        let rsi = indicators::rsi(&close, 14);
        let macd = indicators::macd(&close, 12, 26, 9);
        let atr = indicators::atr(&high, &low, &close, 14);
        let bb = indicators::bollinger(&close, 20, 2.0);
        let adx = indicators::adx(&high, &low, &close, 14);
        let recent: Vec<CandleLite> = candles
            .iter()
            .rev()
            .take(30)
            .rev()
            .map(|c| CandleLite {
                t: c.ts.format("%Y-%m-%d %H:%M").to_string(),
                o: c.open,
                h: c.high,
                l: c.low,
                c: c.close,
                v: c.volume,
            })
            .collect();
        Self {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            last_close: last,
            change_pct_1d,
            ema20: indicators::last_finite(&e20).unwrap_or(f64::NAN),
            ema50: indicators::last_finite(&e50).unwrap_or(f64::NAN),
            ema200: indicators::last_finite(&e200).unwrap_or(f64::NAN),
            rsi14: indicators::last_finite(&rsi).unwrap_or(f64::NAN),
            macd: indicators::last_finite(&macd.macd).unwrap_or(f64::NAN),
            macd_signal: indicators::last_finite(&macd.signal).unwrap_or(f64::NAN),
            macd_hist: indicators::last_finite(&macd.hist).unwrap_or(f64::NAN),
            atr14: indicators::last_finite(&atr).unwrap_or(f64::NAN),
            bb_upper: indicators::last_finite(&bb.upper).unwrap_or(f64::NAN),
            bb_lower: indicators::last_finite(&bb.lower).unwrap_or(f64::NAN),
            adx14: indicators::last_finite(&adx).unwrap_or(f64::NAN),
            recent,
        }
    }
}
