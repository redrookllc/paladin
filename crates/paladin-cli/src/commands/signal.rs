use crate::ui;
use console::style;
use paladin_ai::Client;
use paladin_core::candle::Interval;
use paladin_core::config::Config;
use paladin_core::error::{Error, Result};
use paladin_core::FeatureBundle;

pub async fn run(symbol: &str, interval: &str, range: Option<&str>, json_out: bool) -> Result<()> {
    let cfg = Config::load()?;
    let iv = Interval::parse(interval)
        .ok_or_else(|| Error::InvalidInput(format!("unknown interval: {interval}")))?;
    let fetch_sp = ui::spinner(&format!(
        "fetching {} ({}) history",
        style(symbol).magenta(),
        interval
    ));
    let candles = paladin_data::fetch_cached(symbol, iv, range).await?;
    fetch_sp.finish_and_clear();
    println!("  {} loaded {} candles", ui::ok(), candles.len());

    let features = FeatureBundle::from_candles(symbol, interval, &candles);

    let api_key = cfg.effective_api_key();
    let client = Client::new(cfg.api_base_url.clone(), api_key, cfg.model.clone())?;

    let infer_sp = ui::spinner(&format!(
        "reasoning with {}",
        style(&cfg.model).cyan()
    ));
    let signal = client.signal(&features).await?;
    infer_sp.finish_and_clear();

    if json_out {
        println!("{}", serde_json::to_string_pretty(&signal)?);
        return Ok(());
    }

    ui::section(&format!("{} — {}", signal.symbol, signal.pattern));
    println!(
        "  direction    {}  confidence {:>5.1}%",
        ui::verdict_style(&signal.direction),
        signal.confidence * 100.0
    );
    println!(
        "  entry {:>10.4}   stop {:>10.4}   target {:>10.4}   R:R {:>4.2}",
        signal.entry_price, signal.stop_loss, signal.take_profit, signal.risk_reward
    );
    println!(
        "  regime {}   divergence {}   vol {}   trend {:+.2}   confluence {}",
        style(&signal.regime).cyan(),
        style(&signal.divergence).cyan(),
        style(&signal.vol_state).cyan(),
        signal.trend_score,
        signal.confluence
    );
    if !signal.phases.is_empty() {
        ui::section("phases");
        for p in &signal.phases {
            println!(
                "  {} {}  {}",
                style(format!("[{}]", p.phase)).dim(),
                ui::verdict_style(p.verdict.as_str()),
                style(&p.title).bold()
            );
            if !p.detail.is_empty() {
                for line in textwrap(&p.detail, 80) {
                    println!("      {}", line);
                }
            }
        }
    }
    if !signal.reasoning.is_empty() {
        ui::section("reasoning");
        for line in textwrap(&signal.reasoning, 80) {
            println!("  {line}");
        }
    }
    Ok(())
}

fn textwrap(s: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for paragraph in s.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if line.len() + word.len() + 1 > width && !line.is_empty() {
                out.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            out.push(line);
        }
    }
    out
}
