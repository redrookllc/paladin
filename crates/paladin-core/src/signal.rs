use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Verdict {
    Bullish,
    Bearish,
    Neutral,
    Caution,
}

impl Default for Verdict {
    fn default() -> Self {
        Verdict::Neutral
    }
}

impl Verdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Verdict::Bullish => "BULLISH",
            Verdict::Bearish => "BEARISH",
            Verdict::Neutral => "NEUTRAL",
            Verdict::Caution => "CAUTION",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChartAnnotation {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub price2: f64,
    #[serde(default = "neg_one")]
    pub xi: i64,
    #[serde(default)]
    pub label: String,
    #[serde(default = "white")]
    pub color: String,
    #[serde(default = "default_alpha")]
    pub alpha: f64,
    #[serde(default)]
    pub phase: u32,
    #[serde(default)]
    pub tooltip: String,
}

fn neg_one() -> i64 {
    -1
}
fn white() -> String {
    "#ffffff".into()
}
fn default_alpha() -> f64 {
    0.7
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReasoningPhase {
    #[serde(default)]
    pub phase: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub verdict: Verdict,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradeSignal {
    #[serde(default)]
    pub symbol: String,
    #[serde(default = "hold")]
    pub direction: String,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub entry_price: f64,
    #[serde(default)]
    pub stop_loss: f64,
    #[serde(default)]
    pub take_profit: f64,
    #[serde(default)]
    pub risk_reward: f64,
    #[serde(default = "dash")]
    pub pattern: String,
    #[serde(default)]
    pub reasoning: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub annotations: Vec<ChartAnnotation>,
    #[serde(default)]
    pub phases: Vec<ReasoningPhase>,
    #[serde(default = "ranging")]
    pub regime: String,
    #[serde(default = "none_str")]
    pub divergence: String,
    #[serde(default = "normal_str")]
    pub vol_state: String,
    #[serde(default)]
    pub trend_score: f64,
    #[serde(default)]
    pub confluence: u32,
}

fn hold() -> String {
    "HOLD".into()
}
fn dash() -> String {
    "—".into()
}
fn default_source() -> String {
    "OpenAI-compatible".into()
}
fn ranging() -> String {
    "RANGING".into()
}
fn none_str() -> String {
    "NONE".into()
}
fn normal_str() -> String {
    "NORMAL".into()
}
