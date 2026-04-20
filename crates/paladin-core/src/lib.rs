pub mod candle;
pub mod config;
pub mod error;
pub mod features;
pub mod indicators;
pub mod paths;
pub mod signal;

pub use candle::{Candle, Interval};
pub use config::Config;
pub use error::{Error, Result};
pub use features::FeatureBundle;
pub use signal::{ChartAnnotation, ReasoningPhase, TradeSignal, Verdict};
