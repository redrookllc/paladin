use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Candle {
    pub ts: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    OneMinute,
    FiveMinute,
    FifteenMinute,
    OneHour,
    OneDay,
    OneWeek,
}

impl Interval {
    pub fn as_yahoo(&self) -> &'static str {
        match self {
            Interval::OneMinute => "1m",
            Interval::FiveMinute => "5m",
            Interval::FifteenMinute => "15m",
            Interval::OneHour => "1h",
            Interval::OneDay => "1d",
            Interval::OneWeek => "1wk",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "1m" => Interval::OneMinute,
            "5m" => Interval::FiveMinute,
            "15m" => Interval::FifteenMinute,
            "1h" | "60m" => Interval::OneHour,
            "1d" => Interval::OneDay,
            "1wk" | "1w" => Interval::OneWeek,
            _ => return None,
        })
    }
}
